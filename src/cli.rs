use clap::{App, Arg};

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
