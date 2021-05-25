#![feature(drain_filter)]
#![feature(iter_intersperse)]
use std::io::{self, Read, Write};

use clap::{App, Arg};

#[derive(Clone)]
enum Alignment {
    Left,
    Right,
    Center,
}

impl Alignment {
    const IDENTIFIERS: [char; 9] = ['l', 'L', '<', 'r', 'R', '>', 'c', 'C', '^'];
    const LEFT_IDENTIFIER: [char; 3] = ['l', 'L', '<'];
    const RIGHT_IDENTIFIER: [char; 3] = ['r', 'R', '>'];
    const CENTER_IDENTIFIER: [char; 3] = ['c', 'C', '^'];

    fn from(alignment: char) -> Self {
        match alignment {
            c if Alignment::LEFT_IDENTIFIER.contains(&c) => Alignment::Left,
            c if Alignment::RIGHT_IDENTIFIER.contains(&c) => Alignment::Right,
            c if Alignment::CENTER_IDENTIFIER.contains(&c) => Alignment::Center,
            _ => unreachable!(),
        }
    }

    fn write_cell(&self, lock: &mut std::io::StdoutLock, cell: &str, output_separator: &str, space_width: usize) {
        match &self {
            Alignment::Left => {
                write!(
                    lock,
                    "{cell}{:<width$}{output_separator}",
                    "",
                    cell = cell,
                    output_separator = output_separator,
                    width = space_width
                )
                .unwrap();
            }
            Alignment::Right => {
                write!(
                    lock,
                    "{:>width$}{cell}{output_separator}",
                    "",
                    cell = cell,
                    output_separator = output_separator,
                    width = space_width
                )
                .unwrap();
            }
            Alignment::Center => {
                if space_width % 2 == 0 {
                    write!(
                        lock,
                        "{0:^width$}{cell}{0:^width$}{output_separator}",
                        "",
                        cell = cell,
                        output_separator = output_separator,
                        width = space_width / 2
                    )
                    .unwrap();
                } else {
                    let width_left = (space_width - 1) / 2;
                    let width_right = width_left + 1;

                    write!(
                        lock,
                        "{0:^width_left$}{cell}{0:^width_right$}{output_separator}",
                        "",
                        cell = cell,
                        output_separator = output_separator,
                        width_left = width_left,
                        width_right = width_right
                    )
                    .unwrap();
                }
            }
        }
    }
}

