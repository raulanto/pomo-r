use clap::Parser;

#[derive(Parser, Debug, Clone, Default, PartialEq, Eq)]
#[command(
    name = "pomo",
    version,
    about = "Pomodoro de terminal en Rust con ratatui"
)]
pub struct Cli {
    /// Minutos de trabajo
    #[arg(short = 'w', long = "work", value_name = "MIN")]
    pub work: Option<u64>,

    /// Minutos de descanso corto
    #[arg(short = 's', long = "short", value_name = "MIN")]
    pub short: Option<u64>,

    /// Minutos de descanso largo
    #[arg(short = 'l', long = "long", value_name = "MIN")]
    pub long: Option<u64>,

    /// Pomodoros antes del descanso largo
    #[arg(short = 'c', long = "cycles", value_name = "N")]
    pub cycles: Option<u32>,

    /// Iniciar la siguiente fase sola
    #[arg(long = "auto-start")]
    pub auto_start: bool,

    /// Sin campana
    #[arg(long = "no-bell")]
    pub no_bell: bool,

    /// Sin sonido ambiental
    #[arg(long = "no-sound")]
    pub no_sound: bool,

    /// Volumen del sonido (0-100)
    #[arg(short = 'v', long = "volume", value_name = "0-100")]
    pub volume: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_empty() {
        let args: Vec<&str> = vec!["pomo"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.work, None);
        assert_eq!(cli.short, None);
        assert_eq!(cli.long, None);
        assert_eq!(cli.cycles, None);
        assert!(!cli.auto_start);
        assert!(!cli.no_bell);
        assert!(!cli.no_sound);
        assert_eq!(cli.volume, None);
    }

    #[test]
    fn test_cli_parse_flags() {
        let args = vec![
            "pomo",
            "-w",
            "50",
            "-s",
            "10",
            "-l",
            "20",
            "-c",
            "3",
            "--auto-start",
            "--no-bell",
            "--no-sound",
            "-v",
            "65",
        ];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.work, Some(50));
        assert_eq!(cli.short, Some(10));
        assert_eq!(cli.long, Some(20));
        assert_eq!(cli.cycles, Some(3));
        assert!(cli.auto_start);
        assert!(cli.no_bell);
        assert!(cli.no_sound);
        assert_eq!(cli.volume, Some(65));
    }
}
