use sha2::{Digest, Sha256};
use std::process::Command;
use zeroize::Zeroize;

static ASLR_ANCHOR: u8 = 0xAA;

pub fn derive_key() -> [u8; 32] {
    let mut entropy = Vec::new();

    if let Ok(output) = Command::new("hostname").output() {
        entropy.extend_from_slice(&output.stdout);
    }

    if let Ok(output) = Command::new("uname").arg("-a").output() {
        entropy.extend_from_slice(&output.stdout);
    }

    let boot_time = capture_boot_time();
    entropy.extend_from_slice(&boot_time.to_le_bytes());

    let aslr_addr = &ASLR_ANCHOR as *const u8 as usize;
    entropy.extend_from_slice(&aslr_addr.to_le_bytes());

    let mut startup_random = [0u8; 32];
    if getrandom::getrandom(&mut startup_random).is_err() {
        startup_random.copy_from_slice(b"AMNESIA_STEALTH_FALLBACK_RANDOM_");
    }
    entropy.extend_from_slice(&startup_random);

    creative_shuffle(&mut entropy);

    let mut hasher = Sha256::new();
    hasher.update(&entropy);
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);

    entropy.zeroize();
    startup_random.zeroize();

    key
}

fn capture_boot_time() -> u64 {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("kern.boottime")
            .output()
        {
            let s = String::from_utf8_lossy(&output.stdout);
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

fn creative_shuffle(data: &mut Vec<u8>) {
    if data.len() < 2 {
        return;
    }
    let len = data.len();
    for i in 0..len {
        let j = (data[i] as usize + i) % len;
        data.swap(i, j);
    }
}
