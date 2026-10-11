use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore();
    }
}

pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        original_hook(panic_info);
    }));
}

pub fn register_signals() -> io::Result<Arc<AtomicBool>> {
    let term_flag = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    {
        use signal_hook::consts::signal::{SIGHUP, SIGTERM};
        use signal_hook::flag;

        flag::register(SIGTERM, Arc::clone(&term_flag))?;
        flag::register(SIGHUP, Arc::clone(&term_flag))?;
    }
    Ok(term_flag)
}

pub fn init() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if let Err(err) = execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture) {
        let _ = disable_raw_mode();
        return Err(err);
    }
    INITIALIZED.store(true, Ordering::SeqCst);
    Terminal::new(CrosstermBackend::new(stdout))
}

pub fn restore() -> io::Result<()> {
    if !INITIALIZED.swap(false, Ordering::SeqCst) {
        return Ok(());
    }
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, Show, DisableMouseCapture);
    let _ = disable_raw_mode();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_idempotent() {
        assert!(restore().is_ok());
        assert!(restore().is_ok());
    }

    #[test]
    fn test_register_signals() {
        let signals = register_signals();
        assert!(signals.is_ok());
    }
}
