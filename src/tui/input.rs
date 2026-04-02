use std::io;

use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;
use tui_input::backend::crossterm::EventHandler;

use crate::media::Media;

use super::app::{App, ConfirmAction, Mode};

pub fn handle_key(
    app: &mut App,
    key: KeyEvent,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    match &app.mode {
        Mode::Normal => handle_normal(app, key, terminal),
        Mode::Filter => handle_filter(app, key),
        Mode::Rate(_) => handle_rate(app, key),
        Mode::Confirm(_) => handle_confirm(app, key),
    }
}

fn handle_normal(
    app: &mut App,
    key: KeyEvent,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    app.message = None;
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('d') if ctrl => {
            if !app.filtered.is_empty() {
                let half = terminal.size()?.height as usize / 2;
                let max = app.filtered.len() - 1;
                let target = (app.selected + half).min(max);
                let offset = app.list_state.offset();
                *app.list_state.offset_mut() = (offset + half).min(max);
                app.select(target);
            }
        }
        KeyCode::Char('u') if ctrl => {
            let half = terminal.size()?.height as usize / 2;
            let offset = app.list_state.offset();
            *app.list_state.offset_mut() = offset.saturating_sub(half);
            app.select(app.selected.saturating_sub(half));
        }
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Esc => {
            if !app.filter.is_empty() {
                app.filter.clear();
                app.apply_filter();
            } else {
                app.quit = true;
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.filtered.is_empty() && app.selected < app.filtered.len() - 1 {
                app.select(app.selected + 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected > 0 {
                app.select(app.selected - 1);
            }
        }
        KeyCode::Char('g') | KeyCode::Home => app.select(0),
        KeyCode::Char('G') | KeyCode::End => {
            if !app.filtered.is_empty() {
                app.select(app.filtered.len() - 1);
            }
        }
        KeyCode::Char('/') => {
            app.input = tui_input::Input::new(app.filter.clone());
            app.mode = Mode::Filter;
            app.message = None;
        }
        KeyCode::Char('w') => action_toggle_watchlist(app)?,
        KeyCode::Char('r') => {
            if app.selected_item().is_some() {
                app.mode = Mode::Rate(String::new());
                app.message = None;
            }
        }
        KeyCode::Char('a') => {
            action_add(app, terminal)?;
        }
        KeyCode::Char('d') => {
            if let Some(idx) = app.selected_repo_index() {
                app.mode = Mode::Confirm(ConfirmAction::Delete(idx));
                app.message = None;
            }
        }
        KeyCode::Char('e') => action_edit(app, terminal)?,
        _ => {}
    }
    Ok(())
}

fn is_enter(key: &KeyEvent) -> bool {
    key.code == KeyCode::Enter
        || (key.code == KeyCode::Char('j') && key.modifiers.contains(KeyModifiers::CONTROL))
}

fn handle_filter(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        _ if is_enter(&key) => {
            app.filter = app.input.value().to_string();
            app.mode = Mode::Normal;
        }
        KeyCode::Esc => {
            app.filter.clear();
            app.apply_filter();
            app.mode = Mode::Normal;
        }
        _ => {
            app.input.handle_event(&Event::Key(key));
            app.filter = app.input.value().to_string();
            app.apply_filter();
        }
    }
    Ok(())
}

fn handle_rate(app: &mut App, key: KeyEvent) -> Result<()> {
    let input = match &app.mode {
        Mode::Rate(s) => s.clone(),
        _ => unreachable!(),
    };

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        _ if is_enter(&key) => {
            if let Some(idx) = app.selected_repo_index() {
                let item = app.repo.get_by_index_mut(idx);
                let name = item.name.clone();
                if input.is_empty() {
                    item.rating = None;
                    app.repo.write()?;
                    app.apply_filter();
                    app.message = Some(format!("Unrated {name}"));
                } else if let Ok(rating) = input.parse::<u8>() {
                    item.rating = Some(rating);
                    let _ = item.remove_tag("watchlist");
                    app.repo.write()?;
                    app.apply_filter();
                    app.message = Some(format!("Rated {name}: {rating}"));
                } else {
                    app.message = Some("Invalid rating".into());
                }
            }
            app.mode = Mode::Normal;
        }
        KeyCode::Backspace => {
            let mut s = input;
            s.pop();
            app.mode = Mode::Rate(s);
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let mut s = input;
            s.push(c);
            app.mode = Mode::Rate(s);
        }
        _ => {}
    }
    Ok(())
}

