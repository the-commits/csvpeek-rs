use clap::{CommandFactory, Parser};
use rand::seq::SliceRandom;
use std::error::Error;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Operator {
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Eq => write!(f, "="),
            Operator::NotEq => write!(f, "!="),
            Operator::Lt => write!(f, "<"),
            Operator::Gt => write!(f, ">"),
            Operator::LtEq => write!(f, "<="),
            Operator::GtEq => write!(f, ">="),
        }
    }
}

fn parse_filter_arg(s: &str) -> Result<(String, Operator, String), String> {
    let (key_str_full, op, val_str_full) = if let Some((k, v)) = s.split_once("!=") {
        (k, Operator::NotEq, v)
    } else if let Some((k, v)) = s.split_once(">=") {
        (k, Operator::GtEq, v)
    } else if let Some((k, v)) = s.split_once("<=") {
        (k, Operator::LtEq, v)
    } else if let Some((k, v)) = s.split_once('=') {
        (k, Operator::Eq, v)
    } else if let Some((k, v)) = s.split_once('>') {
        (k, Operator::Gt, v)
    } else if let Some((k, v)) = s.split_once('<') {
        (k, Operator::Lt, v)
    } else {
        return Err(format!(
            "Invalid filter format: Operator (e.g., =, !=, >, <, >=, <=) missing or unrecognized in '{}'. Expected COLUMN<OP>VALUE.", s
        ));
    };

    let key = key_str_full.trim();

    if key.is_empty() {
        return Err(format!("Invalid filter format: Column name cannot be empty in '{}'. Expected COLUMN<OP>VALUE.", s));
    }

    if key.chars().any(|c| "<>=!".contains(c)) {
        return Err(format!(
            "Invalid filter format: Column name '{}' is malformed (contains operator characters) in filter string '{}'.", key, s
        ));
    }
    
    Ok((key.to_string(), op, val_str_full.trim().to_string()))
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
    * Precisely filter rows using the --filter \"COLUMN<OP>VALUE\" syntax 
        (e.g., \"Age>=30\", \"City!=London\"). OP can be =, !=, >, <, >=, <=. 
        This can be repeated for multiple AND-conditions.
    * Comparisons are case-insensitive for = and !=. For ordering operators, 
        numeric comparison is attempted first; if that fails, a lexicographical 
        string comparison is performed.
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

    /// Filter the list based on COLUMN<OP>VALUE (e.g., "Age>=30", "City!=London").
    /// OP can be =, !=, >, <, >=, <=. Can be repeated for multiple AND conditions.
    /// Used with --list.
    #[clap(long, value_parser = parse_filter_arg, requires = "list", num_args = 0..)]
    filter: Option<Vec<(String, Operator, String)>>,

    /// Path to a single CSV data file. Use "-" to read from stdin.
    /// If neither -f nor -d is given, an attempt to read from stdin (if piped) or show help.
    #[clap(long, short = 'f')]
    data_file: Option<PathBuf>,

    /// Path to a directory containing CSV files to merge.
    /// Takes precedence over --data-file if --main-header-file is not also used to clarify source.
    #[clap(long, short = 'd')]
    directory: Option<PathBuf>,

    /// Specify a file within the input directory (used with -d/--directory)
    /// to define the main headers against which other files will be compared.
    #[clap(long = "main-header-file", short = 'm', value_name = "FILENAME", requires = "directory")]
    main_header_file: Option<String>,

    /// Specify column(s) to display. Use comma-separated values or repeat the flag.
    /// Defaults to the first column if not specified.
    #[clap(long = "columns", short = 'c', value_delimiter = ',')]
    columns: Option<Vec<String>>,

    /// Output raw data values only, one per line (for piping).
    #[clap(long)]
    raw: bool,

    /// Display only the header row from the CSV data and exit.
    /// Cannot be used with --list, --filter, --columns, or --raw.
    #[clap(long, conflicts_with_all = ["list", "filter", "columns", "raw"])]
    headers: bool,
}

fn parse_csv_from_reader<R: Read>(
    reader_source: R,
    load_records: bool,
) -> Result<(Vec<String>, Vec<csv::StringRecord>), Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(reader_source);
    let headers = reader.headers()?.iter().map(String::from).collect::<Vec<String>>();
    if headers.is_empty() {
        return Err("CSV data is missing headers or is empty.".into());
    }

    if !load_records {
        return Ok((headers, Vec::new()));
    }

    let mut records_data = Vec::new();
    for result in reader.records() {
        let record: csv::StringRecord = result?;
        records_data.push(record);
    }
    Ok((headers, records_data))
}

