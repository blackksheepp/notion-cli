use std::{collections::HashMap, io::stdout};

use crossterm::{cursor::MoveTo, execute, style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize}};

use crate::{api::{pages::get_pages, search::Object}, utils::{controls::write_ctrl, dimentions::get_dimensions}};

pub fn search_box(
    search_enabled: bool,
    search_input: &str,
    search_items: &HashMap<String, Object>,
    search_pos: &usize,
    input_update: Option<bool>,
) {
    
    let input_update = input_update.unwrap_or(false);

    let (content_width, content_height, x_center, y_center) = get_dimensions();

    let search_box_width = content_width / 2;
    let x_search = x_center - (content_width / 4) - (search_box_width / 2) + 1;
    let y_search = y_center - (content_height / 2) + 1;

    if search_enabled {
        // Search box top
        if !input_update {
            execute!(
                stdout(),
                SetForegroundColor(Color::DarkGrey),
                MoveTo(x_center - (content_width / 2), y_search),
                Print("┌".to_string() + &"─".repeat((content_width - 2) as usize) + "┐"),
            )
            .unwrap();
        }

        let search_box_height = 19;
        for i in 0..search_box_height {
            // Search box boundary
            if !input_update {
                execute!(
                    stdout(),
                    MoveTo(x_center - (content_width / 2), y_search + i + 1),
                    Print("│".to_string()),
                    MoveTo(x_center + (content_width / 2) - 1, y_search + i + 1),
                    Print("│"),
                )
                .unwrap();
            }

            // Search box content
            execute!(
                stdout(),
                MoveTo(x_center - (content_width / 2) + 1, y_search + i + 1),
                Print(" ".repeat((content_width - 2) as usize)),
            )
            .unwrap();

            // Search bar
            if i == search_box_height - 2 {
                execute!(
                    stdout(),
                    MoveTo(x_center - (content_width / 2) + 1, y_search + i),
                    Print("─".repeat(content_width as usize - 2)),
                )
                .unwrap();
            } else if i == search_box_height - 1 {
                execute!(
                    stdout(),
                    MoveTo(x_center - (content_width / 2) + 2, y_search + i),
                    SetForegroundColor(Color::DarkGrey),
                )
                .unwrap();

                if search_input.is_empty() {
                    execute!(stdout(), Print("search...".to_string()), ResetColor).unwrap();
                } else {
                    let mut search_input_text = search_input.to_string();

                    if search_input.len() > (search_box_width as usize * 2 - 7) {
                        search_input_text = "...".to_string()
                            + &search_input
                                [search_input.len() - (search_box_width as usize * 2 - 7)..]
                                .to_string();
                    }

                    execute!(
                        stdout(),
                        SetForegroundColor(Color::Rgb {r: 49, g: 116, b: 143}),
                        Print(search_input_text),
                        ResetColor
                    )
                    .unwrap();
                }
            }
        }

        // Search items / results
        for (si, page) in search_items.iter().enumerate() {
            if si == *search_pos {
                execute!(
                    stdout(),
                    MoveTo(
                        x_center - (content_width / 2) + 2,
                        y_search + search_box_height - 3 - (si as u16)
                    ),
                    SetForegroundColor(Color::White),
                    SetBackgroundColor(Color::Rgb {r: 49, g: 116, b: 143}),
                    Print(format!(" {}", page.to_string()).bold()),
                    SetBackgroundColor(Color::Rgb {r: 49, g: 116, b: 143}),
                    SetForegroundColor(Color::DarkGrey),
                    Print(" ".repeat((content_width - 2) as usize - page.len() as usize - 3)),
                    ResetColor
                )
                .unwrap()
            } else {
                execute!(
                    stdout(),
                    MoveTo(
                        x_center - (content_width / 2) + 2,
                        y_search + search_box_height - 3 - (si as u16)
                    ),
                    SetForegroundColor(Color::White),
                    Print(page.to_string()),
                    ResetColor
                )
                .unwrap()
            }
        }

        // Search box bottom
        execute!(
            stdout(),
            MoveTo(x_center - (content_width / 2), y_search + search_box_height),
            SetForegroundColor(Color::DarkGrey),
            Print("└".to_string() + &"─".repeat((content_width - 2) as usize) + "┘"),
            ResetColor
        )
        .unwrap();
    } else {
        // Search box
        execute!(
            stdout(),
            SetForegroundColor(Color::DarkGrey),
            MoveTo(x_center - (content_width / 2), y_search),
            Print("┌".to_string() + &"─".repeat((content_width - 2) as usize) + "┐"),
            MoveTo(x_center - (content_width / 2), y_search + 1),
            Print("│".to_string() + &" ".repeat((content_width - 2) as usize) + "│"),
            MoveTo(x_center - (content_width / 2), y_search + 2),
            Print("└".to_string() + &"─".repeat((content_width - 2) as usize) + "┘"),
            ResetColor
        )
        .unwrap();

        let search_icon = '\u{2315}';
        let search_text = format!(
            "{}{}{}",
            "[s]earch",
            " ".repeat(search_box_width as usize * 2 - 12),
            search_icon.bold()
        );
        write_ctrl(&search_text, x_search, y_search + 1);
    }

    get_pages();
}
