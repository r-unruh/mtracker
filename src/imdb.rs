//! IMDb datasets (https://datasets.imdbws.com/): download, match, cache.

use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};

use crate::media::Media;

pub mod dataset;
pub mod download;
pub mod matcher;
pub mod meta;
pub mod names;

/// Required by IMDb's dataset terms of use.
pub const ATTRIBUTION: &str =
    "Information courtesy of IMDb (https://www.imdb.com). Used with permission.";

/// `~/.cache/mtracker`
pub fn cache_dir() -> Result<PathBuf> {
    let mut path =
        dirs::cache_dir().ok_or_else(|| anyhow!("failed to get user cache directory"))?;
    path.push(env!("CARGO_PKG_NAME"));
    Ok(path)
}

pub fn meta_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join(meta::FILE_NAME))
}

pub const CATALOG_FILE: &str = "catalog.tsv";

pub fn catalog_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join(CATALOG_FILE))
}

/// Catalog title with a precomputed lowercase search key
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogEntry {
    pub meta: Meta,
    /// "primary title | original title | director, director", lowercased
    pub key: String,
}

impl CatalogEntry {
    pub fn new(meta: Meta) -> Self {
        let mut key = meta.primary_title.to_lowercase();
        if meta.original_title != meta.primary_title {
            key.push_str(" | ");
            key.push_str(&meta.original_title.to_lowercase());
        }
        if !meta.directors.is_empty() {
            key.push_str(" | ");
            key.push_str(&meta.directors.join(", ").to_lowercase());
        }
        Self { meta, key }
    }
}

/// Load the search catalog written by `sync`; empty if there is none.
pub fn load_catalog() -> Result<Vec<CatalogEntry>> {
    Ok(meta::load_vec(&catalog_path()?)?.into_iter().map(CatalogEntry::new).collect())
}

/// Load the metadata cache; empty if nothing has been synced yet.
pub fn load_meta() -> Result<HashMap<String, Meta>> {
    meta::load(&meta_path()?)
}

/// Metadata for an item, if it is linked and the cache knows the title.
pub fn meta_for<'a>(metas: &'a HashMap<String, Meta>, item: &Media) -> Option<&'a Meta> {
    item.imdb.as_ref().and_then(|id| metas.get(id))
}

/// IMDb `titleType`, restricted to the kinds of titles worth tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleType {
    Movie,
    Series,
    MiniSeries,
    TvMovie,
    TvSpecial,
    Video,
    Short,
}

impl TitleType {
    /// `None` for ignored types (episodes, video games, ...)
    pub fn from_imdb(s: &str) -> Option<Self> {
        match s {
            "movie" => Some(Self::Movie),
            "tvSeries" => Some(Self::Series),
            "tvMiniSeries" => Some(Self::MiniSeries),
            "tvMovie" => Some(Self::TvMovie),
            "tvSpecial" => Some(Self::TvSpecial),
            "video" => Some(Self::Video),
            "short" => Some(Self::Short),
            _ => None,
        }
    }

    /// Lower is better when several titles share name and year.
    pub fn preference(self) -> u8 {
        match self {
            Self::Movie => 0,
            Self::Series => 1,
            Self::MiniSeries => 2,
            Self::TvMovie => 3,
            Self::TvSpecial => 4,
            Self::Video => 5,
            Self::Short => 6,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Series => "series",
            Self::MiniSeries => "miniseries",
            Self::TvMovie => "tvmovie",
            Self::TvSpecial => "tvspecial",
            Self::Video => "video",
            Self::Short => "short",
        }
    }
}

impl fmt::Display for TitleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TitleType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "movie" => Ok(Self::Movie),
            "series" => Ok(Self::Series),
            "miniseries" => Ok(Self::MiniSeries),
            "tvmovie" => Ok(Self::TvMovie),
            "tvspecial" => Ok(Self::TvSpecial),
            "video" => Ok(Self::Video),
            "short" => Ok(Self::Short),
            _ => Err(anyhow!("unknown title type: {s}")),
        }
    }
}

/// Metadata about one IMDb title, derived from the datasets.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    pub tconst: String,
    pub title_type: TitleType,

    /// IMDb's display title (usually the English / international title)
    pub primary_title: String,

    /// Title in the original language
    pub original_title: String,
    pub year: Option<u16>,
    pub runtime: Option<u16>,

    /// Lowercase, e.g. "sci-fi"
    pub genres: Vec<String>,

    /// IMDb user rating in tenths: 78 == 7.8
    pub rating: Option<u8>,
    pub votes: u32,
    pub directors: Vec<String>,
}

impl Meta {
    /// "7.8", or "-" if unrated
    pub fn rating_string(&self) -> String {
        match self.rating {
            Some(r) => format!("{}.{}", r / 10, r % 10),
            None => "-".into(),
        }
    }

    pub fn has_genre(&self, genre: &str) -> bool {
        self.genres.iter().any(|g| g.eq_ignore_ascii_case(genre))
    }

    /// "Title (year)", with the original title appended if it differs
    pub fn describe(&self) -> String {
        let mut s = self.primary_title.clone();
        if let Some(y) = self.year {
            s += &format!(" ({y})");
        }
        if self.original_title != self.primary_title {
            s += &format!(" / {}", self.original_title);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_type_roundtrip() {
        for s in ["movie", "tvSeries", "tvMiniSeries", "tvMovie", "tvSpecial", "video", "short"] {
            let t = TitleType::from_imdb(s).unwrap();
            assert_eq!(t.as_str().parse::<TitleType>().unwrap(), t);
        }
        assert!(TitleType::from_imdb("tvEpisode").is_none());
        assert!(TitleType::from_imdb("videoGame").is_none());
        assert!("episode".parse::<TitleType>().is_err());
    }

    #[test]
    fn rating_string() {
        let mut m = Meta {
            tconst: "tt1".into(),
            title_type: TitleType::Movie,
            primary_title: "A".into(),
            original_title: "A".into(),
            year: None,
            runtime: None,
            genres: vec![],
            rating: Some(78),
            votes: 0,
            directors: vec![],
        };
        assert_eq!(m.rating_string(), "7.8");
        m.rating = Some(100);
        assert_eq!(m.rating_string(), "10.0");
        m.rating = None;
        assert_eq!(m.rating_string(), "-");
    }
}
