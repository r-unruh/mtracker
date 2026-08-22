use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::app::{App, ConfirmAction, Mode, Row};
use crate::{
    imdb::{self, CatalogEntry, Meta},
    media::{format::display_tags, Media},
};

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
    let counts = if app.catalog_shown() > 0 {
        format!(
            "({} items, {} of {} from catalog)",
            app.item_count(),
            app.catalog_shown(),
            app.catalog_total
        )
    } else {
        format!("({} items)", app.item_count())
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled("mtracker", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("  {counts}{filter_display}")),
    ]));
    f.render_widget(title, chunks[0]);

    // List
    let max_rating = app.max_rating();
    let metas = &app.metas;
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&row| match row {
            Row::Item(i) => {
                let item = app.repo.get_by_index(i);
                item_line(item, imdb::meta_for(metas, item), max_rating)
            }
            Row::Catalog(i) => catalog_line(&app.catalog[i], max_rating),
        })
        .map(ListItem::new)
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
                "[/]filter [a]dd [r]ate [e]dit [d]elete [w]atchlist [o]pen [s]ync [q]uit".into()
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
                Span::styled(before, yellow),
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

/// A line for an item from the database
fn item_line<'a>(item: &'a Media, meta: Option<&'a Meta>, max_rating: u8) -> Line<'a> {
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
        spans.push(Span::styled(format!(" ({year})"), Style::default().fg(Color::DarkGray)));
    }

    // Tags, then genres dimmed in the same bracket
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
        spans.push(Span::styled(format!(": {}", item.note), Style::default().fg(Color::DarkGray)));
    }

    Line::from(spans)
}

/// A dimmed line for a catalog title that is not in the database
fn catalog_line(entry: &CatalogEntry, max_rating: u8) -> Line<'_> {
    let dim = Style::default().fg(Color::DarkGray);
    let m = &entry.meta;
    let mut spans = vec![];

    // Keep the columns aligned with the rating column of items
    if max_rating > 0 {
        spans.push(Span::raw(" ".repeat(max_rating as usize + 1)));
    }
    spans.push(Span::styled(&m.primary_title, dim));
    if let Some(year) = m.year {
        spans.push(Span::styled(format!(" ({year})"), dim));
    }
    if m.original_title != m.primary_title {
        spans.push(Span::styled(format!(" / {}", m.original_title), dim));
    }
    if !m.genres.is_empty() {
        spans.push(Span::styled(format!(" [{}]", m.genres.join(", ")), dim));
    }
    if m.rating.is_some() {
        spans.push(Span::styled(format!("  {}", m.rating_string()), dim));
    }
    Line::from(spans)
}
