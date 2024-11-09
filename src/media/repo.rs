use anyhow::{anyhow, Result};
use std::fs;
extern crate dirs;

use crate::media;

use super::config::Config;

pub struct Repo {
    pub config: Config,
    pub items: Vec<media::Media>,
}

impl Repo {
    fn new(config: Config) -> Self {
        let mut repo = Self {
            config,
            items: Vec::default(),
        };
        repo.init();
        repo
    }

    pub fn get(&mut self, handle: &media::handle::Handle) -> Option<&mut media::Media> {
        self.items.iter_mut().find(|m| m.matches_handle(handle))
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn get_or_create(&mut self, handle: &media::handle::Handle) -> &mut media::Media {
        // Media gets created here..
        if self.get(handle).is_none() {
            self.add(media::Media::from_handle(handle));
            println!("Added new item: {handle}");
        }

        // ..so this can't fail
        self.get(handle).unwrap()
    }

    pub fn get_all(&self) -> Vec<&media::Media> {
        self.items.iter().collect()
    }

    pub fn update(
        &mut self,
        handle: &media::handle::Handle,
        f: impl FnOnce(&mut media::Media),
    ) -> Result<()> {
        match self.get(handle) {
            Some(item) => {
                f(item);
                Ok(())
            }
            None => Err(anyhow!("item not found: {handle}")),
        }
    }

    pub fn add(&mut self, item: media::Media) {
        self.items.push(item);
    }

    pub fn remove_by_handle(&mut self, handle: &media::handle::Handle) -> Result<()> {
        match self.items.iter().position(|m| m.matches_handle(handle)) {
            Some(index) => {
                self.items.swap_remove(index);
                Ok(())
            }
            None => Err(anyhow!("item not found: {}", &handle)),
        }
    }

    /// Read all items from file into memory
    fn init(&mut self) {
        let file_content = fs::read_to_string(&self.config.path).unwrap_or_default();

        // Get blocks of text separated by empty lines
        let blocks: Vec<_> = file_content
            .split("\n\n")
            .filter(|b| !b.is_empty())
            .map(str::trim)
            .collect();

        // Parse blocks of text into media items
        for block in blocks {
            self.items.push(media::Media::from_db_entry(block).unwrap());
        }
    }

    /// Write all items to file
    pub fn write(&self) -> Result<()> {
        // Create path if it doesn't exist
        std::fs::create_dir_all(self.config.path.parent().unwrap())?;

        let mut output = String::new();
        for entry in self.items.iter().map(media::Media::to_db_entry) {
            output += &entry;
            output += "\n\n";
        }
        Ok(fs::write(&self.config.path, output.trim())?)
    }
}

impl Default for Repo {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_reads.txt");

        fs::write(
            &path,
            "Forrest Gump
year: 1994

Alien
year: 1979


Aliens
year: 1986



Alien 3
year: 1992




The Terminator
year: 1984
",
        )
        .unwrap();

        let config = Config { path: path.clone() };
        let repo = Repo::new(config);

        let media = repo.items.get(0).unwrap();
        assert_eq!(media.name, "Forrest Gump");
        assert_eq!(media.year, Some(1994));

        let media = repo.items.get(1).unwrap();
        assert_eq!(media.name, "Alien");
        assert_eq!(media.year, Some(1979));

        let media = repo.items.get(2).unwrap();
        assert_eq!(media.name, "Aliens");
        assert_eq!(media.year, Some(1986));

        let media = repo.items.get(3).unwrap();
        assert_eq!(media.name, "Alien 3");
        assert_eq!(media.year, Some(1992));

        let media = repo.items.get(4).unwrap();
        assert_eq!(media.name, "The Terminator");
        assert_eq!(media.year, Some(1984));

        let media = repo.items.get(5);
        assert_eq!(media, None);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn reads_empty_file() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_reads.txt");
        fs::write(&path, "").unwrap();
        let config = Config { path: path.clone() };
        let repo = Repo::new(config);
        assert!(repo.items.is_empty());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn writes() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_writes.txt");
        fs::remove_file(&path).ok();

        let config = Config { path: path.clone() };
        let mut repo = Repo::new(config);
        repo.add(media::Media::new("Forrest Gump", Some(1994)));
        repo.add(media::Media::new("Alien", Some(1979)));

        repo.write().unwrap();

        let body = fs::read_to_string(&path).unwrap();

        assert_eq!(
            body,
            "Forrest Gump
year: 1994

Alien
year: 1979"
        );
        fs::remove_file(&path).ok();
    }
}
