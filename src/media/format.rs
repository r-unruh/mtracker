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
