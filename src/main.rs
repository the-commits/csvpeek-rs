use clap::{CommandFactory, Parser};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use atty;

fn parse_filter_arg(s: &str) -> Result<(String, String), String> {
    match s.split_once('=') {
        Some((key_str, value_str)) => {
            let key = key_str.trim();
            if key.is_empty() {
                Err(format!("Invalid filter format: Column name cannot be empty in '{}'. Expected COLUMN=VALUE.", s))
            } else {
                Ok((key.to_string(), value_str.trim().to_string()))
            }
        }
        None => {
            Err(format!("Invalid filter format. Expected COLUMN=VALUE, got '{}'", s))
        }
    }
}

const LONG_ABOUT: &str = "csvpeek-rs: Quickly Inspect and Process Your CSV Data from the Command Line

`csvpeek-rs` is a fast and flexible command-line utility, written in Rust, 
designed to make peeking into and processing CSV (Comma-Separated Values) 
files effortless directly from your terminal. Whether you need a quick 
glance at a large CSV, extract specific information, or prepare data for 
further command-line processing, `csvpeek-rs` offers a streamlined experience.

Core Functionalities:

* Versatile Data Input:
    * Process individual CSV files using the -f <file> flag.
    * Read data directly from stdin by specifying -f - or by piping 
        output from other commands.
    * Aggregate data from all .csv files within a specified directory 
        using the -d <directory> flag. `csvpeek-rs` intelligently handles 
        header matching, merging data from files with identical headers 
        and warning about those that differ.
    * If no input is specified and stdin is a terminal, `csvpeek-rs` 
        provides helpful usage instructions and exits.

* Flexible Data Display & Extraction:
    * List Mode (--list): Display rows from your CSV data. By default, 
        it shows the first column, but you can specify any column(s) using 
        --columns \"Column Name\" (or -c \"Col1,Col2\").
    * Random Row Selection: If no mode (like --list) is specified, 
        `csvpeek-rs` will pick and display a single random row (from the 
        chosen display column(s)), perfect for sampling data.
    * Customizable Display Column(s) (--columns): Choose exactly 
        which column's data you want to see for both listing and random selection.

* Powerful Filtering:
    * Precisely filter rows using the --filter \"COLUMN_NAME=VALUE\" syntax 
        (case-insensitive for both column name and value). This can be repeated
        for multiple AND-conditions.
    * Allows you to quickly drill down to the data you need.

* Unix-Friendly Output:
    * Raw Mode (--raw): Output only the data values, one per line, 
        without any headers, numbering, or informational messages. 
        This makes it ideal for piping the output of `csvpeek-rs` into 
        other standard Unix tools like grep, sort, awk, or for use in scripts.

`csvpeek-rs` aims to be a simple yet powerful addition to your command-line 
data toolkit, combining the performance of Rust with a user-friendly 
interface for common CSV operations.";

#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = LONG_ABOUT
)]
struct Args {
    /// Display the list (first column by default).
    #[clap(short, long, group = "mode")]
    list: bool,

    /// Filter the list based on COLUMN=VALUE. Can be repeated for multiple AND conditions.
    /// Used with --list.
    #[clap(long, value_parser = parse_filter_arg, requires = "list", num_args = 0..)] 
    filter: Option<Vec<(String, String)>>,

    /// Path to a single CSV data file. Use "-" to read from stdin.
    /// If neither -f nor -d is given, 'data.csv' is attempted or stdin if piped.
    #[clap(long, short = 'f')]
    data_file: Option<PathBuf>,

    /// Path to a directory containing CSV files to merge.
    /// Takes precedence over --data-file.
    #[clap(long, short = 'd')]
    directory: Option<PathBuf>,

    /// Specify column(s) to display. Use comma-separated values or repeat the flag.
    /// Defaults to the first column if not specified.
    #[clap(long = "columns", short = 'c', value_delimiter = ',')]
    columns: Option<Vec<String>>,

    /// Output raw data values only, one per line (for piping).
    #[clap(long)]
    raw: bool,
}

fn parse_csv_from_reader<R: Read>(reader_source: R) -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(reader_source);
    let headers = reader.headers()?.iter().map(String::from).collect::<Vec<String>>();
    if headers.is_empty() {
        return Err("CSV data is missing headers or is empty.".into());
    }
    let mut records_data = Vec::new();
    for result in reader.records() {
        let record = result?;
        let mut row_map = HashMap::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            row_map.insert(header.clone(), field.to_string());
        }
        records_data.push(row_map);
    }
    Ok((headers, records_data))
}

