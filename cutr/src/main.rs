use crate::Extract::*;
use anyhow::Result;
use clap::{App, Arg};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;
use std::{error::Error, ops::Range};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
};

type MyResult<T> = Result<T, Box<dyn Error>>;
// Array of range values e.g., 1..3
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8, // Single byte
    extract: Extract,
}

// Cut out selected portion of each line,
// specified by lists in args
fn main() {
    if let Err(e) = get_args().and_then(run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Hong Anh Pham")
        .about("Rust cut")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            // Tell us where one field ends and next field begins
            Arg::with_name("delimiter")
                .value_name("DELIMITER")
                .short("d")
                .long("delim")
                .help("Field delimiter")
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .short("f")
                .long("fields")
                .help("Selected fields")
                .conflicts_with_all(&["chars", "bytes"]),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("b")
                .long("bytes")
                .help("Selected bytes")
                .conflicts_with_all(&["fields", "chars"]),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("c")
                .long("chars")
                .help("Selected characters")
                .conflicts_with_all(&["fields", "bytes"]),
        )
        .get_matches();

    let delimiter = matches.value_of("delimiter").unwrap();
    // why to bytes?
    let delim_bytes = delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            delimiter
        )));
    }
    let fields = matches.value_of("fields").map(parse_pos).transpose()?;
    let bytes = matches.value_of("bytes").map(parse_pos).transpose()?;
    let chars = matches.value_of("chars").map(parse_pos).transpose()?;

    // Figure out which variant to create or generate an error
    // if the user fails to select bytes, chars, or fields
    let extract = if let Some(field_pos) = fields {
        Fields(field_pos)
    } else if let Some(byte_pos) = bytes {
        Bytes(byte_pos)
    } else if let Some(char_pos) = chars {
        Chars(char_pos)
    } else {
        // Convert from Box type to string?
        return Err(From::from("Must have --fields, --bytes, or --chars"));
    };

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        // Are we borrowing value of delim_bytes?
        delimiter: *delim_bytes.first().unwrap(),
        extract,
    })
}

fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => match &config.extract {
                Fields(field_pos) => {
                    // Build CSV reader
                    let mut reader = ReaderBuilder::new()
                        .delimiter(config.delimiter)
                        .has_headers(false)
                        .from_reader(file);

                    let mut writer = WriterBuilder::new()
                        .delimiter(config.delimiter)
                        .from_writer(io::stdout());

                    for record in reader.records() {
                        // Unwrap result since records() return Result as an iterator iterator
                        let record = record?;
                        writer.write_record(extract_fields(&record, field_pos))?
                    }
                }
                Bytes(byte_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, byte_pos));
                    }
                }
                Chars(char_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, char_pos));
                    }
                }
            },
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    // Regex to match two integers separated by a dash e.g., 1-4
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();

    range
        .split(',')
        // Iterator over comma-separated position expressions like "1" or "1-4"
        .map(|val| {
            parse_index(val)
                // Single index like "1" becomes a one-element range (0-based)
                .map(|n| n..n + 1)
                // If single-index parsing fails, try parsing a hyphenated range like "1-4"
                .or_else(|e| {
                    // If not a single index,
                    // check whether it matches the range pattern with captures();
                    // otherwise propagate the original parse error
                    range_re.captures(val).ok_or(e).and_then(|captures| {
                        let n1 = parse_index(&captures[1])?;
                        let n2 = parse_index(&captures[2])?;
                        if n1 >= n2 {
                            return Err(format!(
                                "First number in range ({}) \
                                must be lower than second number ({})",
                                n1 + 1,
                                n2 + 1
                            ));
                        }
                        // Valid range
                        Ok(n1..n2 + 1)
                    })
                })
        })
        // Gather values as a Result
        .collect::<Result<_, _>>()
        // Since Rust does not automatically change error types
        // We need to convert e from Err(e) to our custom error type
        // which is Box<dyn Error>
        .map_err(From::from)
}

// Parse the string into a positive index,
// the index will be one less than the given number,
// since Rust needs zero-offset indexes (similar to others?)
fn parse_index(input: &str) -> Result<usize, String> {
    // Why a closure/anon func here?
    let value_error = || format!("illegal first value: \"{}\"", input);
    input
        .starts_with('+')
        // Check invalid first value
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                // Convert from non-zero usize to usize explicitly,
                // since Rust does not do implicit numeric conversion
                // even when the value is guaranteed to fit.
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
        })
}

// Return a new string composed of characters at the given index positions
// char_pos is a slice (view of a vector) containing a range here
fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    // Type annotation is required since collect() can return different types.
    // Rust can infer the vector type here.
    let chars: Vec<_> = line.chars().collect();

    // # 1st approach
    //let mut selected: Vec<char> = vec![];
    //
    // We need to do clone() here
    // since we have an iterator over &[Range<usize>] - Slice of references to ranges
    // but we need to iterate over [Range<uszie>]
    //for range in char_pos.iter().cloned() {
    //    for i in range {
    //        if let Some(val) = chars.get(i) {
    //            // De-reference the value here
    //            // as selected accepts elements of type char and not &char
    //            selected.push(*val)
    //        }
    //    }
    //}
    //selected.iter().collect()

    // 2nd approach: Avoid mutability and focus on shorter functions
    char_pos
        // Return an iterator of references, but we cannot iterate over references
        .iter()
        // so instead we clone the iterator to an iterator of values
        .cloned()
        // Filter out None and unwrap Some(&char)
        .flat_map(|range| range.filter_map(|i| chars.get(i)))
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        // Methods like cloned() or copied() aim to turn iterator/collection of references
        // to iterator/collection of values
        // Since from_utf8_lossy expects a slice of bytes
        // we need to convert it from  vector to slice via copied()
        // Also we do filtering out None and unwrap Some(&usize) here
        .flat_map(|range| range.filter_map(|i| bytes.get(i).copied()))
        .collect();

    // Potential problem that byte selection breaks Unicode chars
    // thus producing invalid UTF-8 string
    String::from_utf8_lossy(&selected)
        // Transfer ownership to the caller of this method
        .into_owned()
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    field_pos
        .iter()
        .cloned()
        // Here we have a slice of strings?
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        // Shorthand conversion from usize to String?
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::extract_bytes;
    use super::extract_chars;
    use super::extract_fields;
    use super::parse_pos;
    use csv::StringRecord;

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_parse_pos() {
        // Empty string => error
        assert!(parse_pos("").is_err());

        // Zero => error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list values: \"0\"");

        // Start with 0 then error
        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);
        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);
        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());
        let res = parse_pos(",");
        assert!(res.is_err());
        let res = parse_pos("1,");
        assert!(res.is_err());
        let res = parse_pos("1-");
        assert!(res.is_err());
        let res = parse_pos("1-1-1");
        assert!(res.is_err());
        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );
        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}
