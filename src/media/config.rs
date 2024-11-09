use std::path::PathBuf;

use clap::crate_name;

pub struct Config {
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let mut db_file_path = dirs::data_dir().expect("failed to get user data directory");
        db_file_path.push(format!("{}/db.txt", crate_name!()));
        Self { path: db_file_path }
    }
}
