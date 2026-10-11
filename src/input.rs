use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::domain::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    CloseHelpOrQuit,
    ToggleTimer,
    Reset,
    Skip,
    Increase,
    Decrease,
    ToggleHelp,
    SelectPhase(Phase),
    Redraw,
    ToggleBell,
}

pub fn map_key(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match (key.code, key.modifiers) {
        // Salida segura
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char('C'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char('d'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char('D'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char('q'), m) if m.is_empty() => Some(Action::Quit),
        (KeyCode::Esc, _) => Some(Action::CloseHelpOrQuit),

        // Control del temporizador
        (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => Some(Action::ToggleTimer),
        (KeyCode::Char('r'), m) if m.is_empty() => Some(Action::Reset),
        (KeyCode::Char('r'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Reset),
        (KeyCode::Char('R'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Reset),

        // Fases y navegación
        (KeyCode::Char('n'), m) if m.is_empty() => Some(Action::Skip),
        (KeyCode::Char('s'), m) if m.is_empty() => Some(Action::Skip),
        (KeyCode::Right, _) => Some(Action::Skip),

        // Ajuste de tiempo
        (KeyCode::Char('+'), _) | (KeyCode::Up, _) => Some(Action::Increase),
        (KeyCode::Char('-'), _) | (KeyCode::Down, _) => Some(Action::Decrease),

        // Selección directa de fase
        (KeyCode::Char('1'), _) => Some(Action::SelectPhase(Phase::Work)),
        (KeyCode::Char('2'), _) => Some(Action::SelectPhase(Phase::ShortBreak)),
        (KeyCode::Char('3'), _) => Some(Action::SelectPhase(Phase::LongBreak)),

        // Utilidades y ayuda
        (KeyCode::Char('b'), m) if m.is_empty() => Some(Action::ToggleBell),
        (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) | (KeyCode::F(1), _) => {
            Some(Action::ToggleHelp)
        }
        (KeyCode::Char('l'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Redraw),
        (KeyCode::Char('L'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Redraw),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_quit_shortcuts() {
        assert_eq!(
            map_key(press(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('d'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(Action::Quit)
        );
        assert_eq!(
            map_key(press(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Action::CloseHelpOrQuit)
        );
    }

    #[test]
    fn test_timer_control_shortcuts() {
        assert_eq!(
            map_key(press(KeyCode::Char(' '), KeyModifiers::NONE)),
            Some(Action::ToggleTimer)
        );
        assert_eq!(
            map_key(press(KeyCode::Enter, KeyModifiers::NONE)),
            Some(Action::ToggleTimer)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('r'), KeyModifiers::NONE)),
            Some(Action::Reset)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('r'), KeyModifiers::CONTROL)),
            Some(Action::Reset)
        );
    }

    #[test]
    fn test_skip_shortcuts() {
        assert_eq!(
            map_key(press(KeyCode::Char('n'), KeyModifiers::NONE)),
            Some(Action::Skip)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('s'), KeyModifiers::NONE)),
            Some(Action::Skip)
        );
        assert_eq!(
            map_key(press(KeyCode::Right, KeyModifiers::NONE)),
            Some(Action::Skip)
        );
    }

    #[test]
    fn test_adjust_shortcuts() {
        assert_eq!(
            map_key(press(KeyCode::Char('+'), KeyModifiers::NONE)),
            Some(Action::Increase)
        );
        assert_eq!(
            map_key(press(KeyCode::Up, KeyModifiers::NONE)),
            Some(Action::Increase)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('-'), KeyModifiers::NONE)),
            Some(Action::Decrease)
        );
        assert_eq!(
            map_key(press(KeyCode::Down, KeyModifiers::NONE)),
            Some(Action::Decrease)
        );
    }

    #[test]
    fn test_select_phase_shortcuts() {
        assert_eq!(
            map_key(press(KeyCode::Char('1'), KeyModifiers::NONE)),
            Some(Action::SelectPhase(Phase::Work))
        );
        assert_eq!(
            map_key(press(KeyCode::Char('2'), KeyModifiers::NONE)),
            Some(Action::SelectPhase(Phase::ShortBreak))
        );
        assert_eq!(
            map_key(press(KeyCode::Char('3'), KeyModifiers::NONE)),
            Some(Action::SelectPhase(Phase::LongBreak))
        );
    }

    #[test]
    fn test_utility_and_help_shortcuts() {
        assert_eq!(
            map_key(press(KeyCode::Char('b'), KeyModifiers::NONE)),
            Some(Action::ToggleBell)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('?'), KeyModifiers::NONE)),
            Some(Action::ToggleHelp)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('h'), KeyModifiers::NONE)),
            Some(Action::ToggleHelp)
        );
        assert_eq!(
            map_key(press(KeyCode::F(1), KeyModifiers::NONE)),
            Some(Action::ToggleHelp)
        );
        assert_eq!(
            map_key(press(KeyCode::Char('l'), KeyModifiers::CONTROL)),
            Some(Action::Redraw)
        );
    }

    #[test]
    fn test_release_and_repeat_are_ignored() {
        let mut key = press(KeyCode::Char('q'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert_eq!(map_key(key), None);

        key.kind = KeyEventKind::Repeat;
        assert_eq!(map_key(key), None);
    }
}
