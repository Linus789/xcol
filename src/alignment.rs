use std::io::Write;

#[derive(Clone)]
pub(crate) enum Alignment {
    Left,
    Right,
    Center,
}

impl Alignment {
    pub(crate) const IDENTIFIERS: [char; 9] = ['l', 'L', '<', 'r', 'R', '>', 'c', 'C', '^'];
    pub(crate) const LEFT_IDENTIFIER: [char; 3] = ['l', 'L', '<'];
    pub(crate) const RIGHT_IDENTIFIER: [char; 3] = ['r', 'R', '>'];
    pub(crate) const CENTER_IDENTIFIER: [char; 3] = ['c', 'C', '^'];

    pub(crate) fn from(alignment: char) -> Self {
        match alignment {
            c if Alignment::LEFT_IDENTIFIER.contains(&c) => Alignment::Left,
            c if Alignment::RIGHT_IDENTIFIER.contains(&c) => Alignment::Right,
            c if Alignment::CENTER_IDENTIFIER.contains(&c) => Alignment::Center,
            _ => unreachable!(),
        }
    }

    pub(crate) fn write_cell(
        &self,
        lock: &mut std::io::StdoutLock,
        cell: &str,
        output_separator: &str,
        space_width: usize,
    ) {
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
