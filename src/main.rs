use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{io::stdout, vec};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

const MAX_WIDTH: u16 = 60;
const MAX_HEIGHT: u16 = 30;
const PAGES: [&str; 10] = [
    "Journal",
    "Notes",
    "Projects",
    "Tasks",
    "Reminders",
    "Archives",
    "Photos",
    "Files",
    "Links",
    "Contacts",
];

// fuzzy search query in given options
fn match_search(query: &str, options: &[&str]) -> Vec<String> {
    if query.is_empty() {
        return options.iter().map(|&option| option.to_string()).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(&str, i64)> = options
        .iter()
        .filter_map(|&option| {
            matcher
                .fuzzy_match(option, query)
                .map(|score| (option, score))
        })
        .collect();

    results.sort_by_key(|&(_, score)| -score);
    results
        .iter()
        .map(|(option, _)| option.to_string())
        .collect()
}

// set scroll region out of bounds
fn set_scroll_region(top: u16, bottom: u16) {
    let command = format!("\x1B[{};{}r", top + 1, bottom + 1);
    execute!(stdout(), Print(command)).unwrap();
}

// write control commands
fn write_ctrl(ctrl: &str, x: u16, y: u16) {
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

// get terminal dimensions
fn get_dimensions() -> (u16, u16, u16, u16) {
    let (cols, rows) = size().unwrap();

    let content_width = cols.min(MAX_WIDTH);
    let content_height = rows.min(MAX_HEIGHT); // Use the terminal height

    let x_center = cols / 2;
    let y_center = rows / 2;

    (content_width, content_height, x_center, y_center)
}

fn render_search_box(
    search_enabled: bool,
    search_input: &str,
    search_items: &Vec<String>,
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
                        SetForegroundColor(Color::Green),
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
                    SetBackgroundColor(Color::Green),
                    Print(format!(" {}", page.to_string()).bold()),
                    SetBackgroundColor(Color::Green),
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
}

const SECTIONS: [&str; 5] = ["home", "pages", "databases", "favorites", "settings"];

fn render_content(
    search_enabled: bool,
    section: &str,
    pages: &indexmap::IndexMap<&str, Vec<&str>>,
    page_pos: Option<usize>,
    page_selected: Option<bool>,
    database_pos: Option<usize>,
) {
    if search_enabled {
        return;
    }

    let page_pos = page_pos.unwrap_or(0);
    let page_selected = page_selected.unwrap_or(false);

    let database_pos = database_pos.unwrap_or(0);

    let (content_width, content_height, x_center, y_center) = get_dimensions();

    let y_search = y_center - (content_height / 2) + 1;

    let favorites = vec!["hobbies", "physics", "calendar"];

    match section {
        "home" => {
            for i in 2..content_height - 4 {
                execute!(
                    stdout(),
                    MoveTo(x_center - (content_width / 2) + 1, y_search + i + 1),
                    Print(" ".repeat((content_width - 2) as usize)),
                )
                .unwrap();
            }

            let options = vec!["[f]avorites", "[p]ages", "[d]atabases", "[r]ecents"];

            for (i, option) in options.iter().enumerate() {
                let x_option = x_center - (options[1].len() as u16 / 2) - 3;
                let y_option = y_center + (i as u16 * 2) - options.len() as u16 / 2 - 1;
                write_ctrl(option, x_option, y_option);
            }
        }
        "pages" => {
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
                    SetBackgroundColor(if i == page_pos && !page_selected {
                        Color::Green
                    } else {
                        Color::Reset
                    }),
                    Print(format!(" {} ", page)),
                    ResetColor
                )
                .unwrap();
            }

            let selected_page = pages.keys().nth(page_pos).unwrap();
            let contents = pages.get(selected_page).unwrap();
            for (i, content) in contents.iter().enumerate() {
                execute!(
                    stdout(),
                    MoveTo(
                        x_center - (content_width / 2) + 35,
                        y_search + 9 + (i as u16)
                    ),
                    SetForegroundColor(if i == database_pos || !page_selected {
                        Color::White
                    } else {
                        Color::DarkGrey
                    }),
                    SetBackgroundColor(if i == database_pos && page_selected {
                        Color::Green
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
        _ => {
            execute!(stdout(), MoveTo(x_center, y_center), Print("Home")).unwrap();
        }
    }
}

fn render_controls(search_enabled: bool) {
    let (content_width, content_height, x_center, y_center) = get_dimensions();

    let y_line = y_center + (content_height / 2) - 2;
    execute!(
        stdout(),
        Clear(ClearType::All),
        SetForegroundColor(Color::DarkGrey),
        MoveTo(x_center - (content_width / 2), y_line),
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
        let mut right_ctrl = vec!["[l]ogin", "[h]elp", "[q]uit"];
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

fn main() {
    // let mut count = 0;
    let size_changed = Arc::new(Mutex::new(false));
    let size_changed_clone = size_changed.clone();

    enable_raw_mode().unwrap();
    execute!(stdout(), Hide).unwrap();

    let (_cols, rows) = size().unwrap();
    set_scroll_region(0, rows - 1);

    let _size_thread = thread::spawn(move || {
        let mut prev_size = size().unwrap();
        loop {
            let current_size = size().unwrap();
            if current_size != prev_size {
                prev_size = current_size;
                *size_changed_clone.lock().unwrap() = true;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    let mut search_enabled = false;

    let mut search_items = Vec::new();
    let mut search_input = String::new();
    let mut search_pos: usize = 0;

    let mut section = SECTIONS[0];
    let mut page_pos = 0 as usize;
    let mut page_selected = false;
    let mut database_pos = 0 as usize;

    let mut pages = indexmap::IndexMap::new();
    pages.insert("writting", vec!["daily journal", "dears diary", "calendar"]);
    pages.insert("academic", vec!["class 12th", "physics", "maths"]);
    pages.insert("programming", vec!["projects", "freelnace", "community"]);
    pages.insert("hobbies", vec!["japanese", "books", "music"]);

    fn render(
        search_enabled: bool,
        search_input: &String,
        search_items: &Vec<String>,
        search_pos: &usize,
        section: &str,
        pages: &indexmap::IndexMap<&str, Vec<&str>>,
    ) {
        render_controls(search_enabled);
        render_search_box(search_enabled, search_input, search_items, search_pos, None);
        render_content(search_enabled, section, &pages, None, None, None);
    }

    render(
        search_enabled,
        &search_input,
        &search_items,
        &search_pos,
        section,
        &pages,
    );

    loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char(c) if search_enabled => {
                        search_input.push(c);
                        search_items = match_search(&search_input, &PAGES);
                        search_pos = 0;

                        render_search_box(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            Some(true),
                        );
                    }
                    KeyCode::Backspace if search_enabled => {
                        search_input.pop();
                        search_items = match_search(&search_input, &PAGES);
                        search_pos = 0;

                        render_search_box(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            Some(true),
                        );
                    }
                    KeyCode::Char('s') => {
                        search_enabled = !search_enabled;
                        search_items = match_search(&search_input, &PAGES);

                        render(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            section,
                            &pages,
                        );
                    }
                    KeyCode::Esc if search_enabled => {
                        search_input.clear();
                        search_enabled = !search_enabled;

                        render(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            section,
                            &pages,
                        );
                    }
                    KeyCode::Esc if section == SECTIONS[1] => {
                        if page_selected {
                            page_selected = false;
                            render_content(
                                search_enabled,
                                section,
                                &pages,
                                Some(page_pos),
                                None,
                                None,
                            );
                        } else {
                            section = SECTIONS[0];
                            render_content(search_enabled, section, &pages, None, None, None);
                        }
                    }
                    KeyCode::Up if search_enabled => {
                        search_items = match_search(&search_input, &PAGES);
                        if search_pos + 1 < search_items.len() {
                            search_pos += 1
                        } else {
                            search_pos = 0
                        }

                        render_search_box(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            None,
                        );
                    }
                    KeyCode::Down if search_enabled => {
                        search_items = match_search(&search_input, &PAGES);
                        if search_pos > 0 {
                            search_pos -= 1
                        } else {
                            search_pos = search_items.len() - 1
                        }

                        render_search_box(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            None,
                        );
                    }

                    KeyCode::Up if section == SECTIONS[1] => {
                        if page_selected {
                            let database_len = pages.get_index(page_pos).unwrap().1.len();
                            database_pos = if database_pos > 0 {
                                database_pos - 1
                            } else {
                                database_len - 1
                            };
                            render_content(
                                search_enabled,
                                section,
                                &pages,
                                Some(page_pos),
                                Some(page_selected),
                                Some(database_pos),
                            )
                        } else {
                            let pages_len = pages.len();
                            page_pos = if page_pos > 0 {
                                page_pos - 1
                            } else {
                                pages_len - 1
                            };
                            render_content(
                                search_enabled,
                                section,
                                &pages,
                                Some(page_pos),
                                None,
                                None,
                            )
                        }
                    }
                    KeyCode::Down if section == SECTIONS[1] => {
                        if page_selected {
                            let database_len = pages.get_index(page_pos).unwrap().1.len();
                            database_pos = if database_pos < database_len - 1 {
                                database_pos + 1
                            } else {
                                0
                            };
                            render_content(
                                search_enabled,
                                section,
                                &pages,
                                Some(page_pos),
                                Some(page_selected),
                                Some(database_pos),
                            )
                        } else {
                            let page_len = pages.len();
                            page_pos = if page_pos < page_len - 1 {
                                page_pos + 1
                            } else {
                                0
                            };
                            render_content(
                                search_enabled,
                                section,
                                &pages,
                                Some(page_pos),
                                None,
                                None,
                            );
                        }
                    }

                    KeyCode::Enter if section == SECTIONS[1] => {
                        if !page_selected {
                            page_selected = true
                        }
                        render_content(
                            search_enabled,
                            "pages",
                            &pages,
                            Some(page_pos),
                            Some(page_selected),
                            Some(database_pos),
                        );
                    }

                    KeyCode::Char('p') => {
                        section = SECTIONS[1];
                        render_content(search_enabled, section, &pages, None, None, None);
                    }
                    KeyCode::Char('h') => {
                        section = SECTIONS[0];
                        render_content(search_enabled, section, &pages, None, None, None);
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        if *size_changed.lock().unwrap() {
            *size_changed.lock().unwrap() = false;
            let (_cols, rows) = size().unwrap();
            set_scroll_region(0, rows - 1);
            render(
                search_enabled,
                &search_input,
                &search_items,
                &search_pos,
                section,
                &pages,
            );
        }
    }

    disable_raw_mode().unwrap();
    execute!(stdout(), Show, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
}
