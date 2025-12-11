// Filter out adjacent duplicate lines
// by collapsing adjacent identical lines into a single line
/*

From

apple
apple
banana
apple
banana
banana

to

apple
banana
apple
banana

* */
use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    // Output file is optional
    out_file: Option<String>,
    count: bool,
}

fn main() {
    if let Err(e) = get_args().and_then(run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Hong Anh Pham")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output file"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .help("Show counts")
                .long("count")
                .takes_value(false),
        )
        .get_matches();

    Ok(Config {
        // Alternatives
        // 1. Apply String::from to the file
        // (however this sounds dumb, since there is only 1 value)
        // in_file: matches.value_of_lossy("in_file").map(String::from).unwrap(),
        // Or use an anon function as a closure
        // since Rust can infer the type of v
        // in_file: matches.value_of_lossy("in_file").map(|v| v.into()).unwrap(),
        in_file: matches.value_of_lossy("in_file").unwrap().to_string(),
        // Apply from() that converts a str to &str (value to ref)
        // and store result on heap
        // Alternative with anon function as closure
        // out_file: matches.value_of("out_file").map(|v| v.to_string()),
        out_file: matches.value_of("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}

fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut previous = String::new();
    // TIP: No need to declare a type
    // since Rust can infer it?
    let mut count: u64 = 0;
    loop {
        // Append delimiter to buffer,
        // thus preserving line ending?
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            // Done reading
            break;
        }

        // Calculate adjacent duplicate lines
        if line.trim_end() != previous.trim_end() {
            // Encounter non-duplicate line,
            // so we copy it for later comparison
            // and reset the counter
            if count > 0 {
                // After calculating duplicate adjacent lines
                print!("{:>4} {}", count, previous);
            }
            previous = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    if count > 0 {
        print!("{:>4} {}", count, previous);
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
