use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, Command};

mod add;
mod arg_util;
mod args;
mod edit;
mod list;
mod media;
mod rate;
mod remove;
mod tags;
mod unrate;

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
pub fn run() -> Result<()> {
    // Setup commands
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(format!("{} - cli media tracker", crate_name!()))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .arg(args::db())
        .subcommand(list::command())
        .subcommand(add::command())
        .subcommand(remove::command())
        .subcommand(rate::command())
        .subcommand(unrate::command())
        .subcommand(edit::command())
        .subcommand(tags::command())
        .get_matches();

    // Run command
    match matches.subcommand() {
        Some(("ls", matches)) => list::handle(matches),
        Some(("add", matches)) => add::handle(matches),
        Some(("rm", matches)) => remove::handle(matches),
        Some(("rate", matches)) => rate::handle(matches),
        Some(("unrate", matches)) => unrate::handle(matches),
        Some(("edit", matches)) => edit::handle(matches),
        Some(("tags", matches)) => tags::handle(matches),
        _ => unreachable!(),
    }
}
