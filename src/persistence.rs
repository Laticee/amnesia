use argon2::{password_hash::SaltString, Argon2};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::{rngs::OsRng, RngCore};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zeroize::Zeroize;

const MAGIC_BYTES: &[u8; 8] = b"AMNESIO2"; // Version 2 uses Argon2id
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Encryption(String),
    InvalidFileFormat,
    DecryptionFailed,
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::Io(e)
    }
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "IO Error: {}", e),
            PersistenceError::Encryption(e) => write!(f, "Encryption Error: {}", e),
            PersistenceError::InvalidFileFormat => {
                write!(f, "Invalid file format (not a v2 .amnesio file)")
            }
            PersistenceError::DecryptionFailed => write!(f, "Decryption failed (wrong password?)"),
        }
    }
}

impl std::error::Error for PersistenceError {}

pub fn save_encrypted<P: AsRef<Path>>(
    path: P,
    content: &str,
    password: &str,
) -> Result<(), PersistenceError> {
    // 1. Generate Salt and Nonce
    let mut salt_bytes = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt_bytes);
    OsRng.fill_bytes(&mut nonce_bytes);

    // 2. Derive Key using Argon2id
    let mut key_bytes = [0u8; KEY_LEN];
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| PersistenceError::Encryption(e.to_string()))?;

    // We use default params for simplicity, but it's significantly stronger than PBKDF2
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(
            password.as_bytes(),
            salt.as_str().as_bytes(),
            &mut key_bytes,
        )
        .map_err(|e| PersistenceError::Encryption(e.to_string()))?;

    let cipher_key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 3. Encrypt
    let ciphertext = cipher
        .encrypt(nonce, content.as_bytes())
        .map_err(|_| PersistenceError::Encryption("Encryption failed".into()))?;

    // 4. Write to File: [MAGIC] [SALT_BYTES] [NONCE] [CIPHERTEXT]
    let mut file = File::create(&path)?;
    file.write_all(MAGIC_BYTES)?;
    file.write_all(&salt_bytes)?;
    file.write_all(&nonce_bytes)?;
    file.write_all(&ciphertext)?;

    // 5. Make Read-Only (Safety)
    let mut perms = file.metadata()?.permissions();
    perms.set_readonly(true);
    file.set_permissions(perms)?;

    // Zeroize key
    key_bytes.zeroize();

    Ok(())
}

pub fn load_encrypted<P: AsRef<Path>>(path: P, password: &str) -> Result<String, PersistenceError> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < MAGIC_BYTES.len() + SALT_LEN + NONCE_LEN {
        return Err(PersistenceError::InvalidFileFormat);
    }

    // 1. Verify Magic
    if &buffer[0..MAGIC_BYTES.len()] != MAGIC_BYTES.as_slice() {
        return Err(PersistenceError::InvalidFileFormat);
    }

    let salt_offset = MAGIC_BYTES.len();
    let nonce_offset = salt_offset + SALT_LEN;
    let ciphertext_offset = nonce_offset + NONCE_LEN;

    let salt_bytes = &buffer[salt_offset..nonce_offset];
    let nonce_bytes = &buffer[nonce_offset..ciphertext_offset];
    let ciphertext = &buffer[ciphertext_offset..];

    // 2. Derive Key
    let mut key_bytes = [0u8; KEY_LEN];
    let salt = SaltString::encode_b64(salt_bytes)
        .map_err(|e| PersistenceError::Encryption(e.to_string()))?;

    let argon2 = Argon2::default();
    argon2
        .hash_password_into(
            password.as_bytes(),
            salt.as_str().as_bytes(),
            &mut key_bytes,
        )
        .map_err(|e| PersistenceError::Encryption(e.to_string()))?;

    let cipher_key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 3. Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| PersistenceError::DecryptionFailed)?;

    let plaintext = String::from_utf8(plaintext_bytes)
        .map_err(|_| PersistenceError::Encryption("Decrypted content is not valid UTF-8".into()))?;

    key_bytes.zeroize();

    Ok(plaintext)
}
