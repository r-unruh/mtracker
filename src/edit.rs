use clap::{ArgMatches, Command};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    env::{temp_dir, var},
    fs,
    path::PathBuf,
    process,
};

use crate::arg_util;
use crate::args;
use crate::media::{handle, repo, Media};

pub fn command() -> Command {
    clap::Command::new("edit")
        .aliases(["e"])
        .about("Edit item or whole database with the default editor")
        .arg(args::identifier().required(false))
        .arg(args::year())
        .arg_required_else_help(false)
}

pub fn handle(
    repo: &mut repo::Repo,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(handle) = arg_util::handle_from_matches(matches)? {
        edit_db_entry(repo, &handle)
    } else {
        edit_db(repo)
    }
}

fn edit_db_entry(
    repo: &mut repo::Repo,
    handle: &handle::Handle,
) -> Result<(), Box<dyn std::error::Error>> {
    // Init repo
    repo.read()?;

    // Find media
    let Some(item) = repo.get(handle) else {
        return Err(format!("item not found: {handle}").into());
    };

    // Create temporary file and fill it with this media's data
    let tmp_file_path = get_tmp_file_path();
    let mut db_entry = item.to_db_entry();
    fs::write(&tmp_file_path, db_entry)?;

    // Edit file manually
    launch_editor(&tmp_file_path)?;

    // Reread media data after manual editing
    db_entry = fs::read_to_string(&tmp_file_path)?.trim().to_string();

    // Create new item based on db entry
    let new_item = match Media::from_db_entry(&db_entry) {
        Ok(item) => item,
        Err(e) => {
            return Err(format!(
                "failed to edit {handle}: {e}\n\nYour input:\n{db_entry}\n\nNo changes made."
            )
            .into())
        }
    };

    // Replace old item with new item
    repo.remove_by_handle(handle)?;
    repo.add(new_item);
    repo.write()?;

    // Clean up
    fs::remove_file(&tmp_file_path)?;

    println!("Updated item: {handle}");
    Ok(())
}

fn edit_db(repo: &mut repo::Repo) -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary file by copying database
    let tmp_file_path = get_tmp_file_path();
    fs::copy(&repo.path, &tmp_file_path)?;

    // Edit file manually
    launch_editor(&tmp_file_path)?;

    // TODO: Validate database

    // Overwrite original config file
    fs::copy(&tmp_file_path, &repo.path)?;

    // Clean up
    fs::remove_file(&tmp_file_path)?;

    println!("Database updated.");
    Ok(())
}

fn launch_editor(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    process::Command::new(var("EDITOR")?).arg(path).status()?;
    Ok(())
}

fn get_tmp_file_path() -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut tmp_file_path = temp_dir();
    tmp_file_path.push(format!("mtracker_{now}.txt"));
    tmp_file_path
}
