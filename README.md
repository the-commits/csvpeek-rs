# csvpeek-rs
`csvpeek-rs` aims to be a simple yet powerful addition to your command-line data toolkit, combining the performance of Rust with a user-friendly interface for common CSV operations.

## Examples

First, let's assume we have a CSV file named `songs.csv` with the following content:

```csv
Title,Artist,Album,Year,Genre,Rating
Bohemian Rhapsody,Queen,A Night at the Opera,1975,Rock,5
Hey Jude,The Beatles,Hey Jude,1968,Rock,4
Stairway to Heaven,Led Zeppelin,Led Zeppelin IV,1971,Rock,5
Imagine,John Lennon,Imagine,1971,Pop,4
Like a Rolling Stone,Bob Dylan,Highway 61 Revisited,1965,Rock,4
Smells Like Teen Spirit,Nirvana,Nevermind,1991,Grunge,5
Wonderwall,Oasis,Morning Glory,1995,Britpop,3
Waterloo Sunset,The Kinks,Something Else,1967,Rock,4
````

Here are some ways you can use `csvpeek-rs`:

### 1\. List all song titles (default first column)

This command lists the content of the first column ("Title") for all rows.

```bash
csvpeek-rs -f songs.csv --list
```

Output will be similar to:

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title)
Number of entries: 8
1. Bohemian Rhapsody
2. Hey Jude
3. Stairway to Heaven
4. Imagine
5. Like a Rolling Stone
6. Smells Like Teen Spirit
7. Wonderwall
8. Waterloo Sunset
```

### 2\. List specific columns (e.g., Title and Artist)

You can specify one or more columns to display using `-c` or `--columns`.

```bash
csvpeek-rs -f songs.csv --list -c "Title,Artist"
```

Or using repeated flags:

```bash
csvpeek-rs -f songs.csv --list -c Title -c Artist
```

Output (values will be tab-separated):

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title, Artist)
Number of entries: 8
1. Bohemian Rhapsody	Queen
2. Hey Jude	The Beatles
3. Stairway to Heaven	Led Zeppelin
4. Imagine	John Lennon
5. Like a Rolling Stone	Bob Dylan
6. Smells Like Teen Spirit	Nirvana
7. Wonderwall	Oasis
8. Waterloo Sunset	The Kinks
```

### 3\. Get a random song (showing default first column: Title)

If `--list` is not specified, `csvpeek-rs` shows a random entry.

```bash
csvpeek-rs -f songs.csv
```

Output (will vary):

```
Reading CSV file: songs.csv
Random entry (from column(s) 'Title' in file 'songs.csv'): Hey Jude
```

### 4\. Get a random song, showing specific columns (Artist and Year)

```bash
csvpeek-rs -f songs.csv -c "Artist,Year"
```

Output (will vary, values tab-separated):

```
Reading CSV file: songs.csv
Random entry (from column(s) 'Artist, Year' in file 'songs.csv'): Nirvana	1991
```

### 5\. Filter songs by a specific artist

Use the `--filter` flag with the format `COLUMN_NAME=VALUE`.

```bash
csvpeek-rs -f songs.csv --list --filter "Artist=The Beatles" -c Title
```

Output:

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title) filtered where Artist = 'The Beatles'
Number of entries: 2
1. Hey Jude
2. Waterloo Sunset
```

```bash
csvpeek-rs -f songs.csv --list --filter "Artist=The Beatles" -c Title
```

Output:

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title) filtered where Artist = 'The Beatles'
Number of entries: 1
1. Hey Jude
```

### 6\. Filter songs by year using a numeric operator

Filter for songs released in or after 1990.

```bash
csvpeek-rs -f songs.csv --list --filter "Year>=1990" -c "Title,Year"
```

Output:

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title, Year) filtered where Year >= '1990'
Number of entries: 2
1. Smells Like Teen Spirit	1991
2. Wonderwall	1995
```

### 7\. Combine multiple filters (AND logic)

List Rock songs with a Rating greater than or equal to 4.

```bash
csvpeek-rs -f songs.csv --list --filter "Genre=Rock" --filter "Rating>=4" -c "Title,Artist,Rating"
```

Output:

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title, Artist, Rating) filtered where Genre = 'Rock' AND Rating >= '4'
Number of entries: 4
1. Bohemian Rhapsody	Queen	5
2. Hey Jude	The Beatles	4
3. Stairway to Heaven	Led Zeppelin	4 
4. Like a Rolling Stone	Bob Dylan	4
```

```bash
csvpeek-rs -f songs.csv --list --filter "Genre=Rock" --filter "Rating>=4" -c "Title,Artist,Rating"
```

Output:

```
Reading CSV file: songs.csv
List from file 'songs.csv' (displaying column(s): Title, Artist, Rating) filtered where Genre = 'Rock' AND Rating >= '4'
Number of entries: 5
1. Bohemian Rhapsody	Queen	5
2. Hey Jude	The Beatles	4
3. Stairway to Heaven	Led Zeppelin	5
4. Like a Rolling Stone	Bob Dylan	4
5. Waterloo Sunset	The Kinks	4
```

### 8\. Filter and display specific columns with raw output (for piping)

Get Titles and Artists of Pop songs, raw tab-separated output.

```bash
csvpeek-rs -f songs.csv --list --filter "Genre=Pop" -c "Title,Artist" --raw
```

Output:

```
Imagine	John Lennon
```

### 9\. Read from stdin

You can pipe data into `csvpeek-rs` using `-f -` or by direct pipe.

```bash
cat songs.csv | csvpeek-rs -f - --list --filter "Artist=Queen" -c Title --raw
```

Or implicitly:

```bash
cat songs.csv | csvpeek-rs --list --filter "Artist=Queen" -c Title --raw
```

Output for both:

```
Bohemian Rhapsody
```

### 10\. Read from a directory of CSV files

If you have a directory `my_song_collection/` containing multiple CSV files (`rock_songs.csv`, `pop_songs.csv`) with the *same headers*:

```bash
csvpeek-rs -d my_song_collection/ --list --filter "Year<1970" -c "Title,Artist,Year"
```

This command would:

1.  Inform you it's reading from the directory.
2.  Read each CSV file (e.g., `Reading file: my_song_collection/rock_songs.csv`).
3.  Warn about and skip files with non-matching headers.
4.  Merge data from files with matching headers.
5.  Apply the filter (Year \< 1970).
6.  Display the "Title", "Artist", and "Year" for matching songs from all processed files.

These examples should cover the main ways to use `csvpeek-rs`\!
