use std::collections::HashSet;

use clap::{lazy_static::lazy_static, App, Arg};
use regex::Regex;

pub(crate) struct ColumnsTitles {
    pub(crate) title_line: String,
    pub(crate) named_columns: usize,
}

impl ColumnsTitles {
    pub(crate) fn from(input: &str, separator: &str) -> Self {
        let mut named_columns = 0usize;
        let title_line = list_items(&list_clean_input(input))
            .map(|name| format!("{}\x1B[0m", name))
            .filter(|_| {
                named_columns += 1;
                true
            })
            .intersperse(separator.to_string())
            .collect();

        ColumnsTitles {
            title_line,
            named_columns,
        }
    }
}

pub(crate) enum ColumnsHide {
    Indices(HashSet<usize>),
    Unnamed,
    None,
}

impl ColumnsHide {
    pub(crate) fn from(input: Option<&str>) -> Self {
        if let Some(columns_hide) = input {
            if columns_hide == "-" {
                return ColumnsHide::Unnamed;
            }

            ColumnsHide::Indices(
                list_items(&list_clean_input(columns_hide))
                    .map(|x| {
                        let human_index = x.parse::<usize>().unwrap_or_else(|_| error_unknown_column_index(x));

                        if human_index == 0 {
                            error_unknown_column_index(x);
                        }

                        human_index - 1
                    })
                    .collect(),
            )
        } else {
            ColumnsHide::None
        }
    }
}

fn error_unknown_column_index(input: &str) -> ! {
    eprintln!("ERROR: Unknown column index ({})", input);
    std::process::exit(1);
}

lazy_static! {
    static ref MULTIPLE_COMMAS_REGEX: Regex = Regex::new(",{2,}").unwrap();
    static ref LIST_IDENTIFIER_CHAR: char = ',';
    static ref LIST_IDENTIFIER_STR: &'static str = ",";
}

fn list_clean_input(list: &str) -> std::borrow::Cow<'_, str> {
    MULTIPLE_COMMAS_REGEX.replace_all(list.trim_matches(*LIST_IDENTIFIER_CHAR), *LIST_IDENTIFIER_STR)
}

fn list_items(list: &str) -> std::str::Split<'_, char> {
    list.split(*LIST_IDENTIFIER_CHAR)
}

pub(crate) fn get_cli_params() -> clap::ArgMatches {
    let align_examples = [
        "Examples:",
        "\nAll columns right: xcol --alignment r",
        "\nLeft, center, right: xcol --alignment lcr",
    ]
    .concat();

    App::new("xcol")
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
            Arg::new("columns-hide")
                .short('H')
                .long("columns-hide")
                .about("Don't print specified columns\nThe special placeholder '-' may be used to hide all unnamed columns")
                .takes_value(true),
        )
        .arg(
            Arg::new("alignment")
                .short('a')
                .long("alignment")
                .default_value("l")
                .validator(|s| {
                    if s.chars().all(|c| super::Alignment::IDENTIFIERS.contains(&c)) {
                        Ok(())
                    } else {
                        Err([
                            &format!(
                                "The alignment can only contain the following characters: {}",
                                super::Alignment::IDENTIFIERS.iter().collect::<String>()
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
        .get_matches()
}
