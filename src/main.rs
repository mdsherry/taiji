use app::run_app;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io, path::PathBuf};
use tui::{
    backend::{CrosstermBackend},
    Terminal,
};

mod line_splitter;
mod panel;
mod render;
mod grid;
use grid::*;
mod app;

#[derive(Parser)]
pub struct Args {
    #[arg(default_value_t = 5)]
    width: usize,
    #[arg(default_value_t = 5)]
    height: usize,
    #[arg(long)]
    out_file: Option<PathBuf>,
    #[arg(long)]
    in_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, args);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
