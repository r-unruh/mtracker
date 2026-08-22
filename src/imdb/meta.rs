//! Metadata cache: one row per linked title.

use std::{collections::HashMap, fs, path::Path};

use anyhow::{anyhow, Context, Result};

use super::Meta;

pub const FILE_NAME: &str = "meta.tsv";
const HEADER: &str =
    "tconst\ttype\tprimary_title\toriginal_title\tyear\truntime\tgenres\trating\tvotes\tdirectors";
const LIST_SEP: char = '|';

/// Missing file = nothing synced yet, not an error
pub fn load(path: &Path) -> Result<HashMap<String, Meta>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    load_map(&content).with_context(|| {
        format!(
            "metadata cache is corrupt: {}\nDelete it and run 'mtracker sync' again.",
            path.display()
        )
    })
}

pub fn save(path: &Path, metas: &HashMap<String, Meta>) -> Result<()> {
    let mut rows: Vec<&Meta> = metas.values().collect();
    rows.sort_by(|a, b| a.tconst.cmp(&b.tconst));
    save_rows(path, &rows)
}

/// Like [`load`], but keeps the file order (the catalog is sorted by popularity)
pub fn load_vec(path: &Path) -> Result<Vec<Meta>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    parse(&content).with_context(|| {
        format!(
            "catalog is corrupt: {}\nDelete it and run 'mtracker sync' again.",
            path.display()
        )
    })
}

pub fn save_vec(path: &Path, metas: &[Meta]) -> Result<()> {
    let rows: Vec<&Meta> = metas.iter().collect();
    save_rows(path, &rows)
}

fn save_rows(path: &Path, rows: &[&Meta]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, serialize(rows)).with_context(|| format!("failed to write {}", path.display()))
}

fn load_map(content: &str) -> Result<HashMap<String, Meta>> {
    Ok(parse(content)?.into_iter().map(|m| (m.tconst.clone(), m)).collect())
}

fn parse(content: &str) -> Result<Vec<Meta>> {
    let mut lines = content.lines();
    if lines.next() != Some(HEADER) {
        return Err(anyhow!("unexpected header"));
    }
    let mut rows = Vec::new();
    for (i, line) in lines.enumerate() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 10 {
            return Err(anyhow!("line {}: expected 10 columns, found {}", i + 2, f.len()));
        }
        let opt_num = |s: &str| -> Result<Option<u16>> {
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(s.parse()?))
            }
        };
        let list = |s: &str| -> Vec<String> {
            s.split(LIST_SEP).filter(|v| !v.is_empty()).map(str::to_string).collect()
        };
        let meta = Meta {
            tconst: f[0].to_string(),
            title_type: f[1].parse()?,
            primary_title: f[2].to_string(),
            original_title: f[3].to_string(),
            year: opt_num(f[4])?,
            runtime: opt_num(f[5])?,
            genres: list(f[6]),
            rating: opt_num(f[7])?.map(u8::try_from).transpose()?,
            votes: f[8].parse()?,
            directors: list(f[9]),
        };
        rows.push(meta);
    }
    Ok(rows)
}

fn serialize(rows: &[&Meta]) -> String {
    let opt = |v: Option<u16>| v.map(|n| n.to_string()).unwrap_or_default();
    let mut out = String::from(HEADER);
    for m in rows {
        out.push('\n');
        out += &[
            m.tconst.as_str(),
            m.title_type.as_str(),
            &m.primary_title,
            &m.original_title,
            &opt(m.year),
            &opt(m.runtime),
            &m.genres.join(&LIST_SEP.to_string()),
            &opt(m.rating.map(u16::from)),
            &m.votes.to_string(),
            &m.directors.join(&LIST_SEP.to_string()),
        ]
        .join("\t");
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imdb::TitleType;

    #[test]
    fn roundtrip() {
        let mut metas = HashMap::new();
        for m in [
            Meta {
                tconst: "tt7322224".into(),
                title_type: TitleType::Movie,
                primary_title: "Triangle of Sadness".into(),
                original_title: "Triangle of Sadness".into(),
                year: Some(2022),
                runtime: Some(147),
                genres: vec!["comedy".into(), "drama".into()],
                rating: Some(72),
                votes: 219437,
                directors: vec!["Ruben Östlund".into()],
            },
            Meta {
                tconst: "tt0000001".into(),
                title_type: TitleType::Series,
                primary_title: "The Wave".into(),
                original_title: "Die Welle".into(),
                year: None,
                runtime: None,
                genres: vec![],
                rating: None,
                votes: 0,
                directors: vec![],
            },
        ] {
            metas.insert(m.tconst.clone(), m);
        }

        let mut path = std::env::temp_dir();
        path.push("mtracker_test_meta/meta.tsv");
        fs::remove_file(&path).ok();

        assert!(load(&path).unwrap().is_empty());
        save(&path, &metas).unwrap();
        assert_eq!(load(&path).unwrap(), metas);

        fs::write(&path, "garbage").unwrap();
        let err = load(&path).unwrap_err().to_string();
        assert!(err.contains("corrupt"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn vec_roundtrip_keeps_order() {
        let a = Meta {
            tconst: "tt2".into(),
            title_type: TitleType::Movie,
            primary_title: "B".into(),
            original_title: "B".into(),
            year: None,
            runtime: None,
            genres: vec![],
            rating: None,
            votes: 500,
            directors: vec![],
        };
        let b = Meta {
            tconst: "tt1".into(),
            votes: 100,
            ..a.clone()
        };
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_meta/catalog.tsv");
        fs::remove_file(&path).ok();

        assert!(load_vec(&path).unwrap().is_empty());
        save_vec(&path, &[a.clone(), b.clone()]).unwrap();
        assert_eq!(load_vec(&path).unwrap(), vec![a, b]);
        fs::remove_file(&path).ok();
    }
}
