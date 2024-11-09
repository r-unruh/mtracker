use anyhow::{anyhow, Result};
use clap::{Arg, ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media;

pub fn command() -> Command {
    Command::new("ls")
        .visible_aliases(["list"])
        .about("List items")
        .arg_required_else_help(false)
        .arg(args::tag().help("Tag(s) to search for, comma-separated"))
        .arg(args::note_bool().help("Whether to display notes"))
        .arg(args::tags_bool().help("Whether to display tags"))
        .arg(
            Arg::new("YEAR")
                .required(false)
                .short('y')
                .long("year")
                .help("Specify year of release")
                .long_help(
                    "Year of release

Valid operators: >, >=, <, <=

If this argument is provided, all items without year numbers are ignored.

Examples:
--year=2021        Items released in 2021
--year=2020-2022   Items released from 2020 to 2022
--year=\">2021\"     Items released from 2022 to 9999
--year=\">=2021\"    Items released from 2021 to 9999",
                ),
        )
}

pub fn handle(repo: &mut media::repo::Repo, matches: &ArgMatches) -> Result<()> {
    // Get args
    let tags = arg_util::tags_from_matches(matches);
    let year = matches.try_get_one::<String>("YEAR").unwrap_or(None);

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

    // Filter by tags
    if !tags.is_empty() {
        items.retain(|i| tags.iter().all(|t| i.matches_tag(t)));
    }

    // Filter by year
    if let Some(y) = year {
        let (min, max) = get_year_min_max(y)?;
        items.retain(|i| match i.year {
            Some(y) => y >= min && y <= max,
            None => false,
        });
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

fn get_year_min_max(input: &str) -> Result<(u16, u16)> {
    match input.len() {
        4 => {
            let y = input.parse::<u16>()?;
            Ok((y, y))
        }
        5 => {
            let y = input[1..].parse::<u16>()?;
            match &input[..1] {
                "=" => Ok((y, y)),
                ">" | "+" => Ok((y + 1, 9999)),
                "<" | "-" => Ok((0, y - 1)),
                _ => Err(anyhow!("invalid comparison symbol")),
            }
        }
        6 => {
            let y = input[2..].parse::<u16>()?;
            match &input[..2] {
                "==" => Ok((y, y)),
                ">=" => Ok((y, 9999)),
                "<=" => Ok((0, y)),
                _ => Err(anyhow!("invalid comparison symbol")),
            }
        }
        9 => {
            let a = input[..4].parse::<u16>()?;
            let b = input[5..].parse::<u16>()?;
            if a <= b {
                Ok((a, b))
            } else {
                Ok((b, a))
            }
        }
        _ => Err(anyhow!("invalid year")),
    }
}

fn get_weight(item: &media::Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_min_max_works() {
        // Valid input
        assert_eq!(get_year_min_max("2023").unwrap(), (2023, 2023));
        assert_eq!(get_year_min_max("2024").unwrap(), (2024, 2024));
        assert_eq!(get_year_min_max("=2024").unwrap(), (2024, 2024));
        assert_eq!(get_year_min_max("==2024").unwrap(), (2024, 2024));
        assert_eq!(get_year_min_max(">=2020").unwrap(), (2020, 9999));
        assert_eq!(get_year_min_max(">2020").unwrap(), (2021, 9999));
        assert_eq!(get_year_min_max("+2020").unwrap(), (2021, 9999));
        assert_eq!(get_year_min_max("<=2020").unwrap(), (0, 2020));
        assert_eq!(get_year_min_max("<2020").unwrap(), (0, 2019));
        assert_eq!(get_year_min_max("-2020").unwrap(), (0, 2019));
        assert_eq!(get_year_min_max("2010-2020").unwrap(), (2010, 2020));
        assert_eq!(get_year_min_max("2020-2010").unwrap(), (2010, 2020));

        // Invalid input
        assert!(get_year_min_max("invalid").is_err());
        assert!(get_year_min_max("#2024").is_err());
        assert!(get_year_min_max("20244").is_err());
    }
}
