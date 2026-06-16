use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use getrandom::getrandom;
use zeroize::Zeroizing;
use zeroize::Zeroize;

#[cfg(unix)]
use libc::{c_void, mlock, munlock};

#[cfg(windows)]
use windows_sys::Win32::System::Memory::{VirtualLock, VirtualUnlock};

pub struct MemoryBuffer {
    data: Vec<u8>,
    key: Option<Zeroizing<[u8; 32]>>,
    nonce: [u8; 12],
}

impl MemoryBuffer {
    pub fn new(size: usize, key: Option<[u8; 32]>) -> Self {
        let mut data = vec![0u8; size];

        let mut nonce = [0u8; 12];
        if getrandom(&mut nonce).is_err() {
            nonce = [0u8; 12];
        }

        lock_memory(&data);

        let key = key.map(Zeroizing::new);

        if let Some(ref key) = key {
            let mut cipher = ChaCha20::new((&**key).into(), (&nonce).into());
            cipher.apply_keystream(&mut data);
        }

        MemoryBuffer { data, key, nonce }
    }

    pub fn is_encrypted(&self) -> bool {
        self.key.is_some()
    }

    pub fn to_string(&self) -> String {
        let mut buffer = self.data.clone();

        if let Some(ref key) = self.key {
            let mut cipher = ChaCha20::new((&**key).into(), (&self.nonce).into());
            cipher.apply_keystream(&mut buffer);
        }

        let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

        let result = String::from_utf8_lossy(&buffer[..len]).to_string();
        buffer.as_mut_slice().zeroize();
        result
    }

    pub fn update(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let new_len = bytes.len();

        self.ensure_capacity(new_len);

        self.data.as_mut_slice().zeroize();

        self.data[..new_len].copy_from_slice(bytes);

        if let Some(ref key) = self.key {
            if getrandom(&mut self.nonce).is_err() {
                self.nonce = [0u8; 12];
            }

            let mut cipher = ChaCha20::new((&**key).into(), (&self.nonce).into());
            cipher.apply_keystream(&mut self.data);
        }
    }

    fn ensure_capacity(&mut self, required_size: usize) {
        if required_size <= self.data.len() {
            return;
        }

        unlock_memory(&self.data);
        self.data.as_mut_slice().zeroize();

        let grow_to = required_size.max(self.data.len() * 2);
        self.data.resize(grow_to, 0u8);

        lock_memory(&self.data);
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        self.data.as_mut_slice().zeroize();
        self.nonce.zeroize();

        unlock_memory(&self.data);
    }
}

fn lock_memory(data: &[u8]) {
    #[cfg(unix)]
    unsafe {
        let _ = mlock(data.as_ptr() as *const c_void, data.len());
    }

    #[cfg(windows)]
    unsafe {
        let _ = VirtualLock(data.as_ptr() as *const _, data.len());
    }
}

fn unlock_memory(data: &[u8]) {
    #[cfg(unix)]
    unsafe {
        let _ = munlock(data.as_ptr() as *const c_void, data.len());
    }

    #[cfg(windows)]
    unsafe {
        let _ = VirtualUnlock(data.as_ptr() as *const _, data.len());
    }
}
