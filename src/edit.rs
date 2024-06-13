use clap::Command;
use std::{
    env::{temp_dir, var},
    fs, process,
};

use crate::media::repo;

pub fn command() -> Command {
    clap::Command::new("edit")
        .aliases(["e"])
        .about("Edit database with the default editor")
        .arg_required_else_help(false)
}

pub fn handle(repo: &mut repo::Repo) -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary config file
    let mut file_path = temp_dir();
    file_path.push("mtracker_db.txt");
    fs::copy(&repo.path, &file_path)?;

    // Launch default editor
    let editor = var("EDITOR")?;
    process::Command::new(editor).arg(&file_path).status()?;

    // Overwrite original config file
    fs::copy(&file_path, &repo.path)?;

    Ok(())
}
