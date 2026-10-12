use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cli::Cli;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub work_minutes: u64,
    pub short_break_minutes: u64,
    pub long_break_minutes: u64,
    pub long_break_every: u32,
    pub auto_start_next: bool,
    pub bell: bool,
    pub sound: bool,
    pub volume: u8,
    pub desktop_notifications: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            long_break_every: 4,
            auto_start_next: false,
            bell: true,
            sound: true,
            volume: 80,
            desktop_notifications: false,
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "pomo")
            .map(|proj| proj.config_dir().join("config.toml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let Some(path) = Self::config_path() else {
            return Ok(Self::default());
        };

        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.sanitize();
        Ok(config)
    }

    pub fn sanitize(&mut self) {
        self.work_minutes = self.work_minutes.max(1);
        self.short_break_minutes = self.short_break_minutes.max(1);
        self.long_break_minutes = self.long_break_minutes.max(1);
        self.long_break_every = self.long_break_every.max(1);
        self.volume = self.volume.min(100);
    }

    pub fn merge(mut self, cli: Cli) -> Self {
        if let Some(w) = cli.work {
            self.work_minutes = w.max(1);
        }
        if let Some(s) = cli.short {
            self.short_break_minutes = s.max(1);
        }
        if let Some(l) = cli.long {
            self.long_break_minutes = l.max(1);
        }
        if let Some(c) = cli.cycles {
            self.long_break_every = c.max(1);
        }
        if cli.auto_start {
            self.auto_start_next = true;
        }
        if cli.no_bell {
            self.bell = false;
        }
        if cli.no_sound {
            self.sound = false;
        }
        if let Some(vol) = cli.volume {
            self.volume = vol.min(100);
        }

        self.sanitize();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.work_minutes, 25);
        assert_eq!(config.short_break_minutes, 5);
        assert_eq!(config.long_break_minutes, 15);
        assert_eq!(config.long_break_every, 4);
        assert!(!config.auto_start_next);
        assert!(config.bell);
        assert!(config.sound);
        assert_eq!(config.volume, 80);
        assert!(!config.desktop_notifications);
    }

    #[test]
    fn test_merge_with_cli() {
        let config = Config::default();
        let cli = Cli {
            work: Some(50),
            short: Some(10),
            long: Some(30),
            cycles: Some(2),
            auto_start: true,
            no_bell: true,
            no_sound: true,
            volume: Some(60),
        };

        let merged = config.merge(cli);
        assert_eq!(merged.work_minutes, 50);
        assert_eq!(merged.short_break_minutes, 10);
        assert_eq!(merged.long_break_minutes, 30);
        assert_eq!(merged.long_break_every, 2);
        assert!(merged.auto_start_next);
        assert!(!merged.bell);
        assert!(!merged.sound);
        assert_eq!(merged.volume, 60);
    }

    #[test]
    fn test_merge_enforces_minimum_and_caps_volume() {
        let config = Config::default();
        let cli = Cli {
            work: Some(0),
            short: Some(0),
            long: Some(0),
            cycles: Some(0),
            auto_start: false,
            no_bell: false,
            no_sound: false,
            volume: Some(150),
        };

        let merged = config.merge(cli);
        assert_eq!(merged.work_minutes, 1);
        assert_eq!(merged.short_break_minutes, 1);
        assert_eq!(merged.long_break_minutes, 1);
        assert_eq!(merged.long_break_every, 1);
        assert_eq!(merged.volume, 100);
    }

    #[test]
    fn test_load_non_existent_file() {
        let path = Path::new("/path/that/definitely/does/not/exist/pomo_config.toml");
        let result = Config::load_from_path(path).unwrap();
        assert_eq!(result, Config::default());
    }

    #[test]
    fn test_deserialize_partial_toml() {
        let toml_str = r#"
            work_minutes = 45
            bell = false
            sound = false
            volume = 40
        "#;
        let mut config: Config = toml::from_str(toml_str).unwrap();
        config.sanitize();

        assert_eq!(config.work_minutes, 45);
        assert_eq!(config.short_break_minutes, 5); // default
        assert_eq!(config.long_break_minutes, 15); // default
        assert_eq!(config.long_break_every, 4); // default
        assert!(!config.bell);
        assert!(!config.sound);
        assert_eq!(config.volume, 40);
        assert!(!config.auto_start_next);
    }
}
