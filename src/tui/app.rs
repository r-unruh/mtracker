use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    list::matches_term,
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
                for raw_term in &terms {
                    let (negated, term) = match raw_term.strip_prefix('!') {
                        Some(t) if !t.is_empty() => (true, t),
                        _ => (false, *raw_term),
                    };
                    let matched = matches_term(item, term, max_rating);
                    if matched == negated {
                        return false;
                    }
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

fn get_weight(item: &Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}
