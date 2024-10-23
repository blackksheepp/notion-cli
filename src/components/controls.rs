use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{
    api::auth,
    utils::{controls::write_ctrl, dimentions::get_dimensions},
    AUTHENTICATED,
};

pub fn controls(search_enabled: bool, clear: bool) {
    let (content_width, content_height, x_center, y_center) = get_dimensions();

    let y_line = y_center + (content_height / 2) - 2;

    if clear {
        execute!(stdout(), Clear(ClearType::All)).unwrap();
    }

    execute!(
        stdout(),
        MoveTo(x_center - (content_width / 2), y_line),
        SetForegroundColor(Color::DarkGrey),
        Print("─".repeat(content_width as usize)),
        ResetColor
    )
    .unwrap();

    let mut left_ctrl = vec!["[m]enu"];
    if search_enabled {
        left_ctrl.pop();
        left_ctrl.push("[esc] exit search");
    }

    let mut left_ctrl_space = 0;
    for ctrl in left_ctrl {
        let x_ctrl = x_center - (content_width / 2) + left_ctrl_space as u16;
        let y_ctrl = y_center + (content_height / 2) - 1;
        write_ctrl(ctrl, x_ctrl, y_ctrl);

        left_ctrl_space += ctrl.len() as u16 + 1;
    }

    if !search_enabled {
        let auth_button = if *AUTHENTICATED.lock().unwrap() {
            "[l]ogout"
        } else {
            " [l]ogin"
        };

        let mut right_ctrl = vec![auth_button, "[h]elp", "[q]uit"];
        right_ctrl.reverse();

        let mut right_ctrl_space = 0;
        for (ctrl, i) in right_ctrl.iter().zip(0..) {
            right_ctrl_space += ctrl.len() as u16;
            right_ctrl_space += if i != 0 { 1 } else { 0 };

            let x_ctrl = x_center + (content_width / 2) - right_ctrl_space as u16;
            let y_ctrl = y_center + (content_height / 2) - 1;
            write_ctrl(ctrl, x_ctrl, y_ctrl);
        }
    }
}
