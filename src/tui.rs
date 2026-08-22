use std::{io, path::PathBuf, time::Duration};

use anyhow::Result;
use clap::ArgMatches;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;

use crate::{imdb, media::repo::Repo};

mod app;
mod input;
mod render;

use app::{App, Mode};

pub fn run(matches: &ArgMatches) -> Result<()> {
    let path = PathBuf::from(matches.get_one::<String>("DB").unwrap());
    let repo = Repo::new(&path)?;
    let metas = imdb::load_meta()?;

    // The search catalog is large; load it while the TUI is already up
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tx.send(imdb::load_catalog()).ok();
    });
    let mut app = App::new(repo, metas, rx);

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Restore the terminal even if we panic, otherwise the shell is left in
    // raw mode on the alternate screen and the panic message is invisible
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        terminal::disable_raw_mode().ok();
        crossterm::execute!(io::stdout(), LeaveAlternateScreen).ok();
        default_hook(info);
    }));

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
        app.poll_catalog()?;
        terminal.draw(|f| render::render(app, f))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    if !matches!(app.mode, Mode::Normal) {
                        app.mode = Mode::Normal;
                        continue;
                    }
                    if !app.filter.is_empty() {
                        app.set_filter(String::new());
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
