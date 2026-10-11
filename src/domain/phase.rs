use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Phase::Work => "Trabajo",
            Phase::ShortBreak => "Descanso corto",
            Phase::LongBreak => "Descanso largo",
        }
    }

    pub fn is_break(&self) -> bool {
        matches!(self, Phase::ShortBreak | Phase::LongBreak)
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_labels() {
        assert_eq!(Phase::Work.label(), "Trabajo");
        assert_eq!(Phase::ShortBreak.label(), "Descanso corto");
        assert_eq!(Phase::LongBreak.label(), "Descanso largo");
    }

    #[test]
    fn test_phase_is_break() {
        assert!(!Phase::Work.is_break());
        assert!(Phase::ShortBreak.is_break());
        assert!(Phase::LongBreak.is_break());
    }

    #[test]
    fn test_phase_display() {
        assert_eq!(format!("{}", Phase::Work), "Trabajo");
        assert_eq!(format!("{}", Phase::ShortBreak), "Descanso corto");
        assert_eq!(format!("{}", Phase::LongBreak), "Descanso largo");
    }
}
