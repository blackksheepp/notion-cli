use std::io::stdout;

use crossterm::{cursor::MoveTo, execute, style::{Print, SetForegroundColor, Color}};

use crate::components::controls::controls;

pub fn login_view(
    content_width: &u16,
    content_height: &u16,
    x_center: &u16,
    y_center: &u16,
    y_search: &u16,
) {
    for i in 2..content_height - 4 {
        execute!(
            stdout(),
            MoveTo(x_center - (content_width / 2) + 1, y_search + i + 1),
            Print(" ".repeat((content_width - 2) as usize)),
        )
        .unwrap();
    }
    let text = "press [L] to login with notion.so";
    let subtext = "select the pages & database you would like to use here";

    controls(false, false);
    
    execute!(
        stdout(),
        MoveTo(x_center - (text.len() as u16 / 2), y_center-0),
        SetForegroundColor(Color::White),
        Print(text),
        MoveTo(x_center - (subtext.len() as u16 / 2), y_center+2),
        SetForegroundColor(Color::DarkGrey),
        Print(subtext)
    ).unwrap()

}
