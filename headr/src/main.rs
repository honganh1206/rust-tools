fn main() {
    if let Err(e) = headr::get_args().and_then(headr::run) {
        // Print from stderr
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