fn load_data_from_csv(filepath: &PathBuf) -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn Error>> {
    let file = fs::File::open(filepath)?;
    parse_csv_from_reader(file)
}

fn load_data_from_stdin() -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn Error>> {
    let stdin = io::stdin();
    parse_csv_from_reader(stdin.lock())
}

fn load_data_from_directory(dir_path: &PathBuf, be_quiet: bool) -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn Error>> {
    let mut master_headers: Option<Vec<String>> = None;
    let mut combined_records: Vec<HashMap<String, String>> = Vec::new();
    let mut files_processed = 0;
    let mut csv_file_paths: Vec<PathBuf> = Vec::new();

    for entry_result in fs::read_dir(dir_path)? {
        let entry = entry_result?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "csv" {
                    csv_file_paths.push(path);
                }
            }
        }
    }
    csv_file_paths.sort();

    for path in csv_file_paths {
        if !be_quiet {
            println!("Reading file: {}", path.display());
        }
        match load_data_from_csv(&path) {
            Ok((current_headers, current_records)) => {
                if master_headers.is_none() {
                    master_headers = Some(current_headers);
                    combined_records.extend(current_records);
                    files_processed += 1;
                } else if Some(&current_headers) == master_headers.as_ref() {
                    combined_records.extend(current_records);
                    files_processed += 1;
                } else {
                    if !be_quiet {
                        eprintln!("Warning: Headers in file '{}' do not match the headers of previously read files. Skipping this file.", path.display());
                        if let Some(mh) = &master_headers {
                             eprintln!("Expected headers: {:?}", mh);
                             eprintln!("Received headers:  {:?}", current_headers);
                        }
                    }
                }
            },
            Err(e) => {
                if !be_quiet {
                    eprintln!("Warning: Could not read or parse CSV file '{}': {}. Skipping.", path.display(), e);
                }
            }
        }
    }

    if files_processed == 0 || master_headers.is_none() {
        Err(format!("No valid CSV files with matching headers found in directory '{}'.", dir_path.display()).into())
    } else {
        Ok((master_headers.unwrap(), combined_records))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let (headers, records) = {
        if let Some(dir_path) = &args.directory {
            if !args.raw {
                println!("Reading CSV files from directory: {}", dir_path.display());
            }
            load_data_from_directory(dir_path, args.raw)?
        } else if let Some(file_path) = &args.data_file {
            if file_path.to_string_lossy() == "-" {
                if !args.raw && atty::is(atty::Stream::Stdin) {
                    println!("Reading CSV data from stdin (specified by '-f -')...");
                }
                load_data_from_stdin()?
            } else {
                if !args.raw {
                    println!("Reading CSV file: {}", file_path.display());
                }
                load_data_from_csv(file_path)?
            }
        } else {
            if atty::is(atty::Stream::Stdin) {
                Args::command().print_help()?;
                eprintln!("\nError: No input source specified. Please use -f <file>, -d <directory>, or pipe data to stdin.");
                std::process::exit(1);
            } else {
                if !args.raw {
                    println!("No input file specified, reading CSV data from piped stdin...");
                }
                load_data_from_stdin()?
            }
        }
    };
    
    if records.is_empty() {
        if !args.raw {
            println!("No data rows found.");
        }
        return Ok(());
    }

    let columns_to_display: Vec<String> = if let Some(ref specified_cols) = args.columns {
        let mut valid_cols = Vec::new();
        for col_name_arg in specified_cols {
            if let Some(found_header) = headers.iter().find(|h| h.eq_ignore_ascii_case(col_name_arg)) {
                valid_cols.push(found_header.clone());
            } else {
                if !args.raw {
                    eprintln!("Error: Specified column '{}' not found in CSV headers: {:?}", col_name_arg, headers);
                }
                std::process::exit(1); 
            }
        }
        if valid_cols.is_empty() { 
             if !args.raw {
                eprintln!("Error: No valid columns were specified for display (or provided list was empty).");
             }
             std::process::exit(1);
        }
        valid_cols
    } else {
        vec![headers.first().ok_or_else(|| Box::<dyn Error>::from("No headers found in data (cannot determine default display column)."))?.clone()]
    };

    if args.list {
        let mut list_title = String::new();
        if !args.raw {
            let display_cols_str = columns_to_display.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", ");
            let source_name_str = if let Some(dir_path) = &args.directory {
                format!("directory '{}'", dir_path.display())
            } else if let Some(file_path) = &args.data_file {
                 if file_path.to_string_lossy() == "-" { "stdin".to_string() }
                 else { format!("file '{}'", file_path.display()) }
            } else { 
                if atty::is(atty::Stream::Stdin) { "(no input specified - error handled)".to_string() }
                else { "stdin".to_string() }
            };
            list_title = format!("List from {} (displaying column(s): {})", source_name_str, display_cols_str);
        }

        let records_to_process_refs: Vec<&HashMap<String, String>> = if let Some(filters) = &args.filter {
            for (filter_column, _) in filters { 
                if !headers.iter().any(|h| h.eq_ignore_ascii_case(filter_column)) {
                    if !args.raw {
                       eprintln!("Error: Filter column '{}' not found in CSV file headers: {:?}", filter_column, headers);
                    }
                    std::process::exit(1);
                }
            }
            if !args.raw {
                let filter_descriptions: Vec<String> = filters.iter()
                    .map(|(col, val)| format!("{} = '{}'", col, val))
                    .collect();
                list_title = format!("{} filtered where {}", list_title, filter_descriptions.join(" AND "));
            }
            records.iter().filter(|record| {
                filters.iter().all(|(filter_column, filter_value)| {
                    if let Some(value_in_record) = record.keys().find(|k| k.eq_ignore_ascii_case(filter_column)).and_then(|found_key| record.get(found_key)) {
                        return value_in_record.eq_ignore_ascii_case(filter_value);
                    }
                    false
                })
            }).collect()
        } else {
            records.iter().collect()
        };

        if !args.raw { 
            if records_to_process_refs.is_empty() {
                if args.filter.is_some() { println!("No entries matched your filter."); }
            } else {
                println!("{}", list_title);
                let mut lines_buffer: Vec<String> = Vec::new();
                for record_ref in &records_to_process_refs {
                    let mut current_line_values = Vec::new();
                    for col_name in &columns_to_display {
                        let value = record_ref.get(col_name).cloned().unwrap_or_else(|| String::from("[N/A]"));
                        current_line_values.push(value);
                    }
                    lines_buffer.push(current_line_values.join("\t"));
                }
                println!("Number of entries: {}", lines_buffer.len());
                for (index, line_str) in lines_buffer.iter().enumerate() {
                    println!("{}. {}", index + 1, line_str);
                }
            }
        } else { 
            for record_ref in &records_to_process_refs {
                let mut current_line_values = Vec::new();
                for col_name in &columns_to_display {
                    let value = record_ref.get(col_name).cloned().unwrap_or_default();
                    current_line_values.push(value);
                }
                println!("{}", current_line_values.join("\t"));
            }
        }
    } else {
        let mut rng = rand::thread_rng();
        if let Some(random_record) = records.choose(&mut rng) {
            let mut values_to_print = Vec::new();
            for col_name in &columns_to_display {
                 let value = random_record.get(col_name).cloned().unwrap_or_else(|| {
                    if !args.raw { String::from("[N/A]") } else { String::default() }
                });
                values_to_print.push(value);
            }

            if !args.raw {
                let display_cols_str = columns_to_display.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", ");
                let source_name = if let Some(dir_path) = &args.directory {
                    format!("directory '{}'", dir_path.display())
                } else if let Some(file_path) = &args.data_file {
                    if file_path.to_string_lossy() == "-" { "stdin".to_string() }
                    else { format!("file '{}'", file_path.display()) }
                } else { if atty::is(atty::Stream::Stdin) {"(interactive mode without input - error handled)".to_string()} else {"stdin".to_string()} };
                println!("Random entry (from column(s) '{}' in {}): {}", display_cols_str, source_name, values_to_print.join("\t"));
            } else {
                println!("{}", values_to_print.join("\t"));
            }
        } else if !args.raw && !records.is_empty() { 
             println!("Could not select a random entry (unexpected).");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter_arg_valid() {
        assert_eq!(
            parse_filter_arg("Artist=Queen"),
            Ok(("Artist".to_string(), "Queen".to_string()))
        );
        assert_eq!(
            parse_filter_arg("  Year = 1999  "),
            Ok(("Year".to_string(), "1999".to_string()))
        );
    }

    #[test]
    fn test_parse_filter_arg_invalid() {
        assert!(parse_filter_arg("ArtistQueen").is_err());
        assert_eq!( 
            parse_filter_arg("Artist="),
            Ok(("Artist".to_string(), "".to_string()))
         );
    }

     #[test]
     fn test_parse_filter_arg_empty_key_error() {
         let result = parse_filter_arg("=Value");
         assert!(result.is_err());
         if let Err(e) = result {
             assert!(e.contains("Column name cannot be empty"));
         }

         let result_empty_both = parse_filter_arg("=");
         assert!(result_empty_both.is_err());
         if let Err(e) = result_empty_both {
            assert!(e.contains("Column name cannot be empty"));
        }
     }
}
