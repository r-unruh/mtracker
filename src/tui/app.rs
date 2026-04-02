use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    list::try_parse_year_range,
    media::{repo::Repo, Media},
};

pub enum Mode {
    Normal,
    Filter,
    Rate(String),
    Confirm(ConfirmAction),
}

pub enum ConfirmAction {
    Delete(usize),
}

pub struct App {
    pub repo: Repo,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub list_state: ListState,
    pub input: Input,
    pub filter: String,
    pub mode: Mode,
    pub message: Option<String>,
    pub quit: bool,
}

impl App {
    pub fn new(repo: Repo) -> Self {
        let mut app = App {
            repo,
            filtered: vec![],
            selected: 0,
            list_state: ListState::default(),
            input: Input::default(),
            filter: String::new(),
            mode: Mode::Normal,
            message: None,
            quit: false,
        };
        app.apply_filter();
        app
    }

    pub fn apply_filter(&mut self) {
        let terms: Vec<&str> = self.filter.split_whitespace().collect();
        let max_rating = (0..self.repo.len())
            .filter_map(|i| self.repo.get_by_index(i).rating)
            .max()
            .unwrap_or(0);

        self.filtered = (0..self.repo.len())
            .filter(|&i| {
                let item = self.repo.get_by_index(i);
                if terms.is_empty() {
                    return true;
                }
                for term in &terms {
                    // Year range
                    if let Some(range) = try_parse_year_range(term) {
                        match item.year {
                            Some(y) if y >= range.0 && y <= range.1 => continue,
                            _ => return false,
                        }
                    }
                    // Special terms
                    if *term == "rated" {
                        if item.rating.is_some() {
                            continue;
                        }
                        return false;
                    }
                    if *term == "unrated" {
                        if item.rating.is_none() {
                            continue;
                        }
                        return false;
                    }
                    // Rating pattern (e.g. ++, ++--, ---)
                    if let Some(m) = try_match_rating(term, item, max_rating) {
                        if m {
                            continue;
                        }
                        return false;
                    }
                    // Tag match
                    if item.has_tag(term) {
                        continue;
                    }
                    // Name substring match (case-insensitive)
                    if item.name.to_lowercase().contains(&term.to_lowercase()) {
                        continue;
                    }
                    return false;
                }
                true
            })
            .collect();

        // Sort: watchlist first, then rating desc, then alphabetical
        self.filtered.sort_by(|&a, &b| {
            let ia = self.repo.get_by_index(a);
            let ib = self.repo.get_by_index(b);
            let wa = get_weight(ia);
            let wb = get_weight(ib);
            if wa == wb {
                ia.name.to_lowercase().cmp(&ib.name.to_lowercase())
            } else {
                wb.cmp(&wa)
            }
        });

        // Clamp selection
        if self.filtered.is_empty() {
            self.selected = 0;
            self.list_state.select(None);
        } else {
            if self.selected >= self.filtered.len() {
                self.selected = self.filtered.len() - 1;
            }
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn select(&mut self, idx: usize) {
        self.selected = idx;
        self.list_state.select(Some(idx));
    }

    pub fn selected_repo_index(&self) -> Option<usize> {
        self.filtered.get(self.selected).copied()
    }

    pub fn selected_item(&self) -> Option<&Media> {
        self.selected_repo_index().map(|i| self.repo.get_by_index(i))
    }

    pub fn max_rating(&self) -> u8 {
        (0..self.repo.len())
            .filter_map(|i| self.repo.get_by_index(i).rating)
            .max()
            .unwrap_or(0)
    }
}

fn try_match_rating(term: &str, item: &Media, max_rating: u8) -> Option<bool> {
    if term.is_empty() || !term.chars().all(|c| c == '+' || c == '-') {
        return None;
    }
    let pluses = term.chars().filter(|&c| c == '+').count() as u8;
    let minuses = term.chars().filter(|&c| c == '-').count() as u8;
    let rating = match item.rating {
        Some(r) => r,
        None => return Some(false),
    };
    if pluses > 0 && minuses > 0 {
        Some(rating == pluses)
    } else if minuses > 0 {
        Some(rating <= max_rating.saturating_sub(minuses))
    } else {
        Some(rating >= pluses)
    }
}

fn get_weight(item: &Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}
