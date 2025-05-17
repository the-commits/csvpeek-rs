use assert_cmd::Command; 
use predicates::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_list_basic_csv() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("test_data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "Name,Value,Category")?;
    writeln!(file, "Alpha,100,X")?;
    writeln!(file, "Beta,200,Y")?;
    writeln!(file, "Gamma,150,X")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?; 
    cmd.current_dir(temp_dir.path());
    cmd.args(["-f", "test_data.csv", "--list"]);

    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("Reading CSV file: test_data.csv")
                .and(predicate::str::contains("List from file 'test_data.csv' (displaying column(s): Name)"))
                .and(predicate::str::contains("Number of entries: 3"))
                .and(predicate::str::contains("1. Alpha"))
                .and(predicate::str::contains("2. Beta"))
                .and(predicate::str::contains("3. Gamma")),
        );
    Ok(())
}

#[test]
fn test_single_filter_and_multiple_display_columns() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("songs.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "Låt,Artist,Album,År")?;
    writeln!(file, "Hey Jude,The Beatles,N/A,1968")?;
    writeln!(file, "Bohemian Rhapsody,Queen,Opera,1975")?;
    writeln!(file, "Yesterday,The Beatles,N/A,1965")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "songs.csv",
        "--list",
        "--filter", "Artist=The Beatles",
        "--columns", "Låt,År",
    ]);

    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("Reading CSV file: songs.csv")
                .and(predicate::str::contains("displaying column(s): Låt, År"))
                .and(predicate::str::contains("filtered where Artist = 'The Beatles'"))
                .and(predicate::str::contains("Number of entries: 2"))
                .and(predicate::str::contains("1. Hey Jude\t1968"))
                .and(predicate::str::contains("2. Yesterday\t1965"))
                .and(predicate::str::contains("Bohemian Rhapsody").not()),
        )
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_directory_input_merges_and_skips() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let dir_path_obj = temp_dir.path();

    let file_books_path = dir_path_obj.join("books_data.csv"); 
    let mut file_books = File::create(file_books_path)?;
    writeln!(file_books, "Titel,Författare,Genre")?;
    writeln!(file_books, "Moby Dick,Herman Melville,Adventure")?;
    file_books.flush()?;

    let file_songs1_path = dir_path_obj.join("songs_part1.csv");
    let mut file_songs1 = File::create(file_songs1_path)?;
    writeln!(file_songs1, "Låt,Artist,Album,År")?;
    writeln!(file_songs1, "Bohemian Rhapsody,Queen,A Night at the Opera,1975")?;
    file_songs1.flush()?;
    
    let file_songs2_path = dir_path_obj.join("songs_part2.csv"); 
    let mut file_songs2 = File::create(file_songs2_path)?;
    writeln!(file_songs2, "Låt,Artist,Album,År")?;
    writeln!(file_songs2, "Hey Jude,The Beatles,Hey Jude,1968")?;
    file_songs2.flush()?;

    let mut cmd_list = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd_list.current_dir(dir_path_obj);
    cmd_list.args(["-d", ".", "--list", "--columns", "Titel,Genre"]);

    cmd_list.assert()
        .success()
        .stdout(
            predicate::str::contains("Reading CSV files from directory: .")
                .and(predicate::str::contains("Reading file: ./books_data.csv"))
                .and(predicate::str::contains("Reading file: ./songs_part1.csv"))
                .and(predicate::str::contains("Reading file: ./songs_part2.csv"))
                .and(predicate::str::contains("List from directory '.' (displaying column(s): Titel, Genre)"))
                .and(predicate::str::contains("Number of entries: 1"))
                .and(predicate::str::contains("1. Moby Dick\tAdventure"))
        )
        .stderr(
            predicate::str::contains("Warning: Headers in file './songs_part1.csv'")
                .and(predicate::str::contains("Expected headers: [\"Titel\", \"Författare\", \"Genre\"]"))
                .and(predicate::str::contains("Warning: Headers in file './songs_part2.csv'"))
        );
    
    let mut cmd_filter_artist = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd_filter_artist.current_dir(dir_path_obj);
    cmd_filter_artist.args(["-d", ".", "--list", "--filter", "Artist=The Beatles"]);
    
    cmd_filter_artist.assert()
        .code(1) 
        .stdout(
            predicate::str::contains("Reading CSV files from directory: .")
                .and(predicate::str::contains("Reading file: ./books_data.csv"))
                .and(predicate::str::contains("Reading file: ./songs_part1.csv"))
                .and(predicate::str::contains("Reading file: ./songs_part2.csv"))
                .and(predicate::str::contains("List from directory").not())
        )
        .stderr(
            predicate::str::contains("Warning: Headers in file './songs_part1.csv'")
                .and(predicate::str::contains("Warning: Headers in file './songs_part2.csv'"))
                .and(predicate::str::contains("Error: Filter column 'Artist' not found in CSV file headers: [\"Titel\", \"Författare\", \"Genre\"]"))
        );

    let mut cmd_filter_author = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd_filter_author.current_dir(dir_path_obj);
    cmd_filter_author.args(["-d", ".", "--list", "--filter", "Författare=Herman Melville", "--columns", "Titel"]);
    
    cmd_filter_author.assert()
        .success()
        .stdout(
            predicate::str::contains("Reading CSV files from directory: .")
                .and(predicate::str::contains("Reading file: ./books_data.csv"))
                .and(predicate::str::contains("Reading file: ./songs_part1.csv"))
                .and(predicate::str::contains("Reading file: ./songs_part2.csv"))
                .and(predicate::str::contains("List from directory '.' (displaying column(s): Titel) filtered where Författare = 'Herman Melville'"))
                .and(predicate::str::contains("Number of entries: 1"))
                .and(predicate::str::contains("1. Moby Dick"))
        )
        .stderr(
             predicate::str::contains("Warning: Headers in file './songs_part1.csv'")
                .and(predicate::str::contains("Warning: Headers in file './songs_part2.csv'"))
                .and(predicate::str::contains("Error: Filter column").not())
        );
    Ok(())
}