fn handle_confirm(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('d') => {
            let action = std::mem::replace(&mut app.mode, Mode::Normal);
            if let Mode::Confirm(ConfirmAction::Delete(idx)) = action {
                let name = app.repo.get_by_index(idx).name.clone();
                app.repo.remove_by_index(idx);
                app.repo.write()?;
                app.apply_filter();
                app.message = Some(format!("Deleted {name}"));
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    Ok(())
}

fn action_toggle_watchlist(app: &mut App) -> Result<()> {
    if let Some(idx) = app.selected_repo_index() {
        let item = app.repo.get_by_index_mut(idx);
        let name = item.name.clone();
        if item.has_tag("watchlist") {
            let _ = item.remove_tag("watchlist");
            app.message = Some(format!("Removed from watchlist: {name}"));
        } else {
            item.add_tag("watchlist");
            app.message = Some(format!("Added to watchlist: {name}"));
        }
        app.repo.write()?;
        app.apply_filter();
    }
    Ok(())
}

const ADD_TEMPLATE: &str = "\
# Enter the name on the first line, then fill in optional fields below.
# Delete or leave blank any fields you don't need.
# Lines starting with # are ignored.

year:
tags:
note:
";

fn action_add(
    app: &mut App,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let result = edit::edit(ADD_TEMPLATE);

    crossterm::execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    terminal.clear()?;

    match result {
        Ok(input) => {
            let cleaned: String = input
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        return false;
                    }
                    if let Some((_, value)) = trimmed.split_once(':') {
                        if value.trim().is_empty() {
                            return false;
                        }
                    }
                    true
                })
                .collect::<Vec<_>>()
                .join("\n");

            if cleaned.is_empty() {
                app.message = Some("Aborted".into());
                return Ok(());
            }

            match Media::from_db_entry(&cleaned) {
                Ok(item) => {
                    let handle = crate::media::handle::Handle {
                        name: item.name.clone(),
                        year: item.year,
                    };
                    if app.repo.get(&handle).is_some() {
                        app.message = Some(format!("Already exists: {handle}"));
                    } else {
                        let name = format!("{handle}");
                        app.repo.add(item)?;
                        app.repo.write()?;
                        app.apply_filter();
                        app.message = Some(format!("Added {name}"));
                    }
                }
                Err(e) => {
                    app.message = Some(format!("Parse error: {e}"));
                }
            }
        }
        Err(e) => {
            app.message = Some(format!("Editor error: {e}"));
        }
    }
    Ok(())
}

fn action_edit(
    app: &mut App,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    let Some(idx) = app.selected_repo_index() else {
        return Ok(());
    };
    let item = app.repo.get_by_index(idx);
    let db_entry = item.to_db_entry();

    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let result = edit::edit(&db_entry);

    crossterm::execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    terminal.clear()?;

    match result {
        Ok(edited) => {
            if edited == db_entry {
                app.message = Some("No changes".into());
                return Ok(());
            }
            match Media::from_db_entry(&edited) {
                Ok(new_item) => {
                    let name = new_item.name.clone();
                    app.repo.remove_by_index(idx);
                    app.repo.add(new_item)?;
                    app.repo.write()?;
                    app.apply_filter();
                    app.message = Some(format!("Updated {name}"));
                }
                Err(e) => {
                    app.message = Some(format!("Parse error: {e}"));
                }
            }
        }
        Err(e) => {
            app.message = Some(format!("Editor error: {e}"));
        }
    }
    Ok(())
}
