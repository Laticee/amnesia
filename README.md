# amnesia (v1.1)

**amnesia** is a high-performance, privacy-focused "volatile-only" notepad. It stores all your data exclusively in RAM and ensures that nothing—not even a single byte—ever touches your disk.

## New in v1.1: Stealth Encryption
You can now optionally encrypt your notes in RAM with a dynamic key derived from ephemeral system state (ASLR, boot time, etc.). This makes it nearly impossible to reconstruct the plaintext from a physical memory dump.

## Privacy Features

- **RAM-Only Storage**: Your notes exist only in your computer's memory.
- **Stealth Encryption [NEW]**: Optional layer to scramble data in RAM.
- **Memory Pinning**: Uses `mlock` to prevent the OS from swapping your notes to disk.
- **Anti-Persistence**: Explicitly overwrites memory buffers with zeros (`zeroize`) before exiting.
- **Anti-Forensics**: Disables core dumps (`RLIMIT_CORE`) to prevent sensitive data leakage.
- **Privacy Timers**: 
  - **TTL (Time to Live)**: Optional self-destruct timer for the entire session.
  - **Idle Timeout**: Automatically wipes and closes the app after inactivity.

## Installation

### Instant Install (macOS/Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/laticee/amnesia/master/install.sh | bash
```

## Configuration

The configuration file is now **highly editable** and automatically documented.

- **macOS**: `~/Library/Application Support/com.laticee.amnesia/config.toml`
- **Linux**: `~/.config/amnesia/config.toml`

Example `config.toml`:

```toml
# amnesia configuration file (v1.1)

# Time to live in minutes (self-destruct)
# ttl = 10.0

# Idle timeout in seconds (default: 300)
idle = 300.0

# Enable stealth memory encryption (volatile-only)
stealth_encryption = true
```

## Usage

```bash
# Start with default settings
amnesia

# Enable stealth encryption for this session
amnesia --encrypt

# Start with a 10-minute self-destruct timer
amnesia --ttl 10
```

## License
Distributed under the MIT License. See `LICENSE` for more information.
