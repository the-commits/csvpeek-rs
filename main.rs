use clap::Parser;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::fs;

fn parse_filter_arg(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .ok_or_else(|| format!("Ogiltigt filterformat. Förväntade KOLUMN=VÄRDE, fick '{}'", s))
}

#[derive(Parser, Debug)]
#[clap(author = "The Commits", version = "0.1.0", about = "Displays and filters data from a csv file", long_about = None)]
struct Args {
    #[clap(short, long)]
    list: bool,

    #[clap(long, value_parser = parse_filter_arg, requires = "list")]
    filter: Option<(String, String)>,

    #[clap(long, short='f', default_value = "data.csv")]
    data_file: PathBuf,

    /// Sökväg till en katalog som innehåller CSV-filer att slå samman.
    /// Om denna anges, ignoreras --data-file.
    #[clap(long, short = 'd')]
    directory: Option<PathBuf>,
}

// load_data_from_csv är i stort sett oförändrad, men kan behöva små justeringar
// för att lättare kunna anropas från load_data_from_directory.
// Vi kan behålla den som den är för nu.
fn load_data_from_csv(filepath: &PathBuf) -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(filepath)?;
    let headers = reader.headers()?.iter().map(String::from).collect::<Vec<String>>();
    if headers.is_empty() {
        return Err(format!("CSV-filen '{}' saknar headers eller är tom.", filepath.display()).into());
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

fn load_data_from_directory(dir_path: &PathBuf) -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn Error>> {
    let mut master_headers: Option<Vec<String>> = None;
    let mut combined_records: Vec<HashMap<String, String>> = Vec::new();
    let mut files_processed = 0;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "csv" {
                    println!("Läser fil: {}", path.display()); // Feedback till användaren
                    match load_data_from_csv(&path) {
                        Ok((current_headers, current_records)) => {
                            if master_headers.is_none() {
                                // Detta är den första CSV-filen, dess headers blir "master"
                                master_headers = Some(current_headers);
                                combined_records.extend(current_records);
                                files_processed += 1;
                            } else if Some(&current_headers) == master_headers.as_ref() {
                                // Headers matchar, lägg till records
                                combined_records.extend(current_records);
                                files_processed += 1;
                            } else {
                                // Headers matchar inte, varna och hoppa över filen
                                eprintln!("Varning: Headers i filen '{}' matchar inte de tidigare inlästa filernas headers. Hoppar över denna fil.", path.display());
                                if let Some(mh) = &master_headers {
                                     eprintln!("Förväntade headers: {:?}", mh);
                                     eprintln!("Fick headers:      {:?}", current_headers);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Varning: Kunde inte läsa eller parsa CSV-filen '{}': {}. Hoppar över.", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    if files_processed == 0 || master_headers.is_none() {
        Err(format!("Inga giltiga CSV-filer med matchande headers hittades i katalogen '{}'.", dir_path.display()).into())
    } else {
        Ok((master_headers.unwrap(), combined_records))
    }
}

// src/main.rs (fortsättning)

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Bestäm datakälla och ladda data
    let (headers, records) = if let Some(dir_path) = &args.directory {
        // Användaren specificerade en katalog
        println!("Läser CSV-filer från katalog: {}", dir_path.display());
        match load_data_from_directory(dir_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Fel vid laddning av data från katalog '{}': {}", dir_path.display(), e);
                return Err(e);
            }
        }
    } else {
        // Användaren specificerade en enskild fil (eller standardvärdet data.csv)
        match load_data_from_csv(&args.data_file) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Fel vid laddning av CSV-fil '{}': {}", args.data_file.display(), e);
                return Err(e);
            }
        }
    };

    // Resten av main-funktionen (från "if records.is_empty()...") är i stort sett oförändrad.
    // Den kommer nu att arbeta med antingen data från en enskild fil eller den sammanslagna
    // datan från flera filer i en katalog.

    if records.is_empty() {
        println!("Inga data-rader hittades (förutom eventuella headers).");
        return Ok(());
    }

    let primary_column_name = headers.first().ok_or_else(|| Box::<dyn Error>::from("Inga headers definierade efter dataladdning."))?.clone();

    if args.list {
        let mut items_to_display: Vec<String> = Vec::new();
        let mut list_title = if let Some(dir_path) = &args.directory {
            format!("Lista från katalog '{}' (kolumn: '{}')", dir_path.display(), primary_column_name)
        } else {
            format!("Lista från '{}' (kolumn: '{}')", args.data_file.display(), primary_column_name)
        };

        let records_to_process_refs: Vec<&HashMap<String, String>>; // Håller referenser

         if let Some((filter_column, filter_value)) = &args.filter {
             if !headers.iter().any(|h| h.eq_ignore_ascii_case(filter_column)) {
                 eprintln!("Fel: Kolumnen '{}' finns inte i CSV-filens headers: {:?}", filter_column, headers);
                 return Ok(());
             }
             list_title = format!("{} filtrerat där {} = '{}'", list_title, filter_column, filter_value);
             
             records_to_process_refs = records.iter()
                 .filter(|record| {
                     if let Some(value_in_record) = record.iter().find(|(k, _)| k.eq_ignore_ascii_case(filter_column)).map(|(_,v)| v) {
                         return value_in_record.eq_ignore_ascii_case(filter_value);
                     }
                     false
                 })
                 .collect();
         } else {
             records_to_process_refs = records.iter().collect();
         }

         if records_to_process_refs.is_empty() {
              if args.filter.is_some() {
                 println!("Inga poster matchade ditt filter.");
             } else {
                 println!("Inga poster att visa.");
             }
         } else {
             println!("{}", list_title);
             for record_ref in &records_to_process_refs {
                 if let Some(value) = record_ref.get(&primary_column_name) {
                     items_to_display.push(value.clone());
                 }
             }
             println!("Antal poster: {}", items_to_display.len());
             for (index, item) in items_to_display.iter().enumerate() {
                 println!("{}. {}", index + 1, item);
             }
         }
    } else { // Standard: slumpmässig post
        let mut rng = rand::thread_rng();
        if let Some(random_record) = records.choose(&mut rng) {
            if let Some(value) = random_record.get(&primary_column_name) {
                let source_name = if let Some(dir_path) = &args.directory {
                    dir_path.display().to_string()
                } else {
                    args.data_file.display().to_string()
                };
                println!("Slumpmässig post (från kolumn '{}' i '{}'): {}", primary_column_name, source_name, value);
            }
        }
    }
    Ok(())
}

