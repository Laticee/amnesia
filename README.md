# amnesia (v1.0)

hey! `amnesia` is a tiny TUI notepad i built because i wanted a place to type stuff that **never** touches my hard drive. it keeps everything in locked RAM and wipes it the second you close it.

i mostly made this on my mac (macOS), and honestly, i'm still figuring out this whole github thing. if you find bugs or have ideas, feel free to send them over, but bear with me if i'm a bit messy with the "issues" tab!

## How it works (The Security Stuff)

- **Strictly Volatile**: nothing is ever written to disk. not even your config (unless you make one yourself).
- **RAM Locking**: the editor buffer uses a pinned allocation (`mlock`) so the OS doesn't accidentally swap your secrets to disk.
- **Auto-Wipe**: it uses `zeroize` to scrub the memory clean when you exit.
- **Stealth Mode**: if you use `--encrypt`, it scrambles the text in RAM with a session key. it's "defense-in-depth"—basically making it harder for anyone trying to peek at your memory.
- **Tripwires**: it monitors system swap and will panic/exit if it sees the OS trying to swap data.

## Features

- **Piping!** you can do `echo "my secret" | amnesia` to open stuff directly.
- **TUI editor** built with `ratatui`.
- **Timeouts**: it'll close itself if you leave it idle (`--idle`) or after a set time (`--ttl`).
- **Markdown preview**: hit `Ctrl + P` to see how your notes look rendered.

## Usage

| Action | Key |
| :--- | :--- |
| Toggle Markdown view | `Ctrl + P` |
| Exit | `Esc` |

```bash
# just open it
amnesia

# open with a 5-minute kill timer
amnesia --ttl 5

# pipe something in
cat passwords.txt | amnesia
```

## Install

If you have Rust:
```bash
cargo install --path .
```

Or just use the script:
```bash
curl -fsSL https://raw.githubusercontent.com/laticee/amnesia/master/install.sh | bash
```

## Config

By default, `amnesia` won't touch your disk. If you *want* to save settings like default timeouts, you can manually create a `config.toml`:

- **macOS**: `~/Library/Application Support/amnesia/config.toml`
- **Linux**: `~/.config/amnesia/config.toml`

```toml
idle = 300.0
stealth_encryption = true
```

## License

MIT. feel free to mess with it.
