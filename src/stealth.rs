use sha2::{Digest, Sha256};
use std::process::Command;
use zeroize::Zeroize;

/// A static variable to leverage ASLR in key derivation.
static ASLR_ANCHOR: u8 = 0xAA;

/// Derives a 32-byte key using system data, ASLR, and startup randomness.
/// This makes it difficult to reproduce the key from a memory dump.
pub fn derive_key() -> [u8; 32] {
    let mut entropy = Vec::new();

    // 1. System Hostname
    if let Ok(output) = Command::new("hostname").output() {
        entropy.extend_from_slice(&output.stdout);
    }

    // 2. Kernel Version / System Info
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        entropy.extend_from_slice(&output.stdout);
    }

    // 3. Boot Time (macOS specific, fallback to 0 if fails)
    let boot_time = capture_boot_time();
    entropy.extend_from_slice(&boot_time.to_le_bytes());

    // 4. ASLR-based address of a static variable
    let aslr_addr = &ASLR_ANCHOR as *const u8 as usize;
    entropy.extend_from_slice(&aslr_addr.to_le_bytes());

    // 5. Ephemeral Startup Randomness
    let mut startup_random = [0u8; 32];
    if getrandom::getrandom(&mut startup_random).is_err() {
        // Fallback to some "random" looking static data if getrandom fails (unlikely)
        startup_random.copy_from_slice(b"AMNESIA_STEALTH_FALLBACK_RANDOM_");
    }
    entropy.extend_from_slice(&startup_random);

    // 6. Creative Shuffling (simple but non-obvious)
    creative_shuffle(&mut entropy);

    // 7. Hash the collected entropy to get the final key
    let mut hasher = Sha256::new();
    hasher.update(&entropy);
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);

    // Cleanup entropy
    entropy.zeroize();
    startup_random.zeroize();

    key
}

fn capture_boot_time() -> u64 {
    #[cfg(target_os = "macos")]
    {
        // On macOS, sysctl kern.boottime returns a struct timeval
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("kern.boottime")
            .output()
        {
            let s = String::from_utf8_lossy(&output.stdout);
            // Example output: { sec = 1770452418, usec = 373454 } Sat Feb  7 09:20:18 2026
            // We'll just take the whole string as entropy for simplicity and creativity.
            let mut hasher = Sha256::new();
            hasher.update(s.as_bytes());
            let result = hasher.finalize();
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&result[..8]);
            return u64::from_le_bytes(bytes);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, use /proc/stat btime
        if let Ok(contents) = std::fs::read_to_string("/proc/stat") {
            for line in contents.lines() {
                if line.starts_with("btime ") {
                    return line[6..].trim().parse().unwrap_or(0);
                }
            }
        }
    }

    0
}

/// A "creative" shuffle to mix entropy bytes in a non-standard way.
fn creative_shuffle(data: &mut Vec<u8>) {
    if data.len() < 2 {
        return;
    }
    let len = data.len();
    for i in 0..len {
        // Use the current byte as an index for the next swap
        let j = (data[i] as usize + i) % len;
        data.swap(i, j);
    }
}
