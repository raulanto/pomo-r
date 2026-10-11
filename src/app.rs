use std::time::{Duration, Instant};

use crate::config::Config;
use crate::domain::{PhaseFinished, Timer};
use crate::input::Action;

pub struct App {
    pub timer: Timer,
    pub show_help: bool,
    pub bell: bool,
    pub should_quit: bool,
    pub message: Option<(String, Instant)>,
    pub auto_start_next: bool,
    pub desktop_notifications: bool,
    pub redraw_requested: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl App {
    pub fn new(config: Config) -> Self {
        let timer = Timer::new(
            Duration::from_secs(config.work_minutes * 60),
            Duration::from_secs(config.short_break_minutes * 60),
            Duration::from_secs(config.long_break_minutes * 60),
            config.long_break_every,
        );
        Self {
            timer,
            show_help: false,
            bell: config.bell,
            should_quit: false,
            message: None,
            auto_start_next: config.auto_start_next,
            desktop_notifications: config.desktop_notifications,
            redraw_requested: false,
        }
    }

    pub fn set_message(&mut self, text: impl Into<String>) {
        self.message = Some((text.into(), Instant::now()));
    }

    #[allow(dead_code)]
    pub fn set_message_at(&mut self, text: impl Into<String>, at: Instant) {
        self.message = Some((text.into(), at));
    }

    pub fn handle(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::CloseHelpOrQuit => {
                if self.show_help {
                    self.show_help = false;
                } else {
                    self.should_quit = true;
                }
            }
            Action::ToggleTimer => self.timer.toggle(),
            Action::Reset => self.timer.reset(),
            Action::Skip => self.timer.skip(),
            Action::Increase => self.timer.adjust(1),
            Action::Decrease => self.timer.adjust(-1),
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::SelectPhase(phase) => self.timer.select_phase(phase),
            Action::Redraw => self.redraw_requested = true,
            Action::ToggleBell => {
                self.bell = !self.bell;
                let status = if self.bell { "activada" } else { "desactivada" };
                self.set_message(format!("Campana {}", status));
            }
        }
    }

    pub fn on_tick(&mut self, now: Instant) -> Option<PhaseFinished> {
        if let Some((_, created_at)) = self.message {
            if now.saturating_duration_since(created_at) >= Duration::from_secs(3) {
                self.message = None;
            }
        }

        let finished = self.timer.tick(now);
        if let Some(finished_event) = finished {
            crate::notify::notify_phase_change(
                &finished_event,
                self.bell,
                self.desktop_notifications,
            );

            let msg = format!(
                "¡{} terminada! Siguiente: {}",
                finished_event.finished_phase.label(),
                finished_event.next_phase.label()
            );
            self.set_message(msg);

            if self.auto_start_next {
                self.timer.start();
            }
            return Some(finished_event);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Phase, Status};

    #[test]
    fn test_handle_quit() {
        let mut app = App::default();
        assert!(!app.should_quit);
        app.handle(Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_close_help_or_quit() {
        let mut app = App {
            show_help: true,
            ..App::default()
        };

        // When help is open, Esc closes help
        app.handle(Action::CloseHelpOrQuit);
        assert!(!app.show_help);
        assert!(!app.should_quit);

        // When help is closed, Esc quits
        app.handle(Action::CloseHelpOrQuit);
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_timer_controls() {
        let mut app = App::default();
        assert_eq!(app.timer.status(), Status::Idle);

        app.handle(Action::ToggleTimer);
        assert_eq!(app.timer.status(), Status::Running);

        app.handle(Action::ToggleTimer);
        assert_eq!(app.timer.status(), Status::Paused);

        app.handle(Action::Reset);
        assert_eq!(app.timer.status(), Status::Idle);
    }

    #[test]
    fn test_handle_skip_and_select_phase() {
        let mut app = App::default();
        assert_eq!(app.timer.phase(), Phase::Work);

        app.handle(Action::Skip);
        assert_eq!(app.timer.phase(), Phase::ShortBreak);

        app.handle(Action::SelectPhase(Phase::LongBreak));
        assert_eq!(app.timer.phase(), Phase::LongBreak);
    }

    #[test]
    fn test_handle_adjust() {
        let mut app = App::default();
        let initial = app.timer.remaining();

        app.handle(Action::Increase);
        assert_eq!(app.timer.remaining(), initial + Duration::from_secs(60));

        app.handle(Action::Decrease);
        assert_eq!(app.timer.remaining(), initial);
    }

    #[test]
    fn test_handle_toggle_bell() {
        let mut app = App::default();
        assert!(app.bell);

        app.handle(Action::ToggleBell);
        assert!(!app.bell);
        assert!(app.message.is_some());

        app.handle(Action::ToggleBell);
        assert!(app.bell);
    }

    #[test]
    fn test_on_tick_expires_message() {
        let mut app = App::default();
        let t0 = Instant::now();
        app.set_message_at("Test aviso", t0);
        assert!(app.message.is_some());

        // Under 3 seconds: remains active
        app.on_tick(t0 + Duration::from_secs(2));
        assert!(app.message.is_some());

        // At or after 3 seconds: message is cleared
        app.on_tick(t0 + Duration::from_secs(3));
        assert!(app.message.is_none());
    }

    #[test]
    fn test_on_tick_finishes_phase_auto_start_next() {
        let mut app = App {
            auto_start_next: true,
            ..App::default()
        };
        app.handle(Action::ToggleTimer);

        let t0 = Instant::now();
        app.on_tick(t0);

        let t1 = t0 + app.timer.remaining();
        let event = app.on_tick(t1);

        assert!(event.is_some());
        assert_eq!(app.timer.phase(), Phase::ShortBreak);
        // Because auto_start_next is true, status should be Running!
        assert_eq!(app.timer.status(), Status::Running);
        assert!(app.message.is_some());
    }
}
