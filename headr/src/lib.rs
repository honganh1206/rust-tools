use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    // Pointer-sized unsigned integer type
    // varying from 4 bytes on 32-bit systems to 8 bytes on 64-bit systems
    lines: usize,
    bytes: Option<usize>,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(_) => println!("Opened {}", filename),
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    // Start the arg parsing process
    let matches = App::new("headr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rust head")
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .takes_value(true)
                .conflicts_with("lines")
                .help("Number of bytes"),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        // First unpack &str from Some
        // then apply function for each line num?
        .map(parse_positive_int)
        // Conversion between Result<> and Option<> nested types
        // like Option<Result<>> to Result<Option<>>
        .transpose()
        // Apply format macro to each error
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        // Field init shorthand, suggested by Clippy,
        // just how they did it in Go? :)
        bytes,
    })
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    // Parse val as string to another type (specified by return type of the function)
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        // Else conver string to Box<dyn Err>
        _ => Err(From::from(val)),
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        // Input from stdin
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        // Input from filehandle, which reads from the physical file
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// Best practice? to test private functions
#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any string is an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // A zero is an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
