mod config;
mod homeassistant;
mod state;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "emon")]
#[command(about = "Terminal UI for real-time energy monitoring from Home Assistant")]
#[command(
    version = "0.1.0 beta",
    disable_version_flag = true,
    disable_help_flag = true
)]
#[command(
    after_help = "EMON - Terminal UI for real-time energy monitoring from Home Assistant\n\nEMON"
)]
struct Args {
    #[arg(short, long, help = "Path to custom config file")]
    config: Option<PathBuf>,

    #[arg(short = 'h', long, action = clap::ArgAction::Help, help = "Print help information")]
    help: (),

    #[arg(short = 'v', long, action = clap::ArgAction::Version, help = "Print version information")]
    version: (),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::load_config_at(args.config.as_deref())?;
    let config_path = args.config.clone();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = state::AppState::new(config, config_path);

    // Fetch data immediately on startup
    let _ = app.update().await;

    // Determine fetch interval from config
    // 0 = realtime (~100ms), None = default 5s, Some(n) = n seconds
    let interval_ms = match app.config.home_assistant.fetch_interval_seconds {
        Some(0) => 100,      // Realtime mode
        Some(s) => s * 1000, // User-defined seconds to milliseconds
        None => 5000,        // Default 5 seconds
    };
    let tick_rate = Duration::from_millis(interval_ms);
    let mut last_tick = std::time::Instant::now();

    // UI refresh rate for counter updates (100ms for smooth counting)
    let ui_refresh_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        // Use shorter timeout to ensure UI updates frequently for counter
        let timeout = ui_refresh_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0))
            .min(Duration::from_millis(100)); // Always refresh UI at least every 100ms

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
                // Dismiss error on any other key press
                if app.error.is_some() {
                    app.error = None;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            match app.update().await {
                Ok(_) => {
                    // Clear any previous errors on successful update
                    app.error = None;
                }
                Err(e) => {
                    // Auto-reconnect: Try again immediately on error
                    // This creates realtime reconnection loop
                    app.error = Some(e.to_string());
                }
            }
            last_tick = std::time::Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
