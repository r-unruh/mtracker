//! Dataset downloads.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::{anyhow, Context, Result};

pub const BASE_URL: &str = "https://datasets.imdbws.com/";

/// Path of the cached `<name>.tsv.gz`, downloading it if missing or `force`
pub fn ensure_dataset(dir: &Path, name: &str, force: bool) -> Result<PathBuf> {
    let file_name = format!("{name}.tsv.gz");
    let path = dir.join(&file_name);

    if path.exists() && !force {
        eprintln!("Using cached {file_name} ({})", age_string(&path));
        return Ok(path);
    }

    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let url = format!("{BASE_URL}{file_name}");
    let part = dir.join(format!("{file_name}.part"));

    let result = download(&url, &part, &file_name);
    if result.is_err() {
        fs::remove_file(&part).ok();
    }
    result.with_context(|| format!("failed to download {url}"))?;

    fs::rename(&part, &path).with_context(|| format!("failed to move {file_name} into place"))?;
    Ok(path)
}

fn download(url: &str, dest: &Path, label: &str) -> Result<()> {
    let response = ureq::get(url).call()?;
    let (parts, body) = response.into_parts();
    if !parts.status.is_success() {
        return Err(anyhow!("server responded with {}", parts.status));
    }
    let total: Option<u64> = parts
        .headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());

    let mut reader = body.into_reader();
    let mut file = fs::File::create(dest)?;
    let mut buf = vec![0u8; 1 << 16];
    let mut done: u64 = 0;
    let mut last_report: u64 = 0;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        done += n as u64;
        if done - last_report >= 4 << 20 {
            last_report = done;
            report(label, done, total);
        }
    }
    file.flush()?;

    if let Some(t) = total {
        if done != t {
            return Err(anyhow!("connection closed after {done} of {t} bytes"));
        }
    }
    report(label, done, total);
    eprintln!();
    Ok(())
}

fn report(label: &str, done: u64, total: Option<u64>) {
    let mb = |b: u64| b as f64 / 1_048_576.0;
    match total {
        Some(t) => eprint!("\rDownloading {label}: {:.0} / {:.0} MB", mb(done), mb(t)),
        None => eprint!("\rDownloading {label}: {:.0} MB", mb(done)),
    }
}

/// "updated today", "3 days old", ...
fn age_string(path: &Path) -> String {
    let days = fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_secs() / 86_400);
    match days {
        Some(0) => "downloaded today".into(),
        Some(1) => "1 day old".into(),
        Some(d) => format!("{d} days old"),
        None => "age unknown".into(),
    }
}
