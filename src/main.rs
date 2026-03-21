fn main() {
    if let Err(error) = ota::cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
