use clap::{ArgMatches, Command};

use crate::arg_util;
use crate::args;
use crate::media::repo;

pub fn command() -> Command {
    Command::new("add")
        .visible_aliases(["a"])
        .about("Add new item and/or tag an existing item")
        .arg_required_else_help(true)
        .arg(args::identifier())
        .arg(args::tag().help("tag(s) to add"))
        .arg(args::year())
        .arg(args::note())
}

pub fn handle(
    repo: &mut repo::Repo,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = arg_util::handle_from_matches(matches)?.unwrap();
    let tags = arg_util::tags_from_matches(matches);
    let note = matches.try_get_one::<String>("NOTE")?;

    // Report error when just adding an existing item
    let media = repo.get(&handle);
    if media.is_some() && tags.is_empty() {
        return Err(format!("item already exists: {handle}").into());
    }

    let media = match media {
        Some(m) => m,
        None => repo.get_or_create(&handle),
    };

    // Add tags
    for tag in tags {
        if media.has_tag(tag) {
            eprintln!("Tag already exists: {tag}");
        } else {
            media.add_tag(tag);
            println!("Added tag to {handle}: {tag}");
        }
    }

    // Add note
    if note.is_some() {
        if media.note.is_empty() {
            media.note = note.unwrap().to_string();
            println!("Added note to {handle}: {}", media.note);
        } else {
            return Err(format!(
                "item already has a note. Run 'mtracker edit \"{handle}\"' to modify it."
            )
            .into());
        }
    }

    repo.write()
}
