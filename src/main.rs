use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event};

mod app;
mod cli;
mod config;
mod domain;
mod input;
mod notify;
mod terminal;
mod ui;

use app::App;
use cli::Cli;
use config::Config;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?.merge(cli);

    terminal::install_panic_hook();
    let signals = terminal::register_signals()?;
    let _guard = terminal::TerminalGuard;
    let mut term = terminal::init()?;

    let mut app = App::new(config);
    let tick_rate = Duration::from_millis(200);

    while !app.should_quit && !signals.load(Ordering::Relaxed) {
        if app.redraw_requested {
            app.redraw_requested = false;
            term.clear()?;
        }

        term.draw(|frame| ui::render(frame, &app))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = input::map_key(key) {
                        app.handle(action);
                    }
                }
                Event::Resize(_, _) => {
                    // El redibujado se ejecuta automáticamente en la siguiente iteración
                }
                _ => {}
            }
        }

        let _ = app.on_tick(Instant::now());
    }

    terminal::restore()?;
    Ok(())
}
