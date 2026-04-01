use anyhow::{anyhow, Result};

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

/// Media is identified by its name + year ("name (year)") OR just its name if
/// year is not given
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_tag() {
        let mut m = Media::new("Test", None);
        m.add_tag("horror");
        assert_eq!(m.tags, vec!["horror"]);
    }

    #[test]
    fn add_tag_prevents_duplicates() {
        let mut m = Media::new("Test", None);
        m.add_tag("horror");
        m.add_tag("horror");
        assert_eq!(m.tags, vec!["horror"]);
    }

    #[test]
    fn remove_tag() {
        let mut m = Media::new("Test", None);
        m.add_tag("horror");
        m.remove_tag("horror").unwrap();
        assert!(m.tags.is_empty());
    }

    #[test]
    fn remove_tag_not_found() {
        let mut m = Media::new("Test", None);
        assert!(m.remove_tag("horror").is_err());
    }

    #[test]
    fn matches_handle() {
        let m = Media::new("Alien", Some(1979));
        let h1 = handle::Handle::from_user_input("Alien (1979)");
        let h2 = handle::Handle::from_user_input("Alien");
        let h3 = handle::Handle::from_user_input("Aliens (1986)");
        assert!(m.matches_handle(&h1));
        assert!(!m.matches_handle(&h2));
        assert!(!m.matches_handle(&h3));
    }

    #[test]
    fn matches_handle_without_year() {
        let m = Media::new("Alien", None);
        let h1 = handle::Handle::from_user_input("Alien");
        let h2 = handle::Handle::from_user_input("Alien (1979)");
        assert!(m.matches_handle(&h1));
        assert!(!m.matches_handle(&h2));
    }
}
