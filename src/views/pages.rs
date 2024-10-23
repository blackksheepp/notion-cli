use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub fn pages_view(
    content_width: &u16,
    content_height: &u16,
    x_center: &u16,
    y_search: &u16,
    pages: &indexmap::IndexMap<&str, Vec<&str>>,
    page_pos: usize,
    page_selected: bool,
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
        Print(" browse pages ".to_string()),
        ResetColor
    )
    .unwrap();

    let selected_page = pages.keys().nth(page_pos).unwrap();
    let contents = pages.get(selected_page).unwrap();

    for (i, (page, _contents)) in pages.iter().enumerate() {
        execute!(
            stdout(),
            MoveTo(
                x_center - (content_width / 2) + 10,
                y_search + 9 + (i as u16)
            ),
            SetForegroundColor(if i == page_pos {
                Color::White
            } else {
                Color::DarkGrey
            }),
            SetBackgroundColor(if i == page_pos {
                Color::Rgb {r: 49, g: 116, b: 143}
            } else {
                Color::Reset
            }),
            Print(format!(" {} ", page)),
            ResetColor
        )
        .unwrap();
    }

    for (i, content) in contents.iter().enumerate() {
        execute!(
            stdout(),
            MoveTo(
                x_center - (content_width / 2) + 35,
                y_search + 9 + (i as u16)
            ),
            SetForegroundColor(if i == table_pos {
                if i == 0 && !page_selected {
                    Color::DarkGrey
                } else {
                    if favorites.contains(content) {
                        Color::Black
                    } else {
                        Color::White
                    }
                }
            } else {
                Color::DarkGrey
            }),
            SetBackgroundColor(if i == table_pos && page_selected {
                if favorites.contains(content) {
                    Color::Rgb {
                        r: 252,
                        g: 186,
                        b: 3,
                    }
                } else {
                    Color::Rgb {r: 66, g: 36, b: 156}
                }
            } else {
                Color::Reset
            }),
            Print(format!(
                " {}{} ",
                if favorites.contains(content) { "*" } else { "" },
                content
            )),
            ResetColor
        )
        .unwrap();
    }
}
