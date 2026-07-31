use amnesia::config::Config;
use amnesia::stealth;
use amnesia::tui_app::Editor;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use is_terminal::IsTerminal;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Read};
use std::time::Duration;
use zeroize::Zeroize;

use crossterm::event::KeyModifiers;

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "amnesia: a volatile-only, privacy-focused cli notepad."
)]
struct Args {
    #[arg(long)]
    ttl: Option<f64>,

    #[arg(long)]
    idle: Option<f64>,

    #[arg(long)]
    encrypt: bool,
}

#[derive(Clone, Copy, Debug)]
struct SwapSnapshot {
    bytes_used: u64,
}

fn get_swap_snapshot() -> Option<SwapSnapshot> {
    #[cfg(target_os = "linux")]
    {
        let contents = std::fs::read_to_string("/proc/meminfo").ok()?;
        let mut total_kb: Option<u64> = None;
        let mut free_kb: Option<u64> = None;
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("SwapTotal:") {
                total_kb = rest.split_whitespace().next()?.parse::<u64>().ok();
            } else if let Some(rest) = line.strip_prefix("SwapFree:") {
                free_kb = rest.split_whitespace().next()?.parse::<u64>().ok();
            }
        }
        let used_kb = total_kb?.saturating_sub(free_kb?);
        Some(SwapSnapshot {
            bytes_used: used_kb.saturating_mul(1024),
        })
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("vm.swapusage")
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&output.stdout);
        let used_gb = s.split_whitespace().skip_while(|t| *t != "used").nth(1)?;

        let used_gb = used_gb.trim_end_matches('G');
        let used_gb: f64 = used_gb.parse().ok()?;
        let bytes_used = (used_gb * 1024.0 * 1024.0 * 1024.0) as u64;
        Some(SwapSnapshot { bytes_used })
    }

    #[cfg(windows)]
    {
        let mut info = PERFORMANCE_INFORMATION {
            cb: std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32,
            CommitTotal: 0,
            CommitLimit: 0,
            CommitPeak: 0,
            PhysicalTotal: 0,
            PhysicalAvailable: 0,
            SystemCache: 0,
            KernelTotal: 0,
            KernelPaged: 0,
            KernelNonpaged: 0,
            PageSize: 0,
            HandleCount: 0,
            ProcessCount: 0,
            ThreadCount: 0,
        };

        let ok = unsafe { GetPerformanceInfo(&mut info, info.cb) };
        if ok == 0 || info.PageSize == 0 {
            return None;
        }

        let commit_bytes = (info.CommitTotal as u64).saturating_mul(info.PageSize as u64);
        let physical_bytes = (info.PhysicalTotal as u64).saturating_mul(info.PageSize as u64);

        let pagefile_estimate = commit_bytes.saturating_sub(physical_bytes);
        Some(SwapSnapshot {
            bytes_used: pagefile_estimate,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::load();

    let mut piped_content = String::new();
    if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut piped_content)?;
    }

    let ttl = args.ttl.or(config.ttl);

    let idle_secs = match (args.idle, ttl) {
        (Some(i), _) => Some(i),
        (None, _) if args.idle.is_some() => None,
        (None, Some(_)) if args.ttl.is_some() => None,
        (None, _) => args.idle.or(config.idle),
    };

    let use_encryption = args.encrypt || config.stealth_encryption.unwrap_or(false);
    let encryption_key = if use_encryption {
        let key = stealth::derive_key();
        Some(key)
    } else {
        None
    };

    #[cfg(unix)]
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::setrlimit(libc::RLIMIT_CORE, &limit) != 0 {
            eprintln!("Warning: Failed to disable core dumps.");
        }
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        default_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    #[cfg(unix)]
    let backend = {
        let tty = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;
        CrosstermBackend::new(tty)
    };
    #[cfg(not(unix))]
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let mut editor = Editor::new(idle_secs, ttl, encryption_key);
    if !piped_content.is_empty() {
        editor.storage.update(&piped_content);
        piped_content.zeroize();
    }

    if let Some(mut key) = encryption_key {
        key.zeroize();
    }

    let swap_baseline = get_swap_snapshot();

    loop {
        if editor.is_timed_out() {
            break;
        }

        if let (Some(base), Some(now)) = (swap_baseline, get_swap_snapshot()) {
            if now.bytes_used > base.bytes_used {
                editor.set_status("Swap activity detected. Exiting.");
                break;
            }
        }

        terminal.draw(|f| editor.draw(f))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        editor.toggle_markdown();
                    }
                    KeyCode::Enter => editor.handle_newline(),
                    KeyCode::Char(c) => editor.handle_input(c),
                    KeyCode::Backspace => editor.delete_backspace(),
                    KeyCode::Left => editor.move_cursor(-1),
                    KeyCode::Right => editor.move_cursor(1),
                    KeyCode::Up => editor.move_cursor_lineal(-1),
                    KeyCode::Down => editor.move_cursor_lineal(1),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;

    #[cfg(unix)]
    execute!(io::stdout(), LeaveAlternateScreen)?;
    #[cfg(not(unix))]
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    println!("\r\namnesia: memory wiped. goodbye.");
    Ok(())
}
