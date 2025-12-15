use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    // Find all files in a directory that contain matching text
    recursive: bool,
    // Summary number of times a match occurs
    count: bool,
    // Find lines that don't match patterns
    invert_match: bool,
}

fn main() {
    if let Err(e) = get_args().and_then(run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Hong Anh Pham")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Search pattern")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count occurrences")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("invert")
                .short("v")
                .long("invert-match")
                .help("Invert match")
                .takes_value(false),
        )
        .get_matches();

    let pattern = matches.value_of("pattern").unwrap();
    let pattern = RegexBuilder::new(pattern)
        .case_insensitive(matches.is_present("insensitive"))
        .build() // Compile the regex to Regex type
        .map_err(|_| format!("Invalid pattern \"{}\"", pattern))?;

    Ok(Config {
        pattern,
        // May contain invalid UTF-8 chars as bytes?
        files: matches.values_of_lossy("files").unwrap(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert"),
    })
}

fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

// We put the String object inside an array literal here,
// and its runtime? type is [String, 1]
fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{Rng, distributions::Alphanumeric};

    #[test]
    fn test_find_files() {
        // Accept a file input when we know it exists
        // When we write a literal string,
        // its type is inferred to be a reference to a static string
        // so we need to convert it to an owned, heap-allocated String object
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // Reject a dir input without recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify recursive option work
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<string> = res
            .iter()
            // Convert the value wrapped byOk inside &Result to &result
            .map(|r| r.as_ref().unwrap().replace("\\", "/")) // Replace Windows way of slashing?
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            // Vectorize stuff!
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );
        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
