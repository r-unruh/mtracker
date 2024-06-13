use std::{fs, path};
extern crate dirs;

use crate::media;

pub struct Repo {
    pub path: path::PathBuf,
    pub items: Vec<media::Media>,
}

impl Repo {
    pub fn load_or_create(path: &path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Repo {
            path: path.clone(),
            items: Self::read(path)?,
        })
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.get(handle) {
            Some(item) => {
                f(item);
                Ok(())
            }
            None => Err(format!("item not found: {handle}").into()),
        }
    }

    pub fn add(&mut self, item: media::Media) {
        self.items.push(item);
    }

    pub fn remove(&mut self, item: &media::Media) {
        if let Some(index) = self.items.iter().position(|m| m == item) {
            self.items.swap_remove(index);
        }
    }

    pub fn remove_by_handle(
        &mut self,
        handle: &media::handle::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.items.iter().position(|m| m.matches_handle(handle)) {
            Some(index) => {
                self.items.swap_remove(index);
                Ok(())
            }
            None => Err(format!("item not found: {}", &handle).into()),
        }
    }

    /// Read all items from file into memory
    fn read(path: &path::PathBuf) -> Result<Vec<media::Media>, Box<dyn std::error::Error>> {
        let file_content = match fs::read_to_string(path) {
            Ok(c) => {
                if c.is_empty() {
                    return Ok(vec![]);
                }
                c
            }
            Err(_) => {
                return Ok(vec![]);
            }
        };

        let blocks = file_content.split("\n\n");

        let mut items: Vec<media::Media> = vec![];
        for block in blocks {
            items.push(media::Media::from_db_entry(block.lines())?);
        }

        Ok(items)
    }

    /// Write all items to file
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
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
year: 1979",
        )
        .unwrap();

        let repo = Repo::load_or_create(&path).unwrap();

        let a = repo.items.get(0).unwrap();
        let b = repo.items.get(1).unwrap();

        assert_eq!(a.name, "Forrest Gump");
        assert_eq!(a.year, Some(1994));
        assert_eq!(b.name, "Alien");
        assert_eq!(b.year, Some(1979));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn writes() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_writes.txt");
        fs::remove_file(&path).ok();

        let mut repo = Repo::load_or_create(&path).unwrap();
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
