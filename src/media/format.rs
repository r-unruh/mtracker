use colored::Colorize;

use crate::{imdb::Meta, media};

pub struct ListOptions {
    pub note: bool,
    pub tags: bool,
    pub genres: bool,
    pub imdb: bool,
    pub max_rating: u8,
}

/// Tags minus those that merely repeat a fetched genre
pub fn display_tags<'a>(tags: &'a [String], meta: Option<&Meta>) -> Vec<&'a String> {
    tags.iter().filter(|t| !meta.is_some_and(|m| m.has_genre(t))).collect()
}

impl media::Media {
    pub fn as_line(&self, meta: Option<&Meta>, options: &ListOptions) -> String {
        let mut result = String::new();

        if options.max_rating > 0 {
            result += &format!("{} ", self.rating_string(options.max_rating));
        };

        if self.on_watchlist() {
            result += &"WL: ".bold().to_string();
        }

        result += &self.name;

        if let Some(year) = self.year {
            result += &format!(" ({year})").dimmed().to_string();
        }

        if options.tags {
            let tags = display_tags(&self.tags, meta);
            if !tags.is_empty() {
                let tags: Vec<&str> = tags.iter().map(|t| t.as_str()).collect();
                result += &format!(" [{}]", tags.join(", "));
            }
        }

        if let Some(m) = meta {
            if options.genres && !m.genres.is_empty() {
                result += &format!(" {{{}}}", m.genres.join(", "));
            }
            if options.imdb {
                result += &format!("  {}", m.rating_string());
                if !m.directors.is_empty() {
                    result += &format!("  {}", m.directors.join(", "));
                }
            }
        }

        if options.note && !&self.note.is_empty() {
            result += &format!(": {}", self.note);
        }

        result
    }

    fn rating_string(&self, max_rating: u8) -> String {
        if let Some(r) = self.rating {
            let mut result = String::new();
            for i in 0..max_rating {
                result += if r > i { "+" } else { "-" };
            }
            result.replace('+', &"+".bold().to_string())
        } else {
            let mr: usize = max_rating.into();
            format!("{:?<mr$}", "").dimmed().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::Media;

    fn opts(max_rating: u8, note: bool, tags: bool) -> ListOptions {
        ListOptions {
            max_rating,
            note,
            tags,
            genres: false,
            imdb: false,
        }
    }

    fn meta() -> Meta {
        Meta {
            tconst: "tt0078748".into(),
            title_type: crate::imdb::TitleType::Movie,
            primary_title: "Alien".into(),
            original_title: "Alien".into(),
            year: Some(1979),
            runtime: Some(117),
            genres: vec!["horror".into(), "sci-fi".into()],
            rating: Some(85),
            votes: 1,
            directors: vec!["Ridley Scott".into()],
        }
    }

    fn setup() {
        colored::control::set_override(false);
    }

    #[test]
    fn as_line_basic() {
        setup();
        let m = Media::new("Alien", Some(1979));
        let line = m.as_line(None, &opts(0, false, false));
        assert_eq!(line, "Alien (1979)");
    }

    #[test]
    fn as_line_with_note() {
        setup();
        let mut m = Media::new("Alien", None);
        m.note = "classic".into();
        let line = m.as_line(None, &opts(0, true, false));
        assert_eq!(line, "Alien: classic");
    }

    #[test]
    fn as_line_with_tags() {
        setup();
        let mut m = Media::new("Alien", None);
        m.tags = vec!["horror".into(), "sci-fi".into()];
        let line = m.as_line(None, &opts(0, false, true));
        assert_eq!(line, "Alien [horror, sci-fi]");
    }

    #[test]
    fn as_line_with_meta() {
        setup();
        let mut m = Media::new("Alien", None);
        m.tags = vec!["Horror".into(), "classic".into()];
        let meta = meta();

        // Tags duplicating a genre are hidden
        let line = m.as_line(Some(&meta), &opts(0, false, true));
        assert_eq!(line, "Alien [classic]");

        let line = m.as_line(None, &opts(0, false, true));
        assert_eq!(line, "Alien [Horror, classic]");

        let o = ListOptions {
            max_rating: 0,
            note: false,
            tags: false,
            genres: true,
            imdb: true,
        };
        let line = m.as_line(Some(&meta), &o);
        assert_eq!(line, "Alien {horror, sci-fi}  8.5  Ridley Scott");

        // Flags without metadata change nothing
        let line = m.as_line(None, &o);
        assert_eq!(line, "Alien");
    }

    #[test]
    fn as_line_with_rating() {
        setup();
        let mut m = Media::new("Alien", None);
        m.rating = Some(3);
        let line = m.as_line(None, &opts(5, false, false));
        assert_eq!(line, "+++-- Alien");
    }

    #[test]
    fn as_line_unrated_with_max_rating() {
        setup();
        let m = Media::new("Alien", None);
        let line = m.as_line(None, &opts(3, false, false));
        assert_eq!(line, "??? Alien");
    }

    #[test]
    fn as_line_watchlist() {
        setup();
        let mut m = Media::new("Alien", None);
        m.tags = vec!["watchlist".into()];
        let line = m.as_line(None, &opts(0, false, false));
        assert_eq!(line, "WL: Alien");
    }
}
