use clap::Arg;

pub fn identifier() -> Arg {
    Arg::new("IDENTIFIER")
        .required(true)
        .help("\"name (year)\" or \"name\"")
        .long_help(
            "The name (and optionally year of release) of the movie / series.

Example: \"The Movie (2024)\" or just \"The Movie\"",
        )
}

pub fn year() -> Arg {
    Arg::new("YEAR")
        .required(false)
        .short('y')
        .value_parser(clap::value_parser!(u16))
        .long("year")
        .help("Specify year of release")
}

pub fn term() -> Arg {
    Arg::new("TERM")
        .required(false)
        .trailing_var_arg(true)
        .num_args(0..)
}

pub fn tag() -> Arg {
    Arg::new("TAG")
        .required(false)
        .num_args(0..)
        .short('t')
        .value_delimiter(',')
        .long("tag")
}

pub fn note() -> Arg {
    Arg::new("NOTE")
        .required(false)
        .short('n')
        .long("note")
        .help("A short, single-line note")
        .long_help(
            "A short, single-line note

Examples:
- Recommended by Max
- Too many jumpscares",
        )
}

pub fn note_bool() -> Arg {
    Arg::new("NOTE")
        .required(false)
        .value_parser(clap::value_parser!(bool))
        .num_args(0)
        .short('n')
        .long("note")
}

pub fn tags_bool() -> Arg {
    Arg::new("TAGS")
        .required(false)
        .value_parser(clap::value_parser!(bool))
        .num_args(0)
        .short('g')
        .long("tags")
}
