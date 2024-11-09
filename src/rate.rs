use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media::repo;

pub fn command() -> Command {
    Command::new("rate")
        .visible_aliases(["r"])
        .about("Rate item")
        .arg_required_else_help(true)
        .arg(args::identifier())
        .arg(
            Arg::new("RATING")
                .required(true)
                .value_parser(clap::value_parser!(u8).range(0..=255))
                .help("Rating (number)"),
        )
        .arg(args::year())
}

pub fn handle(matches: &ArgMatches) -> Result<()> {
    // Get args
    let handle = arg_util::handle_from_matches(matches)?.unwrap();
    let rating = matches.get_one::<u8>("RATING");

    // Init repo
    let mut repo = repo::Repo::default();

    let media = repo.get_or_create(&handle);

    media.rating = rating.copied();
    println!("Rated {handle}: {}", rating.unwrap());

    if media.on_watchlist() {
        media.remove_tag("watchlist")?;
        println!("Removed from watchlist: {handle}");
    }

    repo.write()
}
