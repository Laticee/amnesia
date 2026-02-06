mod config;
mod mem_buffer;
mod tui_app;

use crate::config::Config;
use crate::tui_app::Editor;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "amnesia: A volatile-only, privacy-focused CLI notepad."
)]
struct Args {
    /// Time to live in minutes (self-destruct)
    #[arg(long)]
    ttl: Option<f64>,

    /// Idle timeout in seconds
    #[arg(long)]
    idle: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::load();

    // Determine values, prioritizing CLI args over config, then hardcoded defaults.
    let ttl = args.ttl.or(config.ttl);

    let idle_secs = match (args.idle, ttl) {
        (Some(i), _) => Some(i),                       // Explicit --idle
        (None, _) if args.idle.is_some() => None, // This shouldn't happen with Option but for clarity
        (None, Some(_)) if args.ttl.is_some() => None, // --ttl provided via CLI, no --idle provided via CLI
        (None, _) => args.idle.or(config.idle),        // Use config or default
    };

    // 1. Disable core dumps to prevent RAM data from being written to disk on crash.
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::setrlimit(libc::RLIMIT_CORE, &limit) != 0 {
            eprintln!("Warning: Failed to disable core dumps.");
        }
    }

    // 2. Set up a panic hook to clean up the terminal if the app crashes.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        default_hook(panic_info);
    }));

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut editor = Editor::new(idle_secs, ttl);

    loop {
        // 1. Check for timeout BEFORE drawing or polling
        if editor.is_timed_out() {
            break;
        }

        terminal.draw(|f| editor.draw(f))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
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

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("\r\nAmnesia: Memory wiped. Goodbye.");
    Ok(())
}
