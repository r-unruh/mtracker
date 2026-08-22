use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Receiver,
};

use anyhow::Result;
use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    imdb::{self, CatalogEntry, Meta},
    list::{matches_catalog, matches_term},
    media::{handle::Handle, repo::Repo, Media},
};

pub enum Mode {
    Normal,
    Filter,
    Rate(String),
    Confirm(ConfirmAction),
    /// Choose a website to open the row in
    Open(Row),
}

pub enum ConfirmAction {
    Delete(usize),
}

/// A list line: a db item or a catalog title not in the db
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Item(usize),
    Catalog(usize),
}

/// Catalog rows shown per filter, at most
pub const CATALOG_LIMIT: usize = 500;

pub struct App {
    pub repo: Repo,
    pub metas: HashMap<String, Meta>,
    pub catalog: Vec<CatalogEntry>,
    /// `None` once the background load has been received
    catalog_rx: Option<Receiver<Result<Vec<CatalogEntry>>>>,
    /// IMDb ids present in the database; their catalog rows are hidden
    db_ids: HashSet<String>,
    pub filtered: Vec<Row>,
    /// Catalog matches, including those beyond the limit
    pub catalog_total: usize,
    pub selected: usize,
    pub list_state: ListState,
    pub input: Input,
    pub filter: String,
    pub mode: Mode,
    pub message: Option<String>,
    pub quit: bool,
}

impl App {
    pub fn new(
        repo: Repo,
        metas: HashMap<String, Meta>,
        catalog_rx: Receiver<Result<Vec<CatalogEntry>>>,
    ) -> Self {
        let mut app = App {
            repo,
            metas,
            catalog: vec![],
            catalog_rx: Some(catalog_rx),
            db_ids: HashSet::new(),
            filtered: vec![],
            catalog_total: 0,
            selected: 0,
            list_state: ListState::default(),
            input: Input::default(),
            filter: String::new(),
            mode: Mode::Normal,
            message: None,
            quit: false,
        };
        app.refresh_db_ids();
        app.apply_filter();
        app
    }

    /// Load the catalog on a background thread
    pub fn spawn_catalog_load() -> Receiver<Result<Vec<CatalogEntry>>> {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            tx.send(imdb::load_catalog()).ok();
        });
        rx
    }

    /// Re-read the caches after a sync
    pub fn reload_caches(&mut self) -> Result<()> {
        self.metas = imdb::load_meta()?;
        self.catalog_rx = Some(Self::spawn_catalog_load());
        self.refresh_db_ids();
        self.apply_filter();
        Ok(())
    }

    /// Pick up the catalog once the background load is done
    pub fn poll_catalog(&mut self) -> Result<()> {
        let Some(rx) = &self.catalog_rx else {
            return Ok(());
        };
        if let Ok(result) = rx.try_recv() {
            self.catalog_rx = None;
            self.catalog = result?;
            if !self.filter.is_empty() {
                self.apply_filter();
            }
        }
        Ok(())
    }

    fn refresh_db_ids(&mut self) {
        self.db_ids = (0..self.repo.len())
            .filter_map(|i| self.repo.get_by_index(i).imdb.clone())
            .collect();
    }

    pub fn apply_filter(&mut self) {
        let filter = self.filter.clone();
        let terms: Vec<(bool, &str)> = filter
            .split_whitespace()
            .map(|raw| match raw.strip_prefix('!') {
                Some(t) if !t.is_empty() => (true, t),
                _ => (false, raw),
            })
            .collect();
        let max_rating = self.max_rating();

        // Database items, sorted: watchlist first, then rating desc, then alphabetical
        let mut items: Vec<usize> = (0..self.repo.len())
            .filter(|&i| {
                let item = self.repo.get_by_index(i);
                let meta = imdb::meta_for(&self.metas, item);
                terms
                    .iter()
                    .all(|&(negated, term)| matches_term(item, meta, term, max_rating) != negated)
            })
            .collect();
        items.sort_by(|&a, &b| {
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
        self.filtered = items.into_iter().map(Row::Item).collect();

        // Catalog titles below, only while filtering; already sorted by popularity
        self.catalog_total = 0;
        if !terms.is_empty() {
            for (i, entry) in self.catalog.iter().enumerate() {
                if self.db_ids.contains(&entry.meta.tconst) {
                    continue;
                }
                let hit = terms.iter().all(|&(negated, term)| {
                    matches_catalog(&entry.key, &entry.meta, term) != negated
                });
                if hit {
                    self.catalog_total += 1;
                    if self.catalog_total <= CATALOG_LIMIT {
                        self.filtered.push(Row::Catalog(i));
                    }
                }
            }
        }

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

    /// Change the filter text; a changed filter starts at the top of the list
    pub fn set_filter(&mut self, filter: String) {
        if filter == self.filter {
            return;
        }
        self.filter = filter;
        self.selected = 0;
        *self.list_state.offset_mut() = 0;
        self.apply_filter();
    }

    pub fn item_count(&self) -> usize {
        self.filtered.iter().filter(|r| matches!(r, Row::Item(_))).count()
    }

    pub fn catalog_shown(&self) -> usize {
        self.filtered.len() - self.item_count()
    }

    pub fn select(&mut self, idx: usize) {
        self.selected = idx;
        self.list_state.select(Some(idx));
    }

    pub fn selected_row(&self) -> Option<Row> {
        self.filtered.get(self.selected).copied()
    }

    pub fn selected_repo_index(&self) -> Option<usize> {
        match self.selected_row() {
            Some(Row::Item(i)) => Some(i),
            _ => None,
        }
    }

    pub fn selected_catalog_index(&self) -> Option<usize> {
        match self.selected_row() {
            Some(Row::Catalog(i)) => Some(i),
            _ => None,
        }
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

    /// Add a catalog title to the db (or link a same-named unlinked item); selects its row
    pub fn adopt(&mut self, catalog_idx: usize, tags: &[&str]) -> Result<usize> {
        let meta = self.catalog[catalog_idx].meta.clone();
        let handle = Handle {
            name: meta.primary_title.clone(),
            year: meta.year,
        };
        let idx = match (0..self.repo.len())
            .find(|&i| self.repo.get_by_index(i).matches_handle(&handle))
        {
            Some(i) => i,
            None => {
                self.repo.add(Media::from_handle(&handle))?;
                self.repo.len() - 1
            }
        };
        let item = self.repo.get_by_index_mut(idx);
        item.imdb = Some(meta.tconst.clone());
        for tag in tags {
            item.add_tag(tag);
        }
        self.repo.write()?;

        // Make genres etc. available right away, also to `ls`
        self.metas.insert(meta.tconst.clone(), meta);
        imdb::meta::save(&imdb::meta_path()?, &self.metas)?;

        self.refresh_db_ids();
        self.apply_filter();
        if let Some(pos) = self.filtered.iter().position(|r| *r == Row::Item(idx)) {
            self.select(pos);
        }
        Ok(idx)
    }
}

fn get_weight(item: &Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}