fn main() {
    // Read cli params
    let align_examples = [
        "Examples:",
        "\nAll columns right: xcol --alignment r",
        "\nLeft, center, right: xcol --alignment lcr",
    ]
    .concat();

    let matches = App::new("xcol")
        .version("0.1")
        .author("Linus789")
        .arg(
            Arg::new("separator")
                .short('s')
                .long("separator")
                .default_value(" ")
                .hide_default_value(true)
                .about("Specify the columns delimiter for table output (default is whitespace)")
                .takes_value(true),
        )
        .arg(
            Arg::new("output-separator")
                .short('o')
                .long("output-separator")
                .default_value(" ")
                .hide_default_value(true)
                .about("Specify the possible input item delimiters (default is whitespace)")
                .takes_value(true),
        )
        .arg(
            Arg::new("keep-blank-lines")
                .short('L')
                .long("keep-blank-lines")
                .about("Preserve whitespace-only lines in the input"),
        )
        .arg(
            Arg::new("columns-titles")
                .short('N')
                .long("columns-titles")
                .about("Specify the columns titles by comma separated list of titles")
                .takes_value(true),
        )
        .arg(
            Arg::new("alignment")
                .short('a')
                .long("alignment")
                .default_value("l")
                .validator(|s| {
                    if s.chars().all(|c| Alignment::IDENTIFIERS.contains(&c)) {
                        Ok(())
                    } else {
                        Err([
                            &format!(
                                "The alignment can only contain the following characters: {}",
                                Alignment::IDENTIFIERS.iter().collect::<String>()
                            ),
                            "\n\n",
                            &align_examples,
                        ]
                        .concat())
                    }
                })
                .hide_default_value(true)
                .about(
                    &[
                        "Specify a column's alignment, may be repeated (default is left)",
                        "\nUse 'l', 'r', 'c' for left, right, center alignment",
                        "\n\n",
                        &align_examples,
                    ]
                    .concat(),
                )
                .takes_value(true),
        )
        .arg(
            Arg::new("file")
                .index(1)
                .about("Input file (default is stdin)")
                .takes_value(true),
        )
        .get_matches();

    let separator = matches.value_of("separator").unwrap();
    let output_separator = matches.value_of("output-separator").unwrap();
    let keep_blank = matches.is_present("keep-blank-lines");
    let columns_titles = matches.value_of("columns-titles");
    let alignment = matches.value_of("alignment").unwrap();
    let file = matches.value_of("file");

    // Read lines
    let text = if let Some(file_path) = file {
        if let Ok(text) = std::fs::read_to_string(file_path) {
            text
        } else {
            eprintln!("ERROR: The file '{}' does not exist", file_path);
            std::process::exit(2);
        }
    } else {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let mut text = String::new();
        let mut line = String::new();

        while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
            if n_bytes == 0 {
                break;
            }

            text.push_str(&line);
            line.clear();
        }

        text
    };

    let mut lines: Vec<&str> = text.lines().collect();

    if keep_blank {
        // Remove only last line if blank
        if let Some(last_line) = lines.last() {
            if console::strip_ansi_codes(last_line).trim().is_empty() {
                lines.pop().unwrap();
            }
        }
    } else {
        // Remove blank lines
        lines.drain_filter(|line| console::strip_ansi_codes(line).trim().is_empty());
    };

    // Stop if no lines
    if lines.is_empty() {
        return;
    }

    // Add columns titles
    let header_line: Option<String> = if let Some(header_names) = columns_titles {
        let multiple_commas_regex = regex::Regex::new(",{2,}").unwrap();
        Some(
            multiple_commas_regex
                .replace_all(header_names, ",")
                .split(',')
                .map(|name| format!("{}\x1B[0m", name))
                .intersperse(separator.to_string())
                .collect(),
        )
    } else {
        None
    };

    if let Some(header_line) = header_line.as_deref() {
        lines.insert(0, header_line);
    }

    // Calculate column/cell width
    let mut max_columns_num = 0usize;
    let mut columns_per_line: Vec<Option<Vec<&str>>> = vec![None; lines.len()];

    for (line_index, line) in lines.iter().enumerate() {
        let column_entries: Vec<&str> = line.split(separator).collect();

        if column_entries.len() > max_columns_num {
            max_columns_num = column_entries.len();
        }

        columns_per_line[line_index] = Some(column_entries);
    }

    let mut column_widths: Vec<usize> = vec![0; max_columns_num];
    let mut cell_widths: Vec<Vec<usize>> = vec![Vec::new(); lines.len()];

    for (line_index, column_entries) in columns_per_line.iter().enumerate() {
        for (column_index, column_entry) in column_entries.as_ref().unwrap().iter().enumerate() {
            let column_entry_len = console::measure_text_width(column_entry);

            if column_widths[column_index] < column_entry_len {
                column_widths[column_index] = column_entry_len;
            }

            cell_widths[line_index].push(column_entry_len);
        }
    }

    // Print to console
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let repeat_alignment = if alignment.len() == 1 {
        Alignment::from(alignment.chars().next().unwrap())
    } else {
        Alignment::Left
    };

    let column_handles: Vec<Alignment> = alignment
        .chars()
        .map(Alignment::from)
        .chain(std::iter::repeat(repeat_alignment))
        .take(max_columns_num)
        .collect();

    columns_per_line
        .iter()
        .map(|columns_option| columns_option.as_ref().unwrap())
        .enumerate()
        .for_each(|(line_index, columns)| {
            for (column_index, column_entry) in columns.iter().enumerate() {
                let output_separator = if column_index == columns.len() - 1 {
                    ""
                } else {
                    output_separator
                };

                let actual_width = cell_widths[line_index][column_index];
                let column_width = column_widths[column_index];
                let space_width = column_width - actual_width;

                column_handles[column_index].write_cell(&mut lock, column_entry, output_separator, space_width);
            }

            if line_index != lines.len() - 1 {
                writeln!(lock).unwrap();
            }
        });

    // Reset color
    write!(lock, "\x1B[0m").unwrap();

    // Disable mouse tracking if it got enabled (only for xterm)
    // https://stackoverflow.com/questions/5966903/how-to-get-mousemove-and-mouseclick-in-bash
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("xterm") {
            write!(lock, "\x1B[?9l").unwrap();
            write!(lock, "\x1B[?1000l").unwrap();
            write!(lock, "\x1B[?1001l").unwrap();
            write!(lock, "\x1B[?1002l").unwrap();
            write!(lock, "\x1B[?1003l").unwrap();
        }
    }

    // Last newline after the color got reset (cursor color did not change otherwise)
    writeln!(lock).unwrap();
}
