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
        .arg(args::tag().help("Tag(s) to search for, comma-separated"))
        .arg(args::note_bool().help("Whether to display notes"))
        .arg(args::tags_bool().help("Whether to display tags"))
}

pub fn handle(repo: &mut media::repo::Repo, matches: &ArgMatches) -> Result<()> {
    // Get args
    let tags = arg_util::tags_from_matches(matches);

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
        items.retain(|i| tags.iter().all(|t| i.tags.contains(t)));
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
