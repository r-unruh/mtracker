use clap::{ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media;

pub fn command() -> Command {
    Command::new("ls")
        .visible_aliases(["list"])
        .about("List items")
        .arg_required_else_help(false)
        .arg(args::tag().help("tag(s) to search for"))
        .arg(args::note_bool().help("whether to display notes"))
        .arg(args::tags_bool().help("whether to display tags"))
}

pub fn handle(repo: &media::repo::Repo, matches: &ArgMatches) {
    let tags = arg_util::tags_from_matches(matches);
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

    // Sort (watchlist, rating, unrated)
    items.sort_by(|a, b| get_weight(a).cmp(&get_weight(b)).reverse());

    // Print
    for item in &items {
        println!("{}", item.as_line(&options));
    }
}

fn get_weight(item: &media::Media) -> usize {
    item.rating.unwrap_or(0) as usize + 1 + if item.has_tag("watchlist") { 1000 } else { 0 }
}
