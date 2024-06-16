use clap::Arg;

pub fn identifier() -> Arg {
    Arg::new("IDENTIFIER")
        .required(true)
        .help("\"name (year)\" or \"name\"")
}

pub fn year() -> Arg {
    Arg::new("YEAR")
        .required(false)
        .short('y')
        .value_parser(clap::value_parser!(u16))
        .long("year")
        .help("specify year of release")
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
