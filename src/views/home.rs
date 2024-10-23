use std::io::stdout;

use crossterm::{cursor::MoveTo, execute, style::Print};

use crate::{components::controls::controls, utils::controls::write_ctrl, AUTHENTICATED};

pub fn home_view(
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

    controls(false, false);

    let options = vec!["[f]avorites", "[p]ages", "[t]ables", "[r]ecents"];

    for (i, option) in options.iter().enumerate() {
        let x_option = x_center - (options[1].len() as u16 / 2) - 3;
        let y_option = y_center + (i as u16 * 2) - options.len() as u16 / 2 - 1;
        write_ctrl(option, x_option, y_option);
    }
}
