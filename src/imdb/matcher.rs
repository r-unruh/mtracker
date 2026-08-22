//! Match items to IMDb titles by name and year.

use unicode_normalization::UnicodeNormalization;

use super::Meta;

/// Lowercase, no diacritics, non-alphanumerics collapsed to single spaces
pub fn normalize(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut pending_space = false;

    let mut push = |c: char| {
        if c.is_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.extend(c.to_lowercase());
        } else {
            pending_space = true;
        }
    };

    if title.is_ascii() {
        // Fast path: the vast majority of titles
        title.chars().for_each(&mut push);
    } else {
        // NFKD splits "é" into "e" + combining accent; drop the accents
        title.nfkd().filter(|c| !is_combining_mark(*c)).for_each(&mut push);
    }
    out
}

fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchKind {
    /// Name and year match (possibly picked among several by popularity)
    Exact,

    /// Year differs by one
    NearYear,

    /// Only one title with this name exists, but the year differs
    YearMismatch,

    /// The item has no year; picked by popularity
    NoYear,
}

/// Pick the best of several candidates: most votes wins, title type breaks ties.
fn pick<'a>(cands: impl Iterator<Item = &'a Meta>) -> Option<&'a Meta> {
    cands.max_by_key(|m| (m.votes, std::cmp::Reverse(m.title_type.preference())))
}

/// Pick the best candidate (all share the item's normalized name)
pub fn select<'a>(year: Option<u16>, cands: &[&'a Meta]) -> Option<(&'a Meta, MatchKind)> {
    let Some(year) = year else {
        return pick(cands.iter().copied()).map(|m| (m, MatchKind::NoYear));
    };

    if let Some(m) = pick(cands.iter().copied().filter(|m| m.year == Some(year))) {
        return Some((m, MatchKind::Exact));
    }
    let near = cands.iter().copied().filter(|m| m.year.is_some_and(|y| y.abs_diff(year) == 1));
    if let Some(m) = pick(near) {
        return Some((m, MatchKind::NearYear));
    }
    if let [only] = cands {
        return Some((only, MatchKind::YearMismatch));
    }
    None
}

#[cfg(test)]
fn refs(cands: &[Meta]) -> Vec<&Meta> {
    cands.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imdb::TitleType;

    fn meta(tconst: &str, title_type: TitleType, year: Option<u16>, votes: u32) -> Meta {
        Meta {
            tconst: tconst.into(),
            title_type,
            primary_title: "X".into(),
            original_title: "X".into(),
            year,
            runtime: None,
            genres: vec![],
            rating: None,
            votes,
            directors: vec![],
        }
    }

    #[test]
    fn normalizes() {
        assert_eq!(normalize("Triangle of Sadness"), "triangle of sadness");
        assert_eq!(normalize("  Jeff, Who Lives at Home! "), "jeff who lives at home");
        assert_eq!(normalize("Amélie"), "amelie");
        assert_eq!(normalize("Perfetti sconosciuti"), "perfetti sconosciuti");
        assert_eq!(normalize("Alien³"), "alien3");
        assert_eq!(normalize("Die Welle / The Wave"), "die welle the wave");
        assert_eq!(normalize("..."), "");
    }

    #[test]
    fn exact_year_prefers_votes_then_type() {
        let cands = vec![
            meta("short", TitleType::Short, Some(2017), 100),
            meta("movie", TitleType::Movie, Some(2017), 100),
            meta("other", TitleType::Movie, Some(2017), 10),
            meta("later", TitleType::Movie, Some(2018), 100_000),
        ];
        let (m, kind) = select(Some(2017), &refs(&cands)).unwrap();
        assert_eq!(m.tconst, "movie");
        assert_eq!(kind, MatchKind::Exact);
    }

    #[test]
    fn near_year() {
        let cands = vec![
            meta("a", TitleType::Movie, Some(2018), 5),
            meta("b", TitleType::Movie, Some(2016), 50),
        ];
        let (m, kind) = select(Some(2017), &refs(&cands)).unwrap();
        assert_eq!(m.tconst, "b");
        assert_eq!(kind, MatchKind::NearYear);
    }

    #[test]
    fn unique_candidate_with_wrong_or_unknown_year() {
        let cands = vec![meta("a", TitleType::Movie, Some(2011), 5)];
        let (m, kind) = select(Some(2015), &refs(&cands)).unwrap();
        assert_eq!(m.tconst, "a");
        assert_eq!(kind, MatchKind::YearMismatch);

        let cands = vec![meta("a", TitleType::Movie, None, 5)];
        assert_eq!(select(Some(2026), &refs(&cands)).unwrap().1, MatchKind::YearMismatch);
    }

    #[test]
    fn ambiguous_wrong_year_is_no_match() {
        let cands = vec![
            meta("a", TitleType::Movie, Some(2011), 5),
            meta("b", TitleType::Movie, Some(1990), 5),
        ];
        assert!(select(Some(2015), &refs(&cands)).is_none());
        assert!(select(Some(2015), &[]).is_none());
    }

    #[test]
    fn no_year_picks_popular() {
        let cands = vec![
            meta("a", TitleType::Movie, Some(2011), 5),
            meta("b", TitleType::Series, Some(1990), 500),
        ];
        let (m, kind) = select(None, &refs(&cands)).unwrap();
        assert_eq!(m.tconst, "b");
        assert_eq!(kind, MatchKind::NoYear);
    }
}
