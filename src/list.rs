use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::{arg_util, args, imdb, media};

pub fn command() -> Command {
    Command::new("ls")
        .visible_aliases(["list"])
        .about("List items")
        .arg_required_else_help(false)
        .arg(args::term().help("Terms to search for (tag, year)"))
        .arg(args::note_bool().help("Whether to display notes"))
        .arg(args::tags_bool().help("Whether to display tags"))
        .arg(args::genres_bool().help("Whether to display IMDb genres (see 'sync')"))
        .arg(args::imdb_bool().help("Whether to display IMDb rating and directors (see 'sync')"))
}

pub fn handle(matches: &ArgMatches) -> Result<()> {
    let repo = arg_util::repo_from_matches(matches)?;
    let metas = imdb::load_meta()?;
    let mut items = repo.get_all();

    let options = media::format::ListOptions {
        note: *matches.get_one::<bool>("NOTE").unwrap_or(&false),
        tags: *matches.get_one::<bool>("TAGS").unwrap_or(&false),
        genres: *matches.get_one::<bool>("GENRES").unwrap_or(&false),
        imdb: *matches.get_one::<bool>("IMDB").unwrap_or(&false),

        // Get max rating BEFORE filtering
        max_rating: items.iter().map(|m| m.rating.unwrap_or(0)).max().unwrap_or(0),
    };

    let max_rating = options.max_rating;
    for t in arg_util::terms_from_matches(matches) {
        let (negated, term) = match t.strip_prefix('!') {
            Some(s) if !s.is_empty() => (true, s),
            _ => (false, t.as_str()),
        };
        items.retain(|i| matches_term(i, imdb::meta_for(&metas, i), term, max_rating) != negated);
    }

    // Sort (watchlist, rating, unrated, alphabetic)
    items.sort_by(|a, b| {
        let a_weight = get_weight(a);
        let b_weight = get_weight(b);

        if a_weight == b_weight {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else {
            b_weight.cmp(&a_weight)
        }
    });

    // Print
    for item in &items {
        println!("{}", item.as_line(imdb::meta_for(&metas, item), &options));
    }

    Ok(())
}

fn get_weight(item: &media::Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}

pub fn matches_term(
    item: &media::Media,
    meta: Option<&imdb::Meta>,
    term: &str,
    max_rating: u8,
) -> bool {
    if let Some(range) = try_parse_year_range(term) {
        return matches!(item.year, Some(y) if y >= range.0 && y <= range.1);
    }
    if term == "rated" {
        return item.rating.is_some();
    }
    if term == "unrated" {
        return item.rating.is_none();
    }
    if let Some(m) = try_match_rating(term, item, max_rating) {
        return m;
    }
    if item.has_tag(term) {
        return true;
    }
    if let Some(m) = meta {
        if m.has_genre(term) || term.eq_ignore_ascii_case(m.title_type.as_str()) {
            return true;
        }
        let lower = term.to_lowercase();
        if m.directors.iter().any(|d| d.to_lowercase().contains(&lower)) {
            return true;
        }
    }
    item.name.to_lowercase().contains(&term.to_lowercase())
}

/// Filter term vs. catalog title; rating terms and tags never match (they're about your items)
pub fn matches_catalog(key: &str, meta: &imdb::Meta, term: &str) -> bool {
    if let Some(range) = try_parse_year_range(term) {
        return matches!(meta.year, Some(y) if y >= range.0 && y <= range.1);
    }
    if term == "rated" || term == "unrated" || is_rating_term(term) {
        return false;
    }
    if meta.has_genre(term) || term.eq_ignore_ascii_case(meta.title_type.as_str()) {
        return true;
    }
    key.contains(&term.to_lowercase())
}

fn is_rating_term(term: &str) -> bool {
    !term.is_empty() && term.chars().all(|c| c == '+' || c == '-')
}

fn try_match_rating(term: &str, item: &media::Media, max_rating: u8) -> Option<bool> {
    if !is_rating_term(term) {
        return None;
    }
    let pluses = term.chars().filter(|&c| c == '+').count() as u8;
    let minuses = term.chars().filter(|&c| c == '-').count() as u8;
    let rating = match item.rating {
        Some(r) => r,
        None => return Some(false),
    };
    if pluses > 0 && minuses > 0 {
        Some(rating == pluses)
    } else if minuses > 0 {
        Some(rating <= max_rating.saturating_sub(minuses))
    } else {
        Some(rating >= pluses)
    }
}

pub fn try_parse_year_range(input: &str) -> Option<(u16, u16)> {
    // Byte slicing below is only safe on ASCII; years are ASCII anyway
    if !input.is_ascii() {
        return None;
    }

    // 2024
    if input.len() == 4 {
        return match input.parse::<u16>() {
            Ok(y) => Some((y, y)),
            Err(_) => None,
        };
    }

    // -2024, 2024-
    if input.len() == 5 {
        if &input[..1] == "-" {
            return match input[1..].parse::<u16>() {
                Ok(y) => Some((0, y)),
                Err(_) => None,
            };
        } else if &input[4..] == "-" {
            return match input[..4].parse::<u16>() {
                Ok(y) => Some((y, 9999)),
                Err(_) => None,
            };
        }
    }

    // 2023-2024
    if input.len() == 9 && &input[4..5] == "-" {
        return match input[..4].parse::<u16>() {
            Ok(from) => match input[5..].parse::<u16>() {
                Ok(to) => {
                    if from <= to {
                        Some((from, to))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
            Err(_) => None,
        };
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imdb::{Meta, TitleType};

    #[test]
    fn matches_term_uses_meta() {
        let mut item = media::Media::new("Alien", Some(1979));
        item.tags = vec!["classic".into()];
        let meta = Meta {
            tconst: "tt0078748".into(),
            title_type: TitleType::Movie,
            primary_title: "Alien".into(),
            original_title: "Alien".into(),
            year: Some(1979),
            runtime: None,
            genres: vec!["horror".into(), "sci-fi".into()],
            rating: Some(85),
            votes: 1,
            directors: vec!["Ridley Scott".into()],
        };

        assert!(matches_term(&item, Some(&meta), "horror", 0));
        assert!(matches_term(&item, Some(&meta), "Sci-Fi", 0));
        assert!(matches_term(&item, Some(&meta), "scott", 0));
        assert!(matches_term(&item, Some(&meta), "movie", 0));
        assert!(!matches_term(&item, Some(&meta), "series", 0));
        assert!(!matches_term(&item, Some(&meta), "comedy", 0));

        // Without metadata only tags and name match
        assert!(!matches_term(&item, None, "horror", 0));
        assert!(matches_term(&item, None, "classic", 0));
        assert!(matches_term(&item, None, "ali", 0));
    }

    #[test]
    fn matches_catalog_terms() {
        let entry = imdb::CatalogEntry::new(Meta {
            tconst: "tt7322224".into(),
            title_type: TitleType::Movie,
            primary_title: "Triangle of Sadness".into(),
            original_title: "Triangle of Sadness".into(),
            year: Some(2022),
            runtime: Some(147),
            genres: vec!["comedy".into(), "drama".into()],
            rating: Some(72),
            votes: 219_437,
            directors: vec!["Ruben Östlund".into()],
        });
        let m = |term: &str| matches_catalog(&entry.key, &entry.meta, term);

        assert!(m("östlund"));
        assert!(m("Östlund"));
        assert!(m("triangle"));
        assert!(m("comedy"));
        assert!(m("movie"));
        assert!(m("2022"));
        assert!(m("2020-"));
        assert!(!m("2023"));
        assert!(!m("horror"));
        assert!(!m("series"));
        // Rating terms describe the user's own items only
        assert!(!m("rated"));
        assert!(!m("unrated"));
        assert!(!m("++"));
        assert!(!m("--"));
    }

    #[test]
    fn try_parse_year_range_works() {
        // Valid input
        assert_eq!(try_parse_year_range("2023").unwrap(), (2023, 2023));
        assert_eq!(try_parse_year_range("2024").unwrap(), (2024, 2024));
        assert_eq!(try_parse_year_range("2020-").unwrap(), (2020, 9999));
        assert_eq!(try_parse_year_range("-2020").unwrap(), (0, 2020));
        assert_eq!(try_parse_year_range("1999-2010").unwrap(), (1999, 2010));

        // Invalid input
        assert!(try_parse_year_range("foob").is_none());
        assert!(try_parse_year_range("-foob").is_none());
        assert!(try_parse_year_range("#2024").is_none());
        assert!(try_parse_year_range("20244").is_none());
        assert!(try_parse_year_range("2020-2010").is_none());

        // Non-ASCII input must not panic on byte slicing
        assert!(try_parse_year_range("Über").is_none());
        assert!(try_parse_year_range("Ö").is_none());
        assert!(try_parse_year_range("Öabc-2024").is_none());
        assert!(try_parse_year_range("\"Über\"").is_none());
    }

    #[test]
    fn matches_term_handles_non_ascii() {
        let item = media::Media::new("Über uns", Some(2020));
        assert!(matches_term(&item, None, "über", 0));
        assert!(matches_term(&item, None, "Ü", 0));
        assert!(!matches_term(&item, None, "Öabc", 0));
        assert!(!matches_term(&item, None, "\"", 0));
    }
}
