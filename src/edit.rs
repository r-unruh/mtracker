use anyhow::{anyhow, Result};
use clap::{ArgMatches, Command};
use std::io::{Read, Write};

use crate::arg_util;
use crate::args;
use crate::media::config::Config;
use crate::media::{handle, repo, Media};

pub fn command() -> Command {
    clap::Command::new("edit")
        .aliases(["e"])
        .about("Edit item or whole database with the default editor")
        .arg(args::identifier().required(false))
        .arg(args::year())
        .arg_required_else_help(false)
}

pub fn handle(matches: &ArgMatches) -> Result<()> {
    if let Some(handle) = arg_util::handle_from_matches(matches)? {
        edit_db_entry(&handle)
    } else {
        edit_db()
    }
}

fn edit_db_entry(handle: &handle::Handle) -> Result<()> {
    // Init repo
    let mut repo = repo::Repo::default();

    // Find media
    let Some(item) = repo.get(handle) else {
        return Err(anyhow!("item not found: {handle}"));
    };

    // Edit with editor
    let db_entry = edit::edit(item.to_db_entry())?;

    // Create new item based on db entry
    let new_item = match Media::from_db_entry(&db_entry) {
        Ok(item) => item,
        Err(e) => {
            return Err(anyhow!(
                "failed to edit {handle}: {e}\n\nYour input:\n{db_entry}\n\nNo changes made."
            ))
        }
    };

    // Replace old item with new item
    repo.remove_by_handle(handle)?;
    repo.add(new_item);
    repo.write()?;

    println!("Updated item: {handle}");
    Ok(())
}

fn edit_db() -> Result<()> {
    // Get original db
    let db_path = Config::default().path;
    let original_db = std::fs::read_to_string(&db_path).unwrap_or_default();

    // Copy original db to tempfile
    let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
    tmp_file.write_all(original_db.as_bytes())?;

    // Edit new db
    let mut new_db = String::new();
    tmp_file.reopen()?.read_to_string(&mut new_db)?;
    new_db = edit::edit(new_db)?;

    // No changes, abort
    if new_db == original_db {
        println!("No changes.");
        return Ok(());
    }

    // TODO: Validate database

    // Save changes
    std::fs::write(db_path, new_db)?;
    println!("Database updated.");
    Ok(())
}
