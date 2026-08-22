//! Person id -> name cache, so name.basics is only scanned for unknown people.

use std::{collections::HashMap, fs, path::Path};

use anyhow::{anyhow, Context, Result};

pub const FILE_NAME: &str = "names.tsv";

pub fn load(path: &Path) -> Result<HashMap<String, String>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    content
        .lines()
        .map(|line| {
            line.split_once('\t')
                .map(|(id, name)| (id.to_string(), name.to_string()))
                .ok_or_else(|| anyhow!("name cache is corrupt: {}\nDelete it.", path.display()))
        })
        .collect()
}

pub fn save(path: &Path, names: &HashMap<String, String>) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut rows: Vec<(&String, &String)> = names.iter().collect();
    rows.sort();
    let content: String = rows.iter().map(|(id, name)| format!("{id}\t{name}\n")).collect();
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_names/names.tsv");
        fs::remove_file(&path).ok();

        let names: HashMap<String, String> =
            [("nm0000001", "Ruben Östlund"), ("nm0000002", "Ridley Scott")]
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect();
        assert!(load(&path).unwrap().is_empty());
        save(&path, &names).unwrap();
        assert_eq!(load(&path).unwrap(), names);

        fs::write(&path, "garbage").unwrap();
        assert!(load(&path).unwrap_err().to_string().contains("corrupt"));
        fs::remove_file(&path).ok();
    }
}
