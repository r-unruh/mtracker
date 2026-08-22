use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use anyhow::{anyhow, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;

use crate::{
    arg_util, args,
    imdb::{
        self, dataset, download,
        matcher::{self, MatchKind},
        meta, names, Meta, TitleType,
    },
    media::{handle::Handle, repo::Repo, Media},
};

const BASICS_HEADER: [&str; 9] = [
    "tconst",
    "titleType",
    "primaryTitle",
    "originalTitle",
    "isAdult",
    "startYear",
    "endYear",
    "runtimeMinutes",
    "genres",
];
const RATINGS_HEADER: [&str; 3] = ["tconst", "averageRating", "numVotes"];
const CREW_HEADER: [&str; 3] = ["tconst", "directors", "writers"];
const NAMES_HEADER: [&str; 6] = [
    "nconst",
    "primaryName",
    "birthYear",
    "deathYear",
    "primaryProfession",
    "knownForTitles",
];

pub fn command() -> Command {
    Command::new("sync")
        .about("Link items to IMDb and fetch genres, ratings and directors")
        .long_about(
            "Link items to IMDb and fetch genres, ratings and directors

Downloads IMDb's public datasets (no account or API key needed, ~600 MB once)
into the cache directory, matches every unlinked item by name and year and
stores the IMDb id in the database as 'imdb: tt1234567'. Genres, IMDb rating
and directors are kept in a local cache and shown by 'ls' and the TUI.

Items that already have an 'imdb' id are never re-matched; to fix a wrong
match, edit the id by hand.",
        )
        .arg_required_else_help(false)
        .arg(args::identifier().required(false).help("Only sync this item"))
        .arg(args::year())
        .arg(
            Arg::new("DOWNLOAD")
                .long("download")
                .action(ArgAction::SetTrue)
                .help("Re-download the IMDb datasets even if they are cached"),
        )
        .arg(
            Arg::new("DRY_RUN")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Show what would be linked without writing anything"),
        )
        .arg(
            Arg::new("MIN_VOTES")
                .long("min-votes")
                .value_parser(clap::value_parser!(u32))
                .default_value("1000")
                .help("Titles with at least this many IMDb votes go into the TUI search catalog"),
        )
}

pub struct Options {
    pub force_download: bool,
    pub dry_run: bool,
    pub min_votes: u32,
    /// Only sync this item
    pub item: Option<Handle>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            force_download: false,
            dry_run: false,
            min_votes: 1000,
            item: None,
        }
    }
}

pub fn handle(matches: &ArgMatches) -> Result<()> {
    let mut repo = arg_util::repo_from_matches(matches)?;
    let opts = Options {
        force_download: matches.get_flag("DOWNLOAD"),
        dry_run: matches.get_flag("DRY_RUN"),
        min_votes: *matches.get_one::<u32>("MIN_VOTES").unwrap(),
        item: arg_util::handle_from_matches(matches)?,
    };
    run(&mut repo, &opts)
}

