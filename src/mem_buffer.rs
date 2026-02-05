use libc::{c_void, mlock, munlock};
use zeroize::Zeroize;

/// A buffer that is pinned in RAM and zeroed on drop.
pub struct MemoryBuffer {
    data: Vec<u8>,
}

impl MemoryBuffer {
    /// Creates a new pinned memory buffer of the given size.
    pub fn new(size: usize) -> Self {
        let data = vec![0u8; size];

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

        MemoryBuffer { data }
    }

    /// Access the underlying data as a string (assuming UTF-8).
    pub fn to_string(&self) -> String {
        // Find the first null byte or end of string
        let len = self
            .data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.data.len());
        String::from_utf8_lossy(&self.data[..len]).to_string()
    }

    /// Update the content of the buffer.
    pub fn update(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(self.data.len());

        // Zero the old content first to be sure
        // NOTE: We MUST zeroize the slice, not the Vec, because Vec::zeroize() also clears the Vec (len=0).
        self.data.as_mut_slice().zeroize();

        if len > 0 {
            self.data[..len].copy_from_slice(&bytes[..len]);
        }
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        // Explicitly overwrite with zeros before unlocking.
        self.data.zeroize();

        unsafe {
            let _ = munlock(self.data.as_ptr() as *const c_void, self.data.len());
        }
    }
}
