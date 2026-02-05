# amnesia 

**amnesia** is a high-performance, privacy-focused "volatile-only" notepad. It stores all your data exclusively in RAM and ensures that nothing—not even a single byte—ever touches your disk.

## Privacy Features

- **RAM-Only Storage**: Your notes exist only in your computer's memory.
- **Memory Pinning**: Uses `mlock` to prevent the operating system from swapping your notes to the disk.
- **Anti-Persistence**: Explicitly overwrites memory buffers with zeros (`zeroize`) before exiting.
- **Anti-Forensics**: Disables core dumps (`RLIMIT_CORE`) to prevent sensitive data from leaking to disk during a crash.
- **Privacy Timers**: 
  - **TTL (Time to Live)**: Optional self-destruct timer for the entire session.
  - **Idle Timeout**: Automatically wipes and closes the app after a period of inactivity.

## Installation

### Instant Install (macOS/Linux)

Install the latest version directly via `curl`:

```bash
curl -fsSL https://raw.githubusercontent.com/laticee/amnesia/main/install.sh | bash
```

### From Source

Ensure you have [Rust](https://rustup.rs/) installed, then run:

```bash
cargo install --path .
```

## Usage

```bash
# Start amnesia with a 5-minute idle timeout (default)
amnesia

# Start with a 10-minute self-destruct timer
amnesia --ttl 10

# Disable activity timeout and set a 1-minute TTL
amnesia --ttl 1

# Specify a custom idle timeout (in seconds)
amnesia --idle 60
```

## License

Distributed under the MIT License cuz why not. See `LICENSE` for more information.