/// Link items to IMDb, refresh the metadata cache and rebuild the catalog
pub fn run(repo: &mut Repo, opts: &Options) -> Result<()> {
    let force_download = opts.force_download;
    let dry_run = opts.dry_run;
    let min_votes = opts.min_votes;

    // Which items to sync
    let targets: Vec<usize> = match &opts.item {
        Some(handle) => {
            let idx = (0..repo.len())
                .find(|&i| repo.get_by_index(i).matches_handle(handle))
                .ok_or_else(|| anyhow!("item not found: {handle}"))?;
            vec![idx]
        }
        None => (0..repo.len()).collect(),
    };

    // Unlinked items are matched by normalized name; linked ones are refreshed by id
    let mut wanted: HashMap<String, Vec<usize>> = HashMap::new();
    let mut known: HashSet<String> = HashSet::new();
    for &i in &targets {
        let item = repo.get_by_index(i);
        match &item.imdb {
            Some(id) => {
                known.insert(id.clone());
            }
            None => wanted.entry(matcher::normalize(&item.name)).or_default().push(i),
        }
    }
    // Datasets
    let cache_dir = imdb::cache_dir()?;
    let data_dir = cache_dir.join("imdb");
    let basics = download::ensure_dataset(&data_dir, "title.basics", force_download)?;
    let ratings = download::ensure_dataset(&data_dir, "title.ratings", force_download)?;
    let crew = download::ensure_dataset(&data_dir, "title.crew", force_download)?;
    let names = download::ensure_dataset(&data_dir, "name.basics", force_download)?;

    // Pass 1: ratings. Popular titles form the search catalog.
    let started = Instant::now();
    eprint!("Scanning ratings...");
    let mut popular: HashMap<String, (Option<u8>, u32)> = HashMap::new();
    let mut skipped = 0;
    dataset::for_each_line(&ratings, &RATINGS_HEADER, |line| {
        let Some(f) = dataset::fields(line, RATINGS_HEADER.len()) else {
            skipped += 1;
            return Ok(());
        };
        let votes: u32 = f[2].parse().unwrap_or(0);
        if votes >= min_votes {
            popular.insert(f[0].to_string(), (parse_rating(f[1]), votes));
        }
        Ok(())
    })?;
    warn_skipped(&ratings, skipped);
    eprintln!(" {:.1}s", started.elapsed().as_secs_f32());

    // Pass 2: titles. Keep popular titles, candidates and known ids.
    let started = Instant::now();
    eprint!("Scanning titles...");
    let mut metas: HashMap<String, Meta> = HashMap::new();
    let mut cand_index: HashMap<String, Vec<String>> = HashMap::new();
    let mut skipped = 0;
    dataset::for_each_line(&basics, &BASICS_HEADER, |line| {
        // Check type before splitting the line: drops most rows (episodes) cheaply
        let mut head = line.splitn(3, '\t');
        let (Some(tconst), Some(title_type)) = (head.next(), head.next()) else {
            skipped += 1;
            return Ok(());
        };
        let is_known = known.contains(tconst);
        let pop = popular.get(tconst).copied();
        let Some(title_type) = TitleType::from_imdb(title_type) else {
            return Ok(());
        };
        let Some(f) = dataset::fields(line, BASICS_HEADER.len()) else {
            skipped += 1;
            return Ok(());
        };
        if f[4] == "1" {
            return Ok(()); // adult
        }

        let mut keys: Vec<String> = vec![];
        if !wanted.is_empty() {
            let primary = matcher::normalize(f[2]);
            if wanted.contains_key(&primary) {
                keys.push(primary.clone());
            }
            if f[3] != f[2] {
                let original = matcher::normalize(f[3]);
                if original != primary && wanted.contains_key(&original) {
                    keys.push(original);
                }
            }
        }
        if !is_known && keys.is_empty() && pop.is_none() {
            return Ok(());
        }
        let (rating, votes) = pop.unwrap_or((None, 0));

        for key in keys {
            cand_index.entry(key).or_default().push(tconst.to_string());
        }
        metas.insert(
            tconst.to_string(),
            Meta {
                tconst: tconst.to_string(),
                title_type,
                primary_title: f[2].to_string(),
                original_title: f[3].to_string(),
                year: f[5].parse().ok(),
                runtime: f[7].parse().ok(),
                genres: f[8].split(',').filter(|g| !g.is_empty()).map(str::to_lowercase).collect(),
                rating,
                votes,
                directors: vec![],
            },
        );
        Ok(())
    })?;
    warn_skipped(&basics, skipped);
    eprintln!(" {:.1}s", started.elapsed().as_secs_f32());

    // Pass 3: ratings for the candidates and known ids that are not popular
    if metas.values().any(|m| !popular.contains_key(&m.tconst)) {
        let mut skipped = 0;
        dataset::for_each_line(&ratings, &RATINGS_HEADER, |line| {
            let Some((tconst, _)) = line.split_once('\t') else {
                skipped += 1;
                return Ok(());
            };
            if popular.contains_key(tconst) {
                return Ok(());
            }
            if let Some(m) = metas.get_mut(tconst) {
                let Some(f) = dataset::fields(line, RATINGS_HEADER.len()) else {
                    skipped += 1;
                    return Ok(());
                };
                m.rating = parse_rating(f[1]);
                m.votes = f[2].parse().unwrap_or(0);
            }
            Ok(())
        })?;
        warn_skipped(&ratings, skipped);
    }

    // Resolve matches, in database order
    let mut order: Vec<(usize, &String)> = wanted
        .iter()
        .flat_map(|(key, idxs)| idxs.iter().map(move |&i| (i, key)))
        .collect();
    order.sort();

    let mut linked: HashSet<String> = known.clone();
    let mut matched: Vec<(usize, String)> = vec![];
    let mut misses: Vec<usize> = vec![];
    for (i, key) in order {
        let item = repo.get_by_index(i);
        let cands: Vec<&Meta> = cand_index
            .get(key)
            .map(|ids| ids.iter().map(|id| &metas[id]).collect())
            .unwrap_or_default();
        match matcher::select(item.year, &cands) {
            Some((m, kind)) => {
                let warning = match kind {
                    MatchKind::YearMismatch => "  WARNING: year differs".yellow().to_string(),
                    MatchKind::Exact | MatchKind::NearYear | MatchKind::NoYear => String::new(),
                };
                println!(
                    "{} -> {} {} [{}]{warning}",
                    label(item),
                    m.tconst.dimmed(),
                    m.describe(),
                    m.genres.join(", ")
                );
                linked.insert(m.tconst.clone());
                matched.push((i, m.tconst.clone()));
            }
            None => misses.push(i),
        }
    }

    for id in &known {
        if !metas.contains_key(id) {
            eprintln!("{}", format!("WARNING: {id} not found in the IMDb datasets").yellow());
        }
    }

    // Pass 4 + 5: directors of every kept title, then their names
    let started = Instant::now();
    eprint!("Scanning crew...");
    let mut director_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut wanted_names: HashSet<String> = HashSet::new();
    let mut skipped = 0;
    dataset::for_each_line(&crew, &CREW_HEADER, |line| {
        let Some((tconst, _)) = line.split_once('\t') else {
            skipped += 1;
            return Ok(());
        };
        if metas.contains_key(tconst) {
            let Some(f) = dataset::fields(line, CREW_HEADER.len()) else {
                skipped += 1;
                return Ok(());
            };
            let ids: Vec<String> =
                f[1].split(',').filter(|v| !v.is_empty()).map(str::to_string).collect();
            wanted_names.extend(ids.iter().cloned());
            director_ids.insert(tconst.to_string(), ids);
        }
        Ok(())
    })?;
    warn_skipped(&crew, skipped);
    eprintln!(" {:.1}s", started.elapsed().as_secs_f32());

    // Names are cached across runs; only scan name.basics for unknown people
    let names_path = cache_dir.join(names::FILE_NAME);
    let mut person_names = names::load(&names_path)?;
    let missing: HashSet<&str> = wanted_names
        .iter()
        .filter(|id| !person_names.contains_key(*id))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        let started = Instant::now();
        eprint!("Scanning names...");
        let mut skipped = 0;
        dataset::for_each_line(&names, &NAMES_HEADER, |line| {
            let Some((nconst, _)) = line.split_once('\t') else {
                skipped += 1;
                return Ok(());
            };
            if missing.contains(nconst) {
                let Some(f) = dataset::fields(line, NAMES_HEADER.len()) else {
                    skipped += 1;
                    return Ok(());
                };
                person_names.insert(nconst.to_string(), f[1].to_string());
            }
            Ok(())
        })?;
        warn_skipped(&names, skipped);
        eprintln!(" {:.1}s", started.elapsed().as_secs_f32());
        if !dry_run {
            names::save(&names_path, &person_names)?;
        }
    }
    for (tconst, ids) in director_ids {
        if let Some(m) = metas.get_mut(&tconst) {
            m.directors = ids.iter().filter_map(|id| person_names.get(id).cloned()).collect();
        }
    }

    // Write cache first, database last
    if dry_run {
        println!("\nDry run: nothing written.");
    } else {
        let meta_path = imdb::meta_path()?;
        let mut all = meta::load(&meta_path)?;
        for id in &linked {
            if let Some(m) = metas.get(id) {
                all.insert(id.clone(), m.clone());
            }
        }
        meta::save(&meta_path, &all)?;

        let mut catalog: Vec<Meta> =
            metas.values().filter(|m| popular.contains_key(&m.tconst)).cloned().collect();
        catalog.sort_by(|a, b| b.votes.cmp(&a.votes).then_with(|| a.tconst.cmp(&b.tconst)));
        meta::save_vec(&imdb::catalog_path()?, &catalog)?;

        for (i, tconst) in &matched {
            repo.get_by_index_mut(*i).imdb = Some(tconst.clone());
        }
        if !matched.is_empty() {
            repo.write()?;
        }
    }

    // Summary
    if !misses.is_empty() {
        println!("\n{}", "Not found on IMDb (check for typos):".bold());
        for i in &misses {
            println!("  {}", label(repo.get_by_index(*i)));
        }
    }
    println!(
        "\n{} linked, {} already linked, {} not found",
        matched.len(),
        known.len(),
        misses.len()
    );
    println!(
        "Catalog: {} titles with at least {min_votes} votes",
        metas.values().filter(|m| popular.contains_key(&m.tconst)).count()
    );
    println!("{}", imdb::ATTRIBUTION.dimmed());
    Ok(())
}

fn label(item: &Media) -> String {
    Handle {
        name: item.name.clone(),
        year: item.year,
    }
    .to_string()
}

/// "7.8" -> 78
fn parse_rating(s: &str) -> Option<u8> {
    let v: f64 = s.parse().ok()?;
    if (0.0..=10.0).contains(&v) {
        Some((v * 10.0).round() as u8)
    } else {
        None
    }
}

fn warn_skipped(path: &std::path::Path, skipped: usize) {
    if skipped > 0 {
        eprintln!(
            "{}",
            format!(
                "WARNING: skipped {skipped} malformed rows in {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
            .yellow()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rating() {
        assert_eq!(parse_rating("7.8"), Some(78));
        assert_eq!(parse_rating("10.0"), Some(100));
        assert_eq!(parse_rating("10"), Some(100));
        assert_eq!(parse_rating(""), None);
        assert_eq!(parse_rating("11"), None);
    }
}
