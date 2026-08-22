use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::app::{App, ConfirmAction, Mode};
use crate::{imdb, media::format::display_tags};

pub fn render(app: &mut App, f: &mut ratatui::Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // title bar
        Constraint::Min(1),    // list
        Constraint::Length(1), // footer
    ])
    .split(f.area());

    // Title bar
    let filter_display = if !app.filter.is_empty() {
        format!("  Filter: {}", app.filter)
    } else if matches!(app.mode, Mode::Filter) {
        "  Filter: ".into()
    } else {
        String::new()
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled("mtracker", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("  ({} items){filter_display}", app.filtered.len())),
    ]));
    f.render_widget(title, chunks[0]);

    // List
    let max_rating = app.max_rating();
    let metas = &app.metas;
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&i| {
            let item = app.repo.get_by_index(i);
            let meta = imdb::meta_for(metas, item);
            let mut spans = vec![];

            // Rating column
            if max_rating > 0 {
                let rating_str = if let Some(r) = item.rating {
                    let filled = "+".repeat(r as usize);
                    let empty = "-".repeat(max_rating.saturating_sub(r) as usize);
                    format!("{filled}{empty}")
                } else {
                    "?".repeat(max_rating as usize)
                };
                spans.push(Span::styled(
                    format!("{rating_str} "),
                    if item.rating.is_some() {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ));
            }

            // Watchlist
            if item.on_watchlist() {
                spans.push(Span::styled("WL: ", Style::default().add_modifier(Modifier::BOLD)));
            }

            // Name
            spans.push(Span::raw(&item.name));

            // Year
            if let Some(year) = item.year {
                spans
                    .push(Span::styled(format!(" ({year})"), Style::default().fg(Color::DarkGray)));
            }

            // Tags (excluding watchlist and tags that repeat a genre), followed
            // by IMDb genres in the same bracket but dimmed
            let tags: Vec<&str> = display_tags(&item.tags, meta)
                .into_iter()
                .filter(|t| *t != "watchlist")
                .map(String::as_str)
                .collect();
            let genres: Vec<&str> =
                meta.map(|m| m.genres.iter().map(String::as_str).collect()).unwrap_or_default();
            if !tags.is_empty() || !genres.is_empty() {
                let cyan = Style::default().fg(Color::Cyan);
                let dim = Style::default().fg(Color::DarkGray);
                spans.push(Span::styled(" [", cyan));
                spans.push(Span::styled(tags.join(", "), cyan));
                if !tags.is_empty() && !genres.is_empty() {
                    spans.push(Span::styled(", ", cyan));
                }
                spans.push(Span::styled(genres.join(", "), dim));
                spans.push(Span::styled("]", cyan));
            }

            // Note
            if !item.note.is_empty() {
                spans.push(Span::styled(
                    format!(": {}", item.note),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Footer
    let yellow = Style::default().fg(Color::Yellow);
    let cursor_style = yellow.add_modifier(Modifier::REVERSED);

    let footer_line = match &app.mode {
        Mode::Normal => {
            let text = if let Some(msg) = &app.message {
                msg.clone()
            } else {
                "[/]filter [a]dd [r]ate [e]dit [d]elete [w]atchlist [o]pen [q]uit".into()
            };
            Line::from(Span::raw(text))
        }
        Mode::Filter => {
            // The cursor is a character index, not a byte index
            let val = app.input.value();
            let cur = app.input.cursor();
            let before: String = val.chars().take(cur).collect();
            let mut chars = val.chars().skip(cur);
            let under = chars.next().map_or(" ".to_string(), |c| c.to_string());
            let after: String = chars.collect();
            Line::from(vec![
                Span::styled("Filter: ", yellow),
                Span::styled(before.to_string(), yellow),
                Span::styled(under, cursor_style),
                Span::styled(format!("{after}  (Enter to apply, Esc to clear)"), yellow),
            ])
        }
        Mode::Rate(input) => Line::from(vec![
            Span::styled(format!("Rating: {input}"), yellow),
            Span::styled(" ", cursor_style),
            Span::styled("  (Enter to confirm, Esc to cancel)", yellow),
        ]),
        Mode::Confirm(ConfirmAction::Delete(idx)) => {
            let name = &app.repo.get_by_index(*idx).name;
            Line::from(Span::styled(format!("Delete \"{name}\"? [y/n]"), yellow))
        }
        Mode::Open(_) => Line::from(Span::styled(
            "Open in browser: [i]mdb  [t]mdb  [l]etterboxd  (Esc to cancel)",
            yellow,
        )),
    };
    f.render_widget(Paragraph::new(footer_line), chunks[2]);
}
