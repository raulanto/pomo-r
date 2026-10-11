use std::sync::atomic::Ordering;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::Paragraph;

#[allow(dead_code)]
mod app;
#[allow(dead_code)]
mod cli;
#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod domain;
#[allow(dead_code)]
mod input;
#[allow(dead_code)]
mod notify;
mod terminal;
#[allow(dead_code)]
mod ui;

fn main() -> anyhow::Result<()> {
    terminal::install_panic_hook();
    let signals = terminal::register_signals()?;
    let _guard = terminal::TerminalGuard;
    let mut term = terminal::init()?;

    let tick_rate = Duration::from_millis(200);

    while !signals.load(Ordering::Relaxed) {
        term.draw(|frame| {
            let area = frame.area();
            let text = "pomo (fase 2) — Presiona 'q', 'Esc', Ctrl+C o Ctrl+D para salir";
            frame.render_widget(Paragraph::new(text), area);
        })?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), m) if m.is_empty() => break,
                        (KeyCode::Esc, _) => break,
                        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => break,
                        (KeyCode::Char('d'), m) if m.contains(KeyModifiers::CONTROL) => break,
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    // El redibujado se ejecuta en la siguiente iteración
                }
                _ => {}
            }
        }
    }

    terminal::restore()?;
    Ok(())
}