fn load_data_from_csv(filepath: &PathBuf, load_records: bool) -> Result<(Vec<String>, Vec<csv::StringRecord>), Box<dyn Error>> {
    let file = fs::File::open(filepath)?;
    parse_csv_from_reader(file, load_records)
}

fn load_data_from_stdin(load_records: bool) -> Result<(Vec<String>, Vec<csv::StringRecord>), Box<dyn Error>> {
    let stdin = io::stdin();
    parse_csv_from_reader(stdin.lock(), load_records)
}

fn load_data_from_directory(
    dir_path: &PathBuf,
    be_quiet: bool,
    load_records: bool,
    specified_main_header_filename: &Option<String>,
) -> Result<(Vec<String>, Vec<csv::StringRecord>), Box<dyn Error>> {
    
    let mut csv_file_paths: Vec<PathBuf> = fs::read_dir(dir_path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "csv"))
        .collect();
    csv_file_paths.sort();

    if csv_file_paths.is_empty() {
        return Err(format!("No CSV files found in directory '{}'.", dir_path.display()).into());
    }

    let mut main_headers_option: Option<Vec<String>> = None;

    if let Some(filename_str) = specified_main_header_filename {
        let main_header_path = dir_path.join(filename_str);
        if !csv_file_paths.iter().any(|p| p == &main_header_path) {
             return Err(format!("Specified main header file '{}' not found or is not a .csv file in directory '{}'.", filename_str, dir_path.display()).into());
        }
        if !be_quiet { println!("Attempting to set main headers from specified file: {}", main_header_path.display()); }
        match load_data_from_csv(&main_header_path, false) { 
            Ok((headers_from_file, _)) => {
                if headers_from_file.is_empty() {
                    return Err(format!("Specified main header file '{}' is empty or has no headers.", main_header_path.display()).into());
                }
                main_headers_option = Some(headers_from_file);
            }
            Err(e) => {
                return Err(format!("Failed to load headers from specified main header file '{}': {}", main_header_path.display(), e).into());
            }
        }
    } else {
        for path in &csv_file_paths {
            if !be_quiet { println!("Attempting to determine main headers from: {}", path.display()); }
            match load_data_from_csv(path, false) { 
                Ok((headers_from_file, _)) => {
                    if !headers_from_file.is_empty() {
                        main_headers_option = Some(headers_from_file);
                        break; 
                    } else if !be_quiet {
                        eprintln!("Warning: File '{}' has no headers. Trying next file for main headers.", path.display());
                    }
                }
                Err(e) => {
                    if !be_quiet {
                        eprintln!("Warning: Could not read file '{}' to determine main headers: {}. Trying next.", path.display(), e);
                    }
                }
            }
        }
    }

    let final_main_headers = main_headers_option.ok_or_else(|| format!("Could not determine main headers from any suitable file in directory '{}'.", dir_path.display()))?;
    
    let mut combined_records: Vec<csv::StringRecord> = Vec::new();
    let mut files_contributed_records = 0;

    if load_records {
        for path in &csv_file_paths {
            if !be_quiet { println!("Processing file for data: {}", path.display()); }
            match load_data_from_csv(path, true) { 
                Ok((current_headers, records_chunk)) => {
                    if current_headers == final_main_headers {
                        combined_records.extend(records_chunk);
                        files_contributed_records += 1;
                    } else if !be_quiet {
                        eprintln!("Warning: Headers in file '{}' do not match main headers. Skipping records from this file.", path.display());
                    }
                }
                Err(e) => {
                    if !be_quiet { 
                        eprintln!("Warning: Could not read or parse CSV file '{}' for records: {}. Skipping.", path.display(), e); 
                    }
                }
            }
        }
    } else {
        for path in &csv_file_paths {
            if let Ok((current_headers, _)) = load_data_from_csv(path, false) {
                if current_headers == final_main_headers {
                    files_contributed_records += 1;
                }
            }
        }
    }
    
    if files_contributed_records == 0 {
        let for_what_msg = if load_records { " with records" } else { " (for header consistency check)" };
        return Err(format!("No CSV files{} matching main headers ({:?}) found/processed in directory '{}'.", for_what_msg, final_main_headers, dir_path.display()).into());
    }

    Ok((final_main_headers, combined_records))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let should_load_records = !args.headers;

    let (headers, records): (Vec<String>, Vec<csv::StringRecord>) = {
        if let Some(dir_path) = &args.directory {
            load_data_from_directory(dir_path, args.raw || args.headers, should_load_records, &args.main_header_file)?
        } else if let Some(file_path) = &args.data_file {
            if file_path.to_string_lossy() == "-" {
                if !args.raw && !args.headers && std::io::stdin().is_terminal() {
                    println!("Reading CSV data from stdin (specified by '-f -')...");
                }
                load_data_from_stdin(should_load_records)?
            } else {
                if !args.raw && !args.headers {
                    println!("Reading CSV file: {}", file_path.display());
                }
                load_data_from_csv(file_path, should_load_records)?
            }
        } else {
            if std::io::stdin().is_terminal() {
                Args::command().print_help()?;
                eprintln!("\nError: No input source specified. Please use -f <file>, -d <directory>, or pipe data to stdin.");
                std::process::exit(1);
            } else {
                if !args.raw && !args.headers {
                    println!("No input file specified, reading CSV data from piped stdin...");
                }
                load_data_from_stdin(should_load_records)?
            }
        }
    };
    
    if args.headers {
        if headers.is_empty() {
            eprintln!("No headers found or could be determined from the input source.");
        } else {
            for header_name in &headers {
                println!("{}", header_name);
            }
        }
        return Ok(()); 
    }

    if records.is_empty() { 
        if !args.raw {
            println!("No data rows found.");
        }
        return Ok(());
    }

    let columns_to_display_names: Vec<String> = if let Some(ref specified_cols_args) = args.columns {
        let mut valid_cols = Vec::new();
        for col_name_arg in specified_cols_args {
            if let Some(found_header) = headers.iter().find(|h| h.eq_ignore_ascii_case(col_name_arg)) {
                valid_cols.push(found_header.clone());
            } else {
                if !args.raw {
                    eprintln!("Error: Specified display column '{}' not found in CSV headers: {:?}", col_name_arg, headers);
                }
                std::process::exit(1); 
            }
        }
        if valid_cols.is_empty() { 
             if !args.raw {
                eprintln!("Error: No valid display columns were specified (or provided list was empty).");
             }
             std::process::exit(1);
        }
        valid_cols
    } else {
        vec![headers.first().ok_or_else(|| Box::<dyn Error>::from("No headers found in data (cannot determine default display column)."))?.clone()]
    };

    let display_column_indices: Vec<usize> = columns_to_display_names.iter()
        .map(|name| headers.iter().position(|h| h == name).expect("Internal error: Validated display column name not found in headers during index lookup."))
        .collect();

    if args.list {
        let mut list_title = String::new();
        if !args.raw {
            let display_cols_str = columns_to_display_names.join(", ");
            let source_name_str = if let Some(dir_path) = &args.directory {
                format!("directory '{}'", dir_path.display())
            } else if let Some(file_path) = &args.data_file {
                 if file_path.to_string_lossy() == "-" { "stdin".to_string() }
                 else { format!("file '{}'", file_path.display()) }
            } else { 
                "stdin".to_string() 
            };
            list_title = format!("List from {} (displaying column(s): {})", source_name_str, display_cols_str);
        }

        let records_to_process_refs: Vec<&csv::StringRecord> = if let Some(raw_filters) = &args.filter {
            let mut validated_filters: Vec<(usize, Operator, String)> = Vec::new();
            for (user_col_name, op, val_str) in raw_filters {
                if let Some(idx) = headers.iter().position(|h| h.eq_ignore_ascii_case(user_col_name)) {
                    validated_filters.push((idx, *op, val_str.clone()));
                } else {
                    if !args.raw {
                       eprintln!("Error: Filter column '{}' not found in CSV file headers: {:?}", user_col_name, headers);
                    }
                    std::process::exit(1);
                }
            }
            
            if !args.raw && !validated_filters.is_empty() {
                let filter_descriptions: Vec<String> = raw_filters.iter() 
                    .map(|(col, op, val)| format!("{} {} '{}'", col, op, val)) 
                    .collect();
                list_title = format!("{} filtered where {}", list_title, filter_descriptions.join(" AND "));
            }
            
            records.iter().filter(|record| {
                validated_filters.iter().all(|(col_idx, operator, filter_value_str)| {
                    if let Some(value_in_record_str) = record.get(*col_idx) {
                        match operator {
                            Operator::Eq => value_in_record_str.eq_ignore_ascii_case(filter_value_str),
                            Operator::NotEq => !value_in_record_str.eq_ignore_ascii_case(filter_value_str),
                            Operator::Lt | Operator::Gt | Operator::LtEq | Operator::GtEq => {
                                let record_num_res = value_in_record_str.trim().parse::<f64>();
                                let filter_num_res = filter_value_str.trim().parse::<f64>();
                                if let (Ok(record_num), Ok(filter_num)) = (record_num_res, filter_num_res) {
                                    match operator {
                                        Operator::Lt => record_num < filter_num,
                                        Operator::Gt => record_num > filter_num,
                                        Operator::LtEq => record_num <= filter_num,
                                        Operator::GtEq => record_num >= filter_num,
                                        _ => false, 
                                    }
                                } else { 
                                    match operator {
                                        Operator::Lt => value_in_record_str < filter_value_str,
                                        Operator::Gt => value_in_record_str > filter_value_str,
                                        Operator::LtEq => value_in_record_str <= filter_value_str,
                                        Operator::GtEq => value_in_record_str >= filter_value_str,
                                        _ => false, 
                                    }
                                }
                            }
                        }
                    } else { false } 
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
                    for &idx in &display_column_indices {
                        let value = record_ref.get(idx).unwrap_or("[N/A]");
                        current_line_values.push(value.to_string());
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
                for &idx in &display_column_indices {
                    let value = record_ref.get(idx).unwrap_or(""); 
                    current_line_values.push(value.to_string());
                }
                println!("{}", current_line_values.join("\t"));
            }
        }
    } else {
        let mut rng = rand::thread_rng();
        if let Some(random_record) = records.choose(&mut rng) {
            let mut values_to_print = Vec::new();
            for &idx in &display_column_indices {
                 let value = random_record.get(idx).unwrap_or_else(|| {
                    if !args.raw { "[N/A]" } else { "" }
                });
                values_to_print.push(value.to_string());
            }

            if !args.raw {
                let display_cols_str = columns_to_display_names.join(", ");
                let source_name = if let Some(dir_path) = &args.directory {
                    format!("directory '{}'", dir_path.display())
                } else if let Some(file_path) = &args.data_file {
                    if file_path.to_string_lossy() == "-" { "stdin".to_string() }
                    else { format!("file '{}'", file_path.display()) }
                } else { 
                    "stdin".to_string()
                };
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
    fn test_parse_filter_arg_valid_ops() {
        assert_eq!(parse_filter_arg("Col=Val"), Ok(("Col".to_string(), Operator::Eq, "Val".to_string())));
        assert_eq!(parse_filter_arg("Col!=Val"), Ok(("Col".to_string(), Operator::NotEq, "Val".to_string())));
        assert_eq!(parse_filter_arg("Col>Val"), Ok(("Col".to_string(), Operator::Gt, "Val".to_string())));
        assert_eq!(parse_filter_arg("Col<Val"), Ok(("Col".to_string(), Operator::Lt, "Val".to_string())));
        assert_eq!(parse_filter_arg("Col>=Val"), Ok(("Col".to_string(), Operator::GtEq, "Val".to_string())));
        assert_eq!(parse_filter_arg("Col<=Val"), Ok(("Col".to_string(), Operator::LtEq, "Val".to_string())));
        assert_eq!(parse_filter_arg("  Col  >=  Val  "), Ok(("Col".to_string(), Operator::GtEq, "Val".to_string())));
    }

    #[test]
    fn test_parse_filter_arg_invalid_ops_or_format() {
        assert!(parse_filter_arg("ColVal").is_err()); 
        assert!(parse_filter_arg("Col<>Val").is_err());
        assert_eq!(parse_filter_arg("Col><Val"), Ok(("Col".to_string(), Operator::Gt, "<Val".to_string())));
    }

     #[test]
     fn test_parse_filter_arg_empty_key_error() {
         let result = parse_filter_arg("=Value");
         assert!(result.is_err());
         if let Err(e) = result {
             assert!(e.contains("Column name cannot be empty"));
         }

         let result_op = parse_filter_arg(">=Value"); 
         assert!(result_op.is_err());
         if let Err(e) = result_op {
             assert!(e.contains("Column name cannot be empty"));
         }
     }

    #[test]
    fn test_parse_filter_arg_empty_value_is_ok() {
         assert_eq!(parse_filter_arg("Col="), Ok(("Col".to_string(), Operator::Eq, "".to_string())));
         assert_eq!(parse_filter_arg("Col>="), Ok(("Col".to_string(), Operator::GtEq, "".to_string())));
    }
}
