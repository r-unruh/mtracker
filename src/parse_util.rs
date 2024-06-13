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
) -> Result<handle::Handle, Box<dyn std::error::Error>> {
    let identifier = matches.get_one::<String>("IDENTIFIER").unwrap().to_string();
    let result = match matches.try_get_one::<u16>("YEAR")? {
        Some(year) => format!("{identifier} ({year})"),
        None => identifier,
    };
    Ok(handle::Handle::from_user_input(result.as_str()))
}
