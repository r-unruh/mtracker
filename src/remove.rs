use clap::{ArgMatches, Command};

use crate::args;
use crate::media::repo;
use crate::parse_util;

pub fn command() -> Command {
    Command::new("rm")
        .visible_aliases(["remove"])
        .about("Remove item or tag(s)")
        .arg_required_else_help(true)
        .arg(args::identifier())
        .arg(args::tag().help("tag(s) to remove"))
        .arg(args::year())
}

pub fn handle(
    repo: &mut repo::Repo,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = parse_util::handle_from_matches(matches)?;
    let tags = parse_util::tags_from_matches(matches);

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
