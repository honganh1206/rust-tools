use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

// Type alias
type MyResult<T> = Result<T, Box<dyn Error>>;

// Add the Debug trait
// so the struct can use print method?
#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    // For quick and dirty debugging
    //dbg!(config);
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            // There is stdin?
            Ok(file) => {
                let mut last_num = 0;
                // Using lines() means we replace Windows CRLF
                // with Unix-style newlines
                for (line_num, line_result) in file.lines().enumerate() {
                    let line = line_result?;
                    // Print with line numbers if want
                    if config.number_lines {
                        println!("{:>6}\t{}", line_num + 1, line);
                    } else if config.number_nonblank_lines {
                        // Apply line numbers for non-blank lines
                        if !line.is_empty() {
                            last_num += 1;
                            println!("{:>6}\t{}", last_num, line);
                        } else {
                            println!();
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Hong Anh Pham")
        .about("Rust cat")
        .arg(
            Arg::with_name("files")
                // Cosmetic purpose
                .value_name("FILE")
                .help("Input file(s)")
                // Accept multiple files
                .multiple(true)
                // No default?
                .default_value("-"),
        )
        .arg(
            // Flag to print line numbers
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Number of lines")
                .takes_value(false)
                .conflicts_with("number_nonblank"),
        )
        .arg(
            // Flag to print line numbers for non-blank lines
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Number of non-blank lines")
                .takes_value(false),
        )
        .get_matches();

    // Validate the arguments
    Ok(Config {
        // Clap replaces invalid UTF-8 characters with \u{FFFD}
        // to ensure valid string
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number"),
        number_nonblank_lines: matches.is_present("number_nonblank"),
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    // Similar to switch statement in other languages
    match filename {
        // Either stdin or stdout, so we read directly from stdin instead of physical file
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        // Default case, read from physical file
        // by open() returning a filehandle to read contents of a file,
        // which a buffered reader will receive,
        // and wrapped by a smart pointer Box.
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
