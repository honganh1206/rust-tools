use clap::{App, Arg};

fn main() {
    let matches = App::new("echor")
        .version("0.1.0") // Semantic versioning
        .author("Hong Anh Pham")
        .about("Rust echo")
        .arg(
            // Positional argument must appear at least once and can be repeated
            Arg::with_name("text")
                .value_name("TEXT")
                .help("Input text")
                .required(true)
                .min_values(1), // Expect more than 1 arg
        )
        .arg(
            Arg::with_name("omit_newline")
                .short("n")
                .help("Do not print new line")
                .takes_value(false), // Take no arg
        )
        .get_matches(); // Tell Command to parse args;

    // Placeholder to pretty print the args
    //println!("{:#?}", matches);

    // Fetch inputs to generate output like echo does
    let text = matches.values_of_lossy("text").unwrap();
    let omit_newline = matches.is_present("omit_newline");

    let mut ending = "\n";
    if omit_newline {
        ending = "";
    }
    print!("{}{}", text.join(" "), ending);
}
