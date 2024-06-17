use clap::{crate_authors, crate_name, crate_version, Command};

mod add;
mod arg_util;
mod args;
mod edit;
mod list;
mod media;
mod rate;
mod remove;
mod unrate;

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup commands
    let matches = Command::new(crate_name!())
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

    // Get database path
    let mut db_file_path = dirs::data_dir().expect("failed to get user data directory");
    db_file_path.push(format!("{}/db.txt", crate_name!()));

    // Init media repo
    let mut repo = media::repo::Repo::new(&db_file_path);

    // Run command
    match matches.subcommand() {
        Some(("ls", matches)) => list::handle(&mut repo, matches),
        Some(("add", matches)) => add::handle(&mut repo, matches),
        Some(("rm", matches)) => remove::handle(&mut repo, matches),
        Some(("rate", matches)) => rate::handle(&mut repo, matches),
        Some(("unrate", matches)) => unrate::handle(&mut repo, matches),
        Some(("edit", matches)) => edit::handle(&mut repo, matches),
        _ => unreachable!(),
    }
}
