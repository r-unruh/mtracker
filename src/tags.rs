use crate::media;
use anyhow::Result;
use clap::{ArgMatches, Command};
use std::collections::HashMap;

pub fn command() -> Command {
    Command::new("tags")
        .about("List all tags sorted by frequency")
        .arg_required_else_help(false)
}

pub fn handle(_matches: &ArgMatches) -> Result<()> {
    let repo = media::repo::Repo::default();

    // Get list of tags (including duplicates)
    let tags = repo.get_all()
        .into_iter()
        .map(|i| i.tags.clone())
        .flatten();

    // Count tags
    let mut map = HashMap::new();
    for n in tags {
        *map.entry(n).or_insert(0) += 1;
    }

    // Get flat list of tags, sorted by frequency
    let mut tags: Vec<_> = map.iter().collect();
    tags.sort_by(|a, b| b.1.cmp(a.1));

    for t in tags.into_iter().map(|i| i.0) {
        println!("{}", t);
    }

    Ok(())
}
