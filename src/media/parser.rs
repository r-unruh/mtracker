use anyhow::{anyhow, Result};
use chrono;

use crate::media;

// (key, value)
fn parse_prop<'a, T: std::str::FromStr>(arg: (&'a str, &'a str)) -> Result<Option<T>>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match str::parse::<T>(arg.1) {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(anyhow!("failed to parse {}: {e}", arg.0)),
    }
}

fn parse_last_seen(input: &str) -> Result<Option<chrono::NaiveDate>> {
    match chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        Ok(date) => Ok(Some(date)),
        Err(e) => Err(anyhow!(
            "failed to parse last_seen: {e}\nExpected format: 2024-12-31"
        )),
    }
}

fn parse_tags(input: &str) -> Result<Vec<String>> {
    let tags: Vec<String> = parse_prop::<String>(("tags", input))?
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .collect();

    if tags.contains(&String::new()) {
        Err(anyhow!("empty tag"))
    } else {
        Ok(tags)
    }
}

impl media::Media {
    #[allow(clippy::missing_panics_doc)]
    pub fn from_db_entry(entry: &str) -> Result<Self> {
        let mut year: Option<u16> = None;
        let mut rating: Option<u8> = None;
        let mut note: String = String::new();
        let mut tags: Vec<String> = vec![];
        let mut last_seen: Option<chrono::NaiveDate> = None;

        let mut lines = entry.lines();

        // First line is always the name
        let name = match lines.next() {
            Some(n) => n.to_string(),
            None => {
                return Err(anyhow!("entry can't be empty"));
            }
        };

        // Subsequent lines are key:value pairs
        for line in lines {
            if line.is_empty() {
                return Err(anyhow!("illegal empty line"));
            }

            let (key, value) = match line.split_once(':') {
                Some((n, v)) => (n, v.trim()),
                None => return Err(anyhow!("delimiter missing: {line}")),
            };

            match key {
                "year" => year = parse_prop::<u16>((key, value))?,
                "rating" => rating = parse_prop::<u8>((key, value))?,
                "note" => note = parse_prop::<String>((key, value))?.unwrap(),
                "tags" => tags = parse_tags(value)?,
                "last_seen" => last_seen = parse_last_seen(value)?,
                _ => return Err(anyhow!("unknown key: {key}")),
            };
        }

        Ok(Self {
            name,
            year,
            rating,
            tags,
            note,
            last_seen,
        })
    }

    pub fn to_db_entry(&self) -> String {
        let mut result = String::from(&self.name);
        if let Some(year) = self.year {
            result += format!("\nyear: {year}").as_str();
        }
        if let Some(rating) = self.rating {
            result += format!("\nrating: {rating}").as_str();
        }
        if !self.tags.is_empty() {
            result += format!("\ntags: {}", self.tags.join(", ")).as_str();
        }
        if !self.note.is_empty() {
            result += format!("\nnote: {}", self.note).as_str();
        }
        if let Some(last_seen) = self.last_seen {
            result += format!("\nlast_seen: {last_seen}").as_str();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_from_db_entry() {
        // Badly formatted on purpose
        let entry = "Forrest Gump
year:  1994
rating:2
tags: drama, romance,funny
last_seen: 2020-12-31
note:very long";

        let media = media::Media::from_db_entry(entry).unwrap();
        assert_eq!(media.name, "Forrest Gump");
        assert_eq!(media.year, Some(1994));
        assert_eq!(media.rating, Some(2));
        assert_eq!(media.note, "very long");
        assert_eq!(
            media.last_seen,
            chrono::NaiveDate::from_ymd_opt(2020, 12, 31)
        );
        assert_eq!(media.tags, vec!["drama", "romance", "funny"]);

        // Bad entry, but technically valid
        let entry = "year: 2009
";
        let media = media::Media::from_db_entry(entry).unwrap();
        assert_eq!(media.name, "year: 2009");
        assert_eq!(media.year, None);
        assert_eq!(media.rating, None);
        assert_eq!(media.note, String::new());
        assert_eq!(media.last_seen, None);
        assert!(media.tags.is_empty());
    }

    #[test]
    fn aborts_gracefully() {
        // Empty entry
        let entry = "";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("entry can't be empty"));

        // Illegal empty lines in between
        let entry = "foobar

year: 2009";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("illegal empty line"));

        // Not a number
        let entry = "foobar
year: invalid";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("failed to parse year"));

        // Invalid number
        let entry = "foobar
rating: -4";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("failed to parse rating"));

        // Non-existing key
        let entry = "foobar
foo: bar";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("unknown key"));

        // Empty tags
        let entry = "foobar
tags: a,";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("empty tag"));

        // Prop without delimiter
        let entry = "foobar
name value";
        let error = media::Media::from_db_entry(entry).unwrap_err();
        assert!(error.to_string().starts_with("delimiter missing"));
    }

    #[test]
    fn media_to_db_entry() {
        let media = media::Media {
            name: "Forrest Gump".into(),
            year: Some(1994),
            rating: Some(2),
            tags: vec!["drama".into(), "romance".into()],
            note: "very long".into(),
            last_seen: chrono::NaiveDate::from_ymd_opt(2024, 06, 12),
        };

        let expected = "Forrest Gump
year: 1994
rating: 2
tags: drama, romance
note: very long
last_seen: 2024-06-12";

        assert_eq!(media.to_db_entry(), expected);
    }
}
