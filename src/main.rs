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
                if space_width & 1 == 0 {
                    // space_width even
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
                    // space_width odd
                    let width_left = (space_width - 1) >> 1; // >> 1 => dividing by 2
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
        .get_matches();

    let separator = matches.value_of("separator").unwrap();
    let output_separator = matches.value_of("output-separator").unwrap();
    let alignment = matches.value_of("alignment").unwrap();

    // Read lines from stdin
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut total_line = String::new();
    let mut line = String::new();

    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 {
            break;
        }

        total_line.push_str(&line);
        line.clear();
    }

    let mut lines: Vec<&str> = total_line.lines().collect();

    // Remove/ignore last line if blank
    if let Some(last_line) = lines.last() {
        if console::strip_ansi_codes(last_line).trim().is_empty() {
            lines.pop().unwrap();
        }
    }

    // Stop if no lines
    if lines.is_empty() {
        return;
    }

    // Calculate column/cell width
    let columns_num = lines.first().unwrap().split(&separator).count();
    let mut columns_per_line: Vec<Option<Vec<&str>>> = vec![None; lines.len()];
    let mut actual_widths: Vec<usize> = Vec::with_capacity(lines.len() * columns_num);
    let mut column_widths: Vec<usize> = vec![0; columns_num];

    for (line_index, line) in lines.iter().enumerate() {
        let column_entries: Vec<&str> = line.split(&separator).collect();

        if column_entries.len() != columns_num {
            let column_str = |col_num: usize| {
                if col_num != 1 {
                    "columns"
                } else {
                    "column"
                }
            };

            let strip_codes = |s: &str| -> String { console::strip_ansi_codes(s).replace('\r', "") };

            eprintln!("ERROR: Columns are not of equal length");
            eprintln!(
                "(line 1, {} {}) {}",
                columns_num,
                column_str(columns_num),
                strip_codes(lines.first().unwrap())
            );
            eprintln!(
                "(line {}, {} {}) {}",
                line_index + 1,
                column_entries.len(),
                column_str(column_entries.len()),
                strip_codes(line)
            );

            std::process::exit(1);
        }

        for (column_index, column_entry) in column_entries.iter().enumerate() {
            let column_entry_len = console::measure_text_width(column_entry);

            if column_widths[column_index] < column_entry_len {
                column_widths[column_index] = column_entry_len;
            }

            actual_widths.push(column_entry_len);
        }

        columns_per_line[line_index] = Some(column_entries);
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
        .take(columns_num)
        .collect();

    columns_per_line
        .iter()
        .map(|columns_option| columns_option.as_ref().unwrap())
        .enumerate()
        .for_each(|(line_index, columns)| {
            for (column_index, column_entry) in columns.iter().enumerate() {
                let output_separator = if column_index == columns_num - 1 {
                    ""
                } else {
                    output_separator
                };

                let actual_width = actual_widths[get_width_index(line_index, column_index, columns_num)];
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

fn get_width_index(line_index: usize, column_index: usize, columns_num: usize) -> usize {
    line_index * columns_num + column_index
}
