#![feature(drain_filter)]
#![feature(iter_intersperse)]
mod alignment;
mod cli;

use std::io::{self, Read, Write};

use alignment::Alignment;
use cli::{ColumnsHide, ColumnsTitles};

fn main() {
    // Read cli params
    let matches = cli::get_cli_params();
    let file = matches.value_of("file");
    let separator = matches.value_of("separator").unwrap();
    let alignment = matches.value_of("alignment").unwrap();
    let output_separator = matches.value_of("output-separator").unwrap();
    let keep_blank = matches.is_present("keep-blank-lines");
    let columns_titles = matches.value_of("columns-titles");
    let columns_hide = matches.value_of("columns-hide");

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
    let columns_titles: Option<ColumnsTitles> =
        columns_titles.map(|header_names| ColumnsTitles::from(header_names, separator));

    if let Some(columns_titles) = columns_titles.as_ref() {
        lines.insert(0, &columns_titles.title_line);
    }

    // Hidden columns
    let columns_hide = ColumnsHide::from(columns_hide);

    // Preprocess columns
    // * Split into columns
    // * Calculate column/cell width
    // * Hide columns if necessary
    let mut max_columns_num = 0usize;
    let mut columns_per_line: Vec<Option<Vec<&str>>> = vec![None; lines.len()];

    for (line_index, line) in lines.iter().enumerate() {
        let column_entries: Vec<&str> = line
            .split(separator)
            .enumerate()
            .filter_map(|(column_index, column_entry)| {
                match &columns_hide {
                    ColumnsHide::Indices(indices) => {
                        if indices.contains(&column_index) {
                            return None;
                        }
                    }
                    ColumnsHide::Unnamed => {
                        if let Some(columns_titles) = columns_titles.as_ref() {
                            if column_index >= columns_titles.named_columns {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => (),
                }

                Some(column_entry)
            })
            .collect();

        if column_entries.len() > max_columns_num {
            max_columns_num = column_entries.len();
        }

        columns_per_line[line_index] = Some(column_entries);
    }

    // Stop if no columns
    if max_columns_num == 0 {
        return;
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
