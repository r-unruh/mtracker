use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media::repo;

pub fn command() -> Command {
    Command::new("unrate")
        .visible_aliases(["u"])
        .about("Remove rating from item")
        .arg_required_else_help(true)
        .arg(args::identifier())
        .arg(args::year())
}

pub fn handle(repo: &mut repo::Repo, matches: &ArgMatches) -> Result<()> {
    // Get args
    let handle = arg_util::handle_from_matches(matches)?.unwrap();

    // Init repo
    repo.read()?;

    let media = repo.get_or_create(&handle);

    media.rating = None;
    println!("Removed rating from: {handle}");

    repo.write()
}
