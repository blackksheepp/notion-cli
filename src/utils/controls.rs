use std::io::stdout;

use crossterm::{cursor::MoveTo, execute, style::{Color, Print, ResetColor, SetForegroundColor}};

// write control commands
pub fn write_ctrl(ctrl: &str, x: u16, y: u16) {
    let mut char_indices = ctrl.char_indices();

    match char_indices.find(|(_, c)| c.to_string() == '['.to_string()) {
        Some((i, _)) => match char_indices.find(|(_, c)| c.to_string() == ']'.to_string()) {
            Some((j, _)) => {
                let left = &ctrl[..i + 1];
                let middle = &ctrl[i + 1..j];
                let right = &ctrl[j - 1 + 1..];

                execute!(
                    stdout(),
                    MoveTo(x, y),
                    SetForegroundColor(Color::DarkGrey),
                    Print(left),
                    SetForegroundColor(Color::White),
                    Print(middle),
                    SetForegroundColor(Color::DarkGrey),
                    Print(right),
                    ResetColor
                )
                .unwrap();
            }
            None => {}
        },
        None => {}
    }
}
