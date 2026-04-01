use colored::Colorize;

use crate::media;

pub struct ListOptions {
    pub note: bool,
    pub tags: bool,
    pub max_rating: u8,
}

impl media::Media {
    pub fn as_line(&self, options: &ListOptions) -> String {
        let mut result = String::new();

        if options.max_rating > 0 {
            result += &format!("{} ", &self.rating_string(options.max_rating));
        };

        if self.on_watchlist() {
            result += &"WL: ".bold().to_string();
        }

        result += &self.name;

        if let Some(year) = self.year {
            result += &format!(" ({year})").dimmed().to_string();
        }

        if options.tags && !self.tags.is_empty() {
            result += &format!(" [{}]", self.tags.join(", "));
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
        }
    }

    fn setup() {
        colored::control::set_override(false);
    }

    #[test]
    fn as_line_basic() {
        setup();
        let m = Media::new("Alien", Some(1979));
        let line = m.as_line(&opts(0, false, false));
        assert_eq!(line, "Alien (1979)");
    }

    #[test]
    fn as_line_with_note() {
        setup();
        let mut m = Media::new("Alien", None);
        m.note = "classic".into();
        let line = m.as_line(&opts(0, true, false));
        assert_eq!(line, "Alien: classic");
    }

    #[test]
    fn as_line_with_tags() {
        setup();
        let mut m = Media::new("Alien", None);
        m.tags = vec!["horror".into(), "sci-fi".into()];
        let line = m.as_line(&opts(0, false, true));
        assert_eq!(line, "Alien [horror, sci-fi]");
    }

    #[test]
    fn as_line_with_rating() {
        setup();
        let mut m = Media::new("Alien", None);
        m.rating = Some(3);
        let line = m.as_line(&opts(5, false, false));
        assert_eq!(line, "+++-- Alien");
    }

    #[test]
    fn as_line_unrated_with_max_rating() {
        setup();
        let m = Media::new("Alien", None);
        let line = m.as_line(&opts(3, false, false));
        assert_eq!(line, "??? Alien");
    }

    #[test]
    fn as_line_watchlist() {
        setup();
        let mut m = Media::new("Alien", None);
        m.tags = vec!["watchlist".into()];
        let line = m.as_line(&opts(0, false, false));
        assert_eq!(line, "WL: Alien");
    }
}
