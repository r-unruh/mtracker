use clap::{crate_authors, crate_version, Command};

mod add;
mod arg_util;
mod args;
mod edit;
mod list;
pub mod media;
mod rate;
mod remove;
mod unrate;

pub fn run(repo: &mut media::repo::Repo) -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("mttracker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("mtracker - cli media tracker")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .subcommand(list::command())
        .subcommand(add::command())
        .subcommand(remove::command())
        .subcommand(rate::command())
        .subcommand(unrate::command())
        .subcommand(edit::command())
        .get_matches();

    match matches.subcommand() {
        Some(("ls", matches)) => {
            list::handle(repo, matches);
            Ok(())
        }
        Some(("add", matches)) => add::handle(repo, matches),
        Some(("rm", matches)) => remove::handle(repo, matches),
        Some(("rate", matches)) => rate::handle(repo, matches),
        Some(("unrate", matches)) => unrate::handle(repo, matches),
        Some(("edit", matches)) => edit::handle(repo, matches),
        _ => unreachable!(),
    }
}
