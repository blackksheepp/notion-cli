use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub fn tables_view(
    content_width: &u16,
    content_height: &u16,
    x_center: &u16,
    y_search: &u16,
    tables: &Vec<&str>,
    table_pos: usize,
    favorites: &Vec<&str>,
) {
    for i in 2..content_height - 4 {
        execute!(
            stdout(),
            MoveTo(x_center - (content_width / 2) + 1, y_search + i + 1),
            Print(" ".repeat((content_width - 2) as usize)),
        )
        .unwrap();
    }

    execute!(
        stdout(),
        MoveTo(x_center - (content_width / 2) + 10, y_search + 7),
        SetBackgroundColor(Color::White),
        SetForegroundColor(Color::Black),
        Print(" browse tables ".to_string()),
        ResetColor
    )
    .unwrap();

    let count = 7;
    let start = count * (table_pos / count);
    let end = if start + count > tables.len() {
        tables.len()
    } else {
        start + count
    };
    for (i, table) in tables[start..end].iter().enumerate() {
        execute!(
            stdout(),
            MoveTo(
                x_center - (content_width / 2) + 10,
                y_search + 9 + (i as u16)
            ),
            SetForegroundColor(if i == (table_pos%count) {
                if favorites.contains(table) {
                    Color::Black
                } else {
                    Color::White
                }
            } else {
                Color::DarkGrey
            }),
            SetBackgroundColor(if i == (table_pos%count) {
                if favorites.contains(table) {
                    Color::Rgb { r: 252, g: 186, b: 3 }
                } else {
                    Color::Rgb {r: 49, g: 116, b: 143}
                }
            } else {
                Color::Reset
            }),
            Print(format!(
                " {}{} ",
                if favorites.contains(table) { "*" } else { "" },
                table
            )),
            ResetColor
        )
        .unwrap();
    }
}
