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
    io::{self, BufRead, BufReader, Write},
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

    // Create output file with either File::create or stdout
    // Fun fact: Both File::create and io::stdout implement Write trait
    // so they both satisfy Box<dyn Write>
    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(io::stdout()),
    };

    // WE USE A CLOSURE :)
    // Now I get it: Closure is an anon func that accepts vars from its enclosing env.
    // Btw Rust's syntax for closure is weird IMO.
    let mut print = |count: u64, text: &str| -> MyResult<()> {
        // Accepting count from outer env here
        if count > 0 {
            if config.count {
                // why borrowed here???
                write!(out_file, "{:>4} {}", count, text)?;
            } else {
                write!(out_file, "{}", text)?;
            }
        }

        Ok(())
    };

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

            print(count, &previous)?;
            previous = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    print(count, &previous)?;
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
