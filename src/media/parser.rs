use chrono;

use crate::media;

// (key, value)
fn parse_prop<'a, T: std::str::FromStr>(
    arg: (&'a str, &'a str),
) -> Result<Option<T>, Box<dyn std::error::Error>>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match str::parse::<T>(arg.1) {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(format!("failed to parse {}: {e}", arg.0).into()),
    }
}

fn parse_last_seen(input: &str) -> Result<Option<chrono::NaiveDate>, Box<dyn std::error::Error>> {
    match chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        Ok(date) => Ok(Some(date)),
        Err(e) => {
            Err(format!("failed to parse last_seen: {e}\nExpected format: 2024-12-31").into())
        }
    }
}

impl media::Media {
    #[allow(clippy::missing_panics_doc)]
    pub fn from_db_entry(mut lines: std::str::Lines) -> Result<Self, Box<dyn std::error::Error>> {
        let mut year: Option<u16> = None;
        let mut rating: Option<u8> = None;
        let mut note: String = String::new();
        let mut tags: Vec<String> = vec![];
        let mut last_seen: Option<chrono::NaiveDate> = None;

        // First line is always the name
        let name = lines.next().expect("lines can't be empty").to_string();

        // Subsequent lines are key:value pairs
        for line in lines {
            let (key, value) = match line.split_once(':') {
                Some((n, v)) => (n, v.trim()),
                None => return Err(format!("delimiter missing: {line}").into()),
            };

            // Original
            match key {
                "year" => year = parse_prop::<u16>((key, value))?,
                "rating" => rating = parse_prop::<u8>((key, value))?,
                "note" => note = parse_prop::<String>((key, value))?.unwrap(),
                "tags" => {
                    tags = parse_prop::<String>((key, value))?
                        .unwrap()
                        .split(", ")
                        .map(str::to_string)
                        .collect();
                }
                "last_seen" => last_seen = parse_last_seen(value)?,
                _ => return Err(format!("unknown key: {key}").into()),
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
        let lines = "Forrest Gump
year:  1994
rating:2
tags: drama, romance
last_seen: 2020-12-31
note:very long"
            .lines();

        let media = media::Media::from_db_entry(lines).unwrap();
        assert_eq!(media.name, "Forrest Gump");
        assert_eq!(media.year, Some(1994));
        assert_eq!(media.rating, Some(2));
        assert_eq!(media.note, "very long");
        assert_eq!(
            media.last_seen,
            chrono::NaiveDate::from_ymd_opt(2020, 12, 31)
        );
        assert_eq!(media.tags, vec!["drama", "romance"]);

        // Bad entry, but technically valid
        let lines = "year: 2009".lines();
        let media = media::Media::from_db_entry(lines).unwrap();
        assert_eq!(media.name, "year: 2009");
        assert_eq!(media.year, None);
        assert_eq!(media.rating, None);
        assert_eq!(media.note, String::new());
        assert_eq!(media.last_seen, None);
        assert!(media.tags.is_empty());
    }

    #[test]
    fn aborts_gracefully() {
        // Not a number
        let lines = "foobar
year: invalid"
            .lines();
        let error = media::Media::from_db_entry(lines).unwrap_err();
        assert!(error.to_string().starts_with("failed to parse year"));

        // Invalid number
        let lines = "foobar
rating: -4"
            .lines();
        let error = media::Media::from_db_entry(lines).unwrap_err();
        assert!(error.to_string().starts_with("failed to parse rating"));

        // Non-existing key
        let lines = "foobar
foo: bar"
            .lines();
        let error = media::Media::from_db_entry(lines).unwrap_err();
        assert!(error.to_string().starts_with("unknown key"));

        // Prop without delimiter
        let lines = "foobar
name value"
            .lines();
        let error = media::Media::from_db_entry(lines).unwrap_err();
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
