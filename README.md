# amnesia (v1.2)

**amnesia** is a high-performance, privacy-focused "volatile-only" notepad. It stores your active data exclusively in RAM and ensures that sensitive content never touches your disk unless explicitly requested and encrypted.

## New in v1.2: Hardened Persistence & Markdown
- **Encrypted Persistence**: Securely save notes to `.amnesio` files using **ChaCha20-Poly1305** and **Argon2id**.
- **Markdown Preview**: Toggle styled headers and bold text with `Ctrl+P`.
- **Security Hardening**: Enforced 8-character minimum passwords for all encrypted files.

## Privacy Features

- **RAM-Only Storage**: Your active notes exist only in your computer's memory.
- **Stealth Encryption**: Scramble data in RAM with keys derived from ephemeral system state.
- **Argon2id Persistence [NEW]**: High-security encrypted saving to disk.
- **Memory Pinning**: Uses `mlock` to prevent the OS from swapping your notes to disk.
- **Anti-Forensics**: Disables core dumps (`RLIMIT_CORE`) to prevent sensitive data leakage.
- **Privacy Timers**: 
  - **TTL (Time to Live)**: Optional self-destruct timer for the entire session.
  - **Idle Timeout**: Automatically wipes and closes the app after inactivity.

## Installation

### Instant Install (macOS/Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/laticee/amnesia/master/install.sh | bash
```

## Usage

| Action | Keybinding / Command |
| :--- | :--- |
| **Toggle Markdown** | `Ctrl + P` |
| **Save Encrypted** | `Ctrl + S` |
| **Exit** | `Esc` |

```bash
# Start with default settings
amnesia

# Load an encrypted file (opens in Read-Only mode)
amnesia secret.amnesio

# Start with a 10-minute self-destruct timer
amnesia --ttl 10
```

## Configuration
- **macOS**: `~/Library/Application Support/amnesia/config.toml`
- **Linux**: `~/.config/amnesia/config.toml`

## License
Distributed under the MIT License. See `LICENSE` for more information.
