use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use libc::{c_void, mlock, munlock};
use zeroize::Zeroize;

/// A buffer that is pinned in RAM and zeroed on drop.
/// Optionally encrypted with an ephemeral key.
pub struct MemoryBuffer {
    data: Vec<u8>,
    key: Option<[u8; 32]>,
}

impl MemoryBuffer {
    /// Creates a new pinned memory buffer of the given size.
    pub fn new(size: usize, key: Option<[u8; 32]>) -> Self {
        let mut data = vec![0u8; size];

        // Pin the memory to prevent swapping.
        unsafe {
            let res = mlock(data.as_ptr() as *const c_void, size);
            if res != 0 {
                eprintln!(
                    "Warning: Failed to lock memory in RAM. mlock returned {}",
                    res
                );
            }
        }

        if let Some(mut k) = key {
            let mut cipher = ChaCha20::new(&k.into(), &[0u8; 12].into());
            cipher.apply_keystream(&mut data);
            k.as_mut_slice().zeroize();
        }

        MemoryBuffer { data, key }
    }

    /// Returns true if the buffer is currently encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.key.is_some()
    }

    /// Access the underlying data as a string (assuming UTF-8).
    pub fn to_string(&self) -> String {
        let mut buffer = self.data.clone();

        if let Some(mut key) = self.key {
            let mut cipher = ChaCha20::new(&key.into(), &[0u8; 12].into());
            cipher.apply_keystream(&mut buffer);
            key.as_mut_slice().zeroize();
        }

        // Find the first null byte or end of string
        let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

        let result = String::from_utf8_lossy(&buffer[..len]).to_string();
        buffer.as_mut_slice().zeroize();
        result
    }

    /// Update the content of the buffer.
    pub fn update(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let new_len = bytes.len();

        // 1. Ensure capacity (scalable!)
        self.ensure_capacity(new_len);

        // 2. Clear old content (preserving the rest of the buffer)
        self.data.as_mut_slice().zeroize();

        // 3. Copy new content
        self.data[..new_len].copy_from_slice(bytes);

        // 4. Always encrypt the entire buffer to maintain consistency
        if let Some(mut key) = self.key {
            let mut cipher = ChaCha20::new(&key.into(), &[0u8; 12].into());
            cipher.apply_keystream(&mut self.data);
            key.as_mut_slice().zeroize();
        }
    }

    fn ensure_capacity(&mut self, required_size: usize) {
        if required_size <= self.data.len() {
            return;
        }

        // We need to grow. To be safe with mlock, we'll:
        // 1. Unlock and zero current memory
        unsafe {
            let _ = munlock(self.data.as_ptr() as *const c_void, self.data.len());
        }
        self.data.as_mut_slice().zeroize();

        // 2. Resize (we'll grow to required_size or double the current size, whichever is larger)
        let grow_to = required_size.max(self.data.len() * 2);
        self.data.resize(grow_to, 0u8);

        // 3. Pin the new memory
        unsafe {
            let res = mlock(self.data.as_ptr() as *const c_void, self.data.len());
            if res != 0 {
                eprintln!("Warning: Failed to lock NEW memory in RAM ({}).", res);
            }
        }
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        // Explicitly overwrite with zeros before unlocking.
        self.data.as_mut_slice().zeroize();

        if let Some(mut key) = self.key {
            key.as_mut_slice().zeroize();
        }

        unsafe {
            let _ = munlock(self.data.as_ptr() as *const c_void, self.data.len());
        }
    }
}
