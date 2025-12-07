fn main() {
    if let Err(e) = catr::get_args().and_then(catr::run) {
        // Print from stderr
        eprintln!("{}", e);
        std::process::exit(1);
    }
    // Exit with code 0 by default, no need to specify
}
