use anyhow::{anyhow, Result};
use std::{fs, path};
extern crate dirs;

use crate::media;

pub struct Repo {
    pub path: path::PathBuf,
    items: Vec<media::Media>,
}

impl Repo {
    pub fn new(path: &path::Path) -> Result<Self> {
        let mut repo = Repo {
            path: path.to_path_buf(),
            items: vec![],
        };
        repo.read()?;
        Ok(repo)
    }

    pub fn get(&mut self, handle: &media::handle::Handle) -> Option<&mut media::Media> {
        self.items.iter_mut().find(|m| m.matches_handle(handle))
    }

    pub fn get_or_create(&mut self, handle: &media::handle::Handle) -> Result<&mut media::Media> {
        if self.get(handle).is_none() {
            self.add(media::Media::from_handle(handle))?;
            println!("Added new item: {handle}");
        }

        Ok(self.get(handle).unwrap())
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

    pub fn add(&mut self, item: media::Media) -> Result<()> {
        self.items.push(item);
        Ok(())
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

    // Read all items from file into memory
    fn read(&mut self) -> Result<()> {
        let file_content = fs::read_to_string(&self.path).unwrap_or_default();

        // Get blocks of text separated by empty lines
        let blocks = file_content
            .split("\n\n")
            .filter(|b| !b.is_empty())
            .map(str::trim);

        // Parse blocks of text into media items
        for block in blocks {
            self.items.push(media::Media::from_db_entry(block)?);
        }

        Ok(())
    }

    /// Write all items to file
    pub fn write(&self) -> Result<()> {
        // Create path if it doesn't exist
        std::fs::create_dir_all(self.path.parent().unwrap())?;

        let mut output = String::new();
        for entry in self.items.iter().map(media::Media::to_db_entry) {
            output += &entry;
            output += "\n\n";
        }
        Ok(fs::write(&self.path, output.trim())?)
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

        let repo = Repo::new(&path).unwrap();
        let items = repo.get_all();

        assert_eq!(items[0].name, "Forrest Gump");
        assert_eq!(items[0].year, Some(1994));

        assert_eq!(items[1].name, "Alien");
        assert_eq!(items[1].year, Some(1979));

        assert_eq!(items[2].name, "Aliens");
        assert_eq!(items[2].year, Some(1986));

        assert_eq!(items[3].name, "Alien 3");
        assert_eq!(items[3].year, Some(1992));

        assert_eq!(items[4].name, "The Terminator");
        assert_eq!(items[4].year, Some(1984));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn writes() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_writes.txt");
        fs::remove_file(&path).ok();

        let mut repo = Repo::new(&path).unwrap();
        repo.add(media::Media::new("Forrest Gump", Some(1994))).ok();
        repo.add(media::Media::new("Alien", Some(1979))).ok();
        repo.write().unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "Forrest Gump
year: 1994

Alien
year: 1979"
        );
        fs::remove_file(&path).ok();
    }
}
