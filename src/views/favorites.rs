use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub fn favorites_view(
    content_width: &u16,
    content_height: &u16,
    x_center: &u16,
    y_search: &u16,
    favorites: &Vec<&str>,
    favorites_pos: usize,
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
        Print(" browse favorites ".to_string()),
        ResetColor
    )
    .unwrap();

    let count = 7;
    let start = count * (favorites_pos / count);
    let end = if start + count > favorites.len() {
        favorites.len()
    } else {
        start + count
    };
    for (i, favorite) in favorites[start..end].iter().enumerate() {
        execute!(
            stdout(),
            MoveTo(
                x_center - (content_width / 2) + 10,
                y_search + 9 + (i as u16)
            ),
            SetForegroundColor(if i == (favorites_pos % count) {
                Color::Black
            } else {
                Color::DarkGrey
            }),
            SetBackgroundColor(if i == (favorites_pos % count) {
                Color::Rgb {
                    r: 252,
                    g: 186,
                    b: 3,
                }
            } else {
                Color::Reset
            }),
            Print(favorite),
            ResetColor
        )
        .unwrap();
    }
}
