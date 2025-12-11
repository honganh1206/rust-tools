use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

fn main() {
    // Interesting: When get_args() returns, it evaluates to Ok(config_value)
    // and tha config instance is passed as self to run(v)??
    // we can roughly define and_then as
    //
    //match self {
    //    Ok(v) => run(v),
    //    Err(e) => Err(e)
    //}
    //
    if let Err(e) = get_args().and_then(run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("Hong Anh Pham")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-") // For stdin
                .multiple(true),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Show word count")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Show byte count")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .help("Show character count")
                .takes_value(false)
                .conflicts_with("bytes"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .help("Show line count")
                .takes_value(false),
        )
        .get_matches();

    // Unpack the matching arguments
    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    // Iterator::all() method expects a closure (anon func works as a higher order func)
    // The anon function captures values from its surrouding scope,
    // and in this case those values are the flags?
    // BTW, we are comparing REFERENCES, not VALUES.
    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
        // If all four flags are false
        // set some of them to true
        // to handle the case where no flag is used
        lines = true;
        words = true;
        bytes = true;
        // No chars flag set
        // since setting it will conflict with --bytes
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if let Ok(info) = count(file) {
                    println!(
                        "{}{}{}{}{}",
                        format_field(info.num_lines, config.lines),
                        format_field(info.num_words, config.words),
                        format_field(info.num_chars, config.chars),
                        format_field(info.num_bytes, config.bytes),
                        if filename == "-" {
                            // Stdin
                            "".to_string()
                        } else {
                            format!(" {}", filename)
                        }
                    )
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        // Read from stdin
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        // Open a physical file
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// Explicitly declare that the following data structure must implement these traits
#[derive(Debug, PartialEq)]
struct FileInfo {
    // usize is more dynamic than other unsigned integer types?
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

// Possiblyr return a FileInfo struct
// because this might involve I/O
fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;
    let mut line = String::new();

    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }
        num_bytes += line_bytes;
        num_lines += 1;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        // Prepare for next line iteration
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

// Only compile when testing
#[cfg(test)]
// Separate module
mod tests {
    use super::{FileInfo, count, format_field};
    // In-memory buffer to fake a filehandle for tests
    // For production, use File::open
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        // The spaces are necessary.
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
