extern crate dirs;

use mtracker::media::repo;

fn main() {
    // Get database
    let db_file_path = match get_or_create_user_dir() {
        Ok(mut path) => {
            path.push("db.txt");
            path
        }
        Err(e) => {
            return exit_with_error(&e);
        }
    };

    // Init repo
    let mut repo = match repo::Repo::load_or_create(&db_file_path) {
        Ok(r) => r,
        Err(e) => {
            return exit_with_error(&e);
        }
    };

    // Run program
    if let Err(e) = mtracker::run(&mut repo) {
        exit_with_error(&e);
    }
}

fn get_or_create_user_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    if let Some(mut dir) = dirs::data_dir() {
        dir.push("mtracker");
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    } else {
        Err("failed to find user data directory".into())
    }
}

fn exit_with_error(error: &Box<dyn std::error::Error>) {
    eprintln!("{error}");
    std::process::exit(1);
}
