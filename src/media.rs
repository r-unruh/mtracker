use anyhow::{anyhow, Result};
use std::convert::Into;

pub mod format;
pub mod handle;
pub mod parser;
pub mod repo;

#[derive(Debug, PartialEq)]
pub struct Media {
    pub name: String,
    pub year: Option<u16>,
    pub rating: Option<u8>,
    pub tags: Vec<String>,
    pub note: String,
    pub last_seen: Option<chrono::NaiveDate>,
}

/// Media is identified by its name + year ("name (year)") OR just its name if year is not given
impl Media {
    pub fn new(name: impl Into<String>, year: Option<u16>) -> Self {
        Media {
            name: name.into(),
            year,
            rating: None,
            tags: vec![],
            note: String::new(),
            last_seen: None,
        }
    }

    pub fn from_handle(handle: &handle::Handle) -> Self {
        Self::new(handle.name.clone(), handle.year)
    }

    pub fn matches_handle(&self, handle: &handle::Handle) -> bool {
        self.name == handle.name && self.year == handle.year
    }

    pub fn matches_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
            || (tag == "rated" && self.rating.is_some())
            || (tag == "unrated" && self.rating.is_none())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.iter().any(|t| t == tag) {
            self.tags.push(tag.into());
        }
    }

    pub fn remove_tag(&mut self, tag: &str) -> Result<()> {
        if let Some(index) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(index);
            Ok(())
        } else {
            Err(anyhow!("Tag not found"))
        }
    }

    pub fn on_watchlist(&self) -> bool {
        self.has_tag("watchlist")
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}
