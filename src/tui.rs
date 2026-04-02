use std::{io, path::PathBuf, time::Duration};

use anyhow::Result;
use clap::ArgMatches;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;

use crate::media::repo::Repo;

mod app;
mod input;
mod render;

use app::{App, Mode};

pub fn run(matches: &ArgMatches) -> Result<()> {
    let path = PathBuf::from(matches.get_one::<String>("DB").unwrap());
    let repo = Repo::new(&path)?;
    let mut app = App::new(repo);

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = main_loop(&mut app, &mut terminal);

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn main_loop(
    app: &mut App,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    loop {
        terminal.draw(|f| render::render(app, f))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    if !matches!(app.mode, Mode::Normal) {
                        app.mode = Mode::Normal;
                        continue;
                    }
                    if !app.filter.is_empty() {
                        app.filter.clear();
                        app.apply_filter();
                        continue;
                    }
                    break;
                }
                input::handle_key(app, key, terminal)?;
            }
        }

        if app.quit {
            break;
        }
    }
    Ok(())
}
