fn main() {
    if let Err(e) = mtracker::run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
