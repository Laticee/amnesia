# amnesia (v1.0)

hey! `amnesia` is a tiny TUI notepad i built because i wanted a place to type stuff that **never** touches my hard drive. it keeps everything in locked RAM and wipes it the second you close it.

i mostly made this on my mac (macos), and honestly, i'm still figuring out this whole github thing. if you find bugs or have ideas, feel free to send them over, but bear with me if i'm a bit messy with the "issues" tab!

## how it works (the security stuff)

- **strictly volatile**: nothing is ever written to disk. not even your config (unless you make one yourself).
- **ram locking**: the editor buffer uses a pinned allocation (`mlock`) so the os doesn't accidentally swap your secrets to disk.
- **auto-wipe**: it uses `zeroize` to scrub the memory clean when you exit.
- **stealth mode**: if you use `--encrypt`, it scrambles the text in ram with a session key. it's "defense-in-depth"—basically making it harder for anyone trying to peek at your memory.
- **tripwires**: it monitors system swap and will panic/exit if it sees the os trying to swap data.

## features

- **piping!** you can do `echo "my secret" | amnesia` to open stuff directly.
- **tui editor** built with `ratatui`.
- **timeouts**: it'll close itself if you leave it idle (`--idle`) or after a set time (`--ttl`).
- **markdown preview**: hit `ctrl + p` to see how your notes look rendered.

## usage

| action | key |
| :--- | :--- |
| toggle markdown view | `ctrl + p` |
| exit | `esc` |

```bash
# just open it
amnesia

# open with a 5-minute kill timer
amnesia --ttl 5

# pipe something in
cat passwords.txt | amnesia
```

## install

if you have rust:
```bash
cargo install --path .
```

or just use the script:
```bash
curl -fsSL https://raw.githubusercontent.com/once2027/amnesia/master/install.sh | bash
```

on **windows** (powershell):
```powershell
powershell -ExecutionPolicy Bypass -Command "Invoke-WebRequest https://raw.githubusercontent.com/once2027/amnesia/master/install.ps1 -OutFile install.ps1; .\install.ps1"
```

or grab the `amnesia-windows-x86_64.zip` from the [releases](https://github.com/once2027/amnesia/releases) page and unzip it anywhere.

## config

by default, `amnesia` won't touch your disk. if you *want* to save settings like default timeouts, you can manually create a `config.toml`:

- **macos**: `~/library/application support/washedshes/amnesia/config.toml`
- **linux**: `~/.config/amnesia/config.toml`
- **windows**: `%appdata%\washedshes\amnesia\config.toml`

```toml
idle = 300.0
stealth_encryption = true
```

## license

mit. feel free to mess with it.
