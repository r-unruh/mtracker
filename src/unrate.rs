use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::arg_util;
use crate::args;

pub fn command() -> Command {
    Command::new("unrate")
        .visible_aliases(["u"])
        .about("Remove rating from item")
        .arg_required_else_help(true)
        .arg(args::identifier())
        .arg(args::year())
}

pub fn handle(matches: &ArgMatches) -> Result<()> {
    let mut repo = arg_util::repo_from_matches(matches)?;
    let handle = arg_util::handle_from_matches(matches)?.unwrap();

    let media = repo.get_or_create(&handle)?;

    media.rating = None;
    println!("Removed rating from: {handle}");

    repo.write()
}
