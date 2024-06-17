use clap::ArgMatches;

use crate::media::handle;

pub fn tags_from_matches(matches: &ArgMatches) -> Vec<&String> {
    match matches.get_many::<String>("TAG") {
        Some(t) => t.collect(),
        None => vec![],
    }
}

pub fn handle_from_matches(
    matches: &ArgMatches,
) -> Result<Option<handle::Handle>, Box<dyn std::error::Error>> {
    let user_input = match matches.try_get_one::<String>("IDENTIFIER")? {
        Some(i) => i.to_string(),
        None => {
            return Ok(None);
        }
    };

    let identifier = match matches.try_get_one::<u16>("YEAR")? {
        Some(year) => format!("{user_input} ({year})"),
        None => user_input,
    };

    Ok(Some(handle::Handle::from_user_input(identifier.as_str())))
}

pub fn note_from_matches(
    matches: &ArgMatches,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    match matches.try_get_one::<String>("NOTE")? {
        Some(note) => if note.contains('\n') {
            Err("note should be a single line".into())
        } else {
            Ok(Some(note.to_string()))
        },
        None => Ok(None),
    }
}
