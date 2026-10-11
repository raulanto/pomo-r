use std::io::{self, Write};

use crate::domain::PhaseFinished;

pub fn ring_bell() {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(b"\x07");
    let _ = stdout.flush();
}

pub fn notify_phase_change(event: &PhaseFinished, bell: bool, desktop: bool) {
    if bell {
        ring_bell();
    }

    if desktop {
        send_desktop_notification(event);
    }
}

pub fn send_desktop_notification(event: &PhaseFinished) {
    #[cfg(feature = "notify")]
    {
        let summary = format!("Fase terminada: {}", event.finished_phase.label());
        let body = format!(
            "Siguiente fase: {}. Pomodoros completados: {}",
            event.next_phase.label(),
            event.completed_pomodoros
        );

        let _ = notify_rust::Notification::new()
            .summary(&summary)
            .body(&body)
            .appname("pomo")
            .show();
    }
    #[cfg(not(feature = "notify"))]
    {
        let _ = event;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Phase;

    #[test]
    fn test_ring_bell() {
        ring_bell();
    }

    #[test]
    fn test_notify_phase_change_no_panic() {
        let event = PhaseFinished {
            finished_phase: Phase::Work,
            next_phase: Phase::ShortBreak,
            completed_pomodoros: 1,
        };
        notify_phase_change(&event, false, false);
        notify_phase_change(&event, true, false);
    }

    #[test]
    fn test_send_desktop_notification_no_panic() {
        let event = PhaseFinished {
            finished_phase: Phase::Work,
            next_phase: Phase::ShortBreak,
            completed_pomodoros: 1,
        };
        send_desktop_notification(&event);
    }
}
