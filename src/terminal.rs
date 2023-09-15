use std::io::{self, Stdout};

use ratatui::prelude::CrosstermBackend;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> anyhow::Result<Terminal> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

pub fn restore_terminal(mut terminal: Terminal) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
