use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media;

pub fn command() -> Command {
    Command::new("ls")
        .visible_aliases(["list"])
        .about("List items")
        .arg_required_else_help(false)
        .arg(args::term().help("Terms to search for (tag, year)"))
        .arg(args::note_bool().help("Whether to display notes"))
        .arg(args::tags_bool().help("Whether to display tags"))
}

pub fn handle(repo: &mut media::repo::Repo, matches: &ArgMatches) -> Result<()> {
    // Get args
    let terms = arg_util::terms_from_matches(matches);

    // Init repo
    repo.read()?;

    let mut items = repo.get_all();

    let options = media::format::ListOptions {
        note: *matches.get_one::<bool>("NOTE").unwrap_or(&false),
        tags: *matches.get_one::<bool>("TAGS").unwrap_or(&false),

        // Get max rating BEFORE filtering
        max_rating: items
            .iter()
            .map(|m| m.rating.unwrap_or(0))
            .max()
            .unwrap_or(0),
    };

    for t in &terms {
        // Filter by year
        if let Some(range) = try_parse_year_range(t) {
            items.retain(|i| match i.year {
                Some(y) => y >= range.0 && y <= range.1,
                None => false,
            });
        }


        // Filter by tags
        else {
            items.retain(|i| i.has_tag(t));
        }
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
        println!("{}", item.as_line(&options));
    }

    Ok(())
}

fn get_weight(item: &media::Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}

pub fn try_parse_year_range(input: &str) -> Option<(u16, u16)> {
    // 2024
    if input.len() == 4 {
        return match input.parse::<u16>() {
            Ok(y) => Some((y, y)),
            Err(_) => None
        }
    }

    // -2024, 2024-
    if input.len() == 5 {
        if &input[..1] == "-" {
            return match input[1..].parse::<u16>() {
                Ok(y) => Some((0, y)),
                Err(_) => None
            }
        } else if &input[4..] == "-" {
            return match input[..4].parse::<u16>() {
                Ok(y) => Some((y, 9999)),
                Err(_) => None
            }
        }
    }

    // 2023-2024
    if input.len() == 9 && &input[4..5] == "-" {
        return match input[..4].parse::<u16>() {
            Ok(from) => match input[5..].parse::<u16>() {
                Ok(to) => if from <= to {Some((from, to))} else {None},
                Err(_) => None
            }
            Err(_) => None
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_parse_year_range_works() {
        // Valid input
        assert_eq!(try_parse_year_range("2023").unwrap(), (2023, 2023));
        assert_eq!(try_parse_year_range("2024").unwrap(), (2024, 2024));
        assert_eq!(try_parse_year_range("2020-").unwrap(), (2020, 9999));
        assert_eq!(try_parse_year_range("-2020").unwrap(), (0, 2020));
        assert_eq!(try_parse_year_range("1999-2010").unwrap(), (1999,2010));

        // Invalid input
        assert!(try_parse_year_range("foob").is_none());
        assert!(try_parse_year_range("-foob").is_none());
        assert!(try_parse_year_range("#2024").is_none());
        assert!(try_parse_year_range("20244").is_none());
        assert!(try_parse_year_range("2020-2010").is_none());
    }
}
