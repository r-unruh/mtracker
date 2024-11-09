use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media::repo;

pub fn command() -> Command {
    Command::new("rm")
        .visible_aliases(["remove"])
        .about("Remove item or tag(s)")
        .arg_required_else_help(true)
        .arg(args::identifier())
        .arg(args::year())
        .arg(args::tag().help("Tag(s) to remove, comma-separated"))
}

pub fn handle(matches: &ArgMatches) -> Result<()> {
    // Get args
    let handle = arg_util::handle_from_matches(matches)?.unwrap();
    let tags = arg_util::tags_from_matches(matches);

    // Init repo
    let mut repo = repo::Repo::default();

    // Remove item
    if tags.is_empty() {
        repo.remove_by_handle(&handle)?;
        println!("Removed: {handle}");

    // Remove tags
    } else {
        repo.update(&handle, |m| {
            for tag in tags {
                match m.remove_tag(tag) {
                    Ok(()) => println!("Removed tag from {handle}: {tag}"),
                    Err(_) => eprintln!("Tag not found: {tag}"),
                }
            }
        })?;
    }

    repo.write()
}
