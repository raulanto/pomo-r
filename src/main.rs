use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ratatui::widgets::Paragraph;

mod app;
#[allow(dead_code)]
mod cli;
#[allow(dead_code)]
mod config;
mod domain;
mod input;
#[allow(dead_code)]
mod notify;
mod terminal;
#[allow(dead_code)]
mod ui;

use app::App;
use domain::{Status, Timer};

fn main() -> anyhow::Result<()> {
    terminal::install_panic_hook();
    let signals = terminal::register_signals()?;
    let _guard = terminal::TerminalGuard;
    let mut term = terminal::init()?;

    let mut app = App::new(Timer::default());
    let tick_rate = Duration::from_millis(200);

    while !app.should_quit && !signals.load(Ordering::Relaxed) {
        if app.redraw_requested {
            app.redraw_requested = false;
            term.clear()?;
        }

        term.draw(|frame| {
            let area = frame.area();

            let mins = app.timer.remaining().as_secs() / 60;
            let secs = app.timer.remaining().as_secs() % 60;
            let time_str = format!("{:02}:{:02}", mins, secs);

            let status_str = match app.timer.status() {
                Status::Running => "▶ En marcha",
                Status::Paused => "⏸ Pausado",
                Status::Idle => "⏹ Inactivo",
            };

            let help_str = if app.show_help {
                "\n\n[AYUDA]\nSpace/Enter: Iniciar/Pausar | r: Reiniciar | n/s/→: Saltar | +/-/↑/↓: ±1 min\n1/2/3: Cambiar fase | b: Campana | ?: Ayuda | q/Esc/Ctrl+C: Salir"
            } else {
                ""
            };

            let msg_str = match &app.message {
                Some((msg, _)) => format!("\n[Aviso]: {}", msg),
                None => String::new(),
            };

            let text = format!(
                "pomo\n\nFase: {}  |  Tiempo: {}  |  Estado: {}  |  Pomodoros: {}{}{}\n\nAtajos: Space (pausa/inicio) · r (reiniciar) · n (saltar) · +/- (ajuste) · ? (ayuda) · q (salir)",
                app.timer.phase(),
                time_str,
                status_str,
                app.timer.completed_pomodoros(),
                msg_str,
                help_str
            );

            frame.render_widget(Paragraph::new(text), area);
        })?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = input::map_key(key) {
                        app.handle(action);
                    }
                }
                Event::Resize(_, _) => {
                    // El redibujado se ejecuta en la siguiente iteración
                }
                _ => {}
            }
        }

        let _ = app.on_tick(Instant::now());
    }

    terminal::restore()?;
    Ok(())
}