#[test]
fn test_list_multiple_filters() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("multi_filter_data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "Stad,Land,Kontinent,Språk")?;
    writeln!(file, "Stockholm,Sverige,Europa,Svenska")?;
    writeln!(file, "Paris,Frankrike,Europa,Franska")?;
    writeln!(file, "London,UK,Europa,Engelska")?;
    writeln!(file, "Berlin,Tyskland,Europa,Tyska")?;
    writeln!(file, "Tokyo,Japan,Asien,Japanska")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "multi_filter_data.csv",
        "--list",
        "--filter", "Kontinent=Europa",
        "--filter", "Språk=Engelska",
        "--columns", "Stad,Land",
    ]);

    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("Reading CSV file: multi_filter_data.csv")
                .and(predicate::str::contains("List from file 'multi_filter_data.csv' (displaying column(s): Stad, Land) filtered where Kontinent = 'Europa' AND Språk = 'Engelska'"))
                .and(predicate::str::contains("Number of entries: 1"))
                .and(predicate::str::contains("1. London\tUK"))
                .and(predicate::str::contains("Stockholm").not())
        )
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_list_multiple_filters_no_match() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("multi_filter_data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "Stad,Land,Kontinent,Språk")?;
    writeln!(file, "Stockholm,Sverige,Europa,Svenska")?;
    writeln!(file, "Paris,Frankrike,Europa,Franska")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "multi_filter_data.csv",
        "--list",
        "--filter", "Kontinent=Europa",
        "--filter", "Språk=Japanska", 
    ]);

    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("Reading CSV file: multi_filter_data.csv")
                .and(predicate::str::contains("No entries matched your filter."))
                .and(predicate::str::contains("List from file").not())
                .and(predicate::str::contains("Number of entries:").not())
        )
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_list_multiple_filters_invalid_column() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "Header1,Header2")?;
    writeln!(file, "val1,val2")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "data.csv",
        "--list",
        "--filter", "Header1=val1",
        "--filter", "NonExistent=foo",
    ]);

    cmd.assert()
        .code(1)
        .stdout(
            predicate::str::contains("Reading CSV file: data.csv")
        )
        .stderr(
            predicate::str::contains("Error: Filter column 'NonExistent' not found in CSV file headers: [\"Header1\", \"Header2\"]")
        );
    Ok(())
}

#[test]
fn test_list_multiple_columns_with_raw_output() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "ID,Produkt,Pris")?;
    writeln!(file, "1,Äpple,10")?;
    writeln!(file, "2,Päron,12")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "data.csv",
        "--list",
        "--columns", "Produkt,Pris",
        "--raw",
    ]);

    let expected_output = "Äpple\t10\n\
                           Päron\t12\n";

    cmd.assert()
        .success()
        .stdout(expected_output)
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_filter_with_raw_output() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "ID,Produkt,Pris,Kategori")?;
    writeln!(file, "1,Äpple,10,Frukt")?;
    writeln!(file, "2,Päron,12,Frukt")?;
    writeln!(file, "3,Morot,8,Grönsak")?;
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "data.csv",
        "--list",
        "--filter", "Kategori=Frukt",
        "--columns", "Produkt",
        "--raw",
    ]);

    let expected_output = "Äpple\n\
                           Päron\n";

    cmd.assert()
        .success()
        .stdout(expected_output)
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_stdin_input_with_list_and_columns_raw() -> Result<(), Box<dyn Error>> {
    let csv_data = "HeaderA,HeaderB,HeaderC\nval1A,val1B,val1C\nval2A,val2B,val2C\n";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args([
        "-f", "-", 
        "--list",
        "--columns", "HeaderC,HeaderA",
        "--raw",
    ]);
    cmd.write_stdin(csv_data).unwrap(); 

    let expected_output = "val1C\tval1A\n\
                           val2C\tval2A\n";
    
    cmd.assert()
        .success()
        .stdout(expected_output)
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_no_input_args_with_empty_pipe_stdin() -> Result<(), Box<dyn Error>> { 
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp_dir = tempdir()?;
    cmd.current_dir(temp_dir.path());

    cmd.assert()
        .failure() 
        .stdout(
            predicate::str::contains("No input file specified, reading CSV data from piped stdin...")
                .and(predicate::str::contains("Usage: csvpeek-rs").not()) 
        )
        .stderr(
            predicate::str::contains("CSV data is missing headers or is empty.")
                .and(predicate::str::contains("Error: No input source specified.").not()) 
        );
    Ok(())
}

#[test]
fn test_version_flag() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION"))) 
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_help_flag() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: csvpeek-rs [OPTIONS]").and(predicate::str::contains("Options:")))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_random_pick_multiple_columns_raw() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let csv_file_path = temp_dir.path().join("single_row_data.csv");
    let mut file = File::create(&csv_file_path)?;
    writeln!(file, "Col1,Col2,Col3")?;
    writeln!(file, "A1,B1,C1")?; 
    file.flush()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(temp_dir.path());
    cmd.args([
        "-f", "single_row_data.csv",
        "--columns", "Col3,Col1",
        "--raw",
    ]);

    let expected_output = "C1\tA1\n";

    cmd.assert()
        .success()
        .stdout(expected_output)
        .stderr(predicate::str::is_empty());
    Ok(())
}
