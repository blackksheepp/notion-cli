mod api;
mod components;
mod database;
mod utils;
mod views;

use api::{
    auth::logout,
    initialize_notion,
    search::{search_api, Object},
};
use components::{controls::controls, search::search_box};
use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use database::initialize_db;
use indexmap::IndexMap;
use utils::{dimentions::get_dimensions, search::match_search};
use views::{auth::auth_view, home::home_view, login::login_view, tables::tables_view};
use views::{favorites::favorites_view, pages::pages_view};

use dotenv::dotenv;
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::{io::stdout, vec};

extern crate lazy_static;

fn set_scroll_region(top: u16, bottom: u16) {
    let command = format!("\x1B[{};{}r", top + 1, bottom + 1);
    execute!(stdout(), Print(command)).unwrap();
}

const SECTIONS: [&str; 7] = [
    "login",
    "pages",
    "tables",
    "favorites",
    "settings",
    "auth",
    "home",
];

lazy_static::lazy_static! (
    pub static ref SECTION: Mutex<i32> = Mutex::new(0);
    pub static ref AUTHENTICATED: Mutex<bool> = Mutex::new(false);
);

pub fn set_section(value: usize) {
    let mut section = SECTION.lock().unwrap();
    *section = value as i32;
}

pub fn set_auth(value: bool) {
    let mut authenticated = AUTHENTICATED.lock().unwrap();
    *authenticated = value;
}

pub fn auth_completed() {
    set_section(6);
    render_content(false, &IndexMap::new(), None, None, None, &Vec::new(), None);
}

pub fn logout_completed() {
    set_section(0);
    render_content(false, &IndexMap::new(), None, None, None, &Vec::new(), None);
}

fn render_content(
    search_enabled: bool,
    pages: &indexmap::IndexMap<&str, Vec<&str>>,
    page_pos: Option<usize>,
    page_selected: Option<bool>,
    table_pos: Option<usize>,
    favorites: &Vec<&str>,
    favorites_pos: Option<usize>,
) {
    let section = SECTIONS[*SECTION.lock().unwrap() as usize];
    if search_enabled {
        return;
    }

    let page_pos = page_pos.unwrap_or(0);
    let page_selected = page_selected.unwrap_or(false);

    let table_pos = table_pos.unwrap_or(0);

    let (content_width, content_height, x_center, y_center) = get_dimensions();

    let y_search = y_center - (content_height / 2) + 1;

    let favorites_pos = favorites_pos.unwrap_or(0);

    let tables: Vec<&str> = pages.values().flat_map(|v| v.iter()).cloned().collect();

    match section {
        "login" => login_view(
            &content_width,
            &content_height,
            &x_center,
            &y_center,
            &y_search,
        ),
        "auth" => auth_view(
            &content_width,
            &content_height,
            &x_center,
            &y_center,
            &y_search,
        ),
        "home" => home_view(
            &content_width,
            &content_height,
            &x_center,
            &y_center,
            &y_search,
        ),
        "pages" => pages_view(
            &content_width,
            &content_height,
            &x_center,
            &y_search,
            pages,
            page_pos,
            page_selected,
            table_pos,
            favorites,
        ),
        "tables" => tables_view(
            &content_width,
            &content_height,
            &x_center,
            &y_search,
            &tables,
            table_pos,
            favorites,
        ),
        "favorites" => favorites_view(
            &content_width,
            &content_height,
            &x_center,
            &y_search,
            &favorites,
            favorites_pos,
        ),
        _ => {
            execute!(stdout(), MoveTo(x_center, y_center), Print("Home")).unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    initialize_db("./ncli.db");
    initialize_notion();

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

    let mut search_items = HashMap::new();
    let mut search_input = String::new();
    let mut search_pos: usize = 0;

    let mut page_pos = 0 as usize;
    let mut page_selected = false;
    let mut table_pos = 0 as usize;

    let mut pages = indexmap::IndexMap::new();
    pages.insert("writting", vec!["daily journal", "dears diary", "calendar"]);
    pages.insert("academic", vec!["class 12th", "physics", "maths"]);
    pages.insert("programming", vec!["projects", "freelnace", "community"]);
    pages.insert("hobbies", vec!["japanese", "books", "music"]);

    let tables: Vec<&str> = pages.values().flat_map(|v| v.iter()).cloned().collect();

    let favorites = vec!["hobbies", "physics", "calendar"];
    let mut favorite_pos = 0 as usize;

    fn render(
        search_enabled: bool,
        search_input: &String,
        search_items: &HashMap<String, Object>,
        search_pos: &usize,
        pages: &indexmap::IndexMap<&str, Vec<&str>>,
        favorites: &Vec<&str>,
        favorite_pos: usize,
    ) {
        controls(search_enabled, true);
        search_box(search_enabled, search_input, search_items, search_pos, None);
        render_content(
            search_enabled,
            &pages,
            None,
            None,
            None,
            favorites,
            Some(favorite_pos),
        );
    }

    render(
        search_enabled,
        &search_input,
        &search_items,
        &search_pos,
        &pages,
        &favorites,
        favorite_pos,
    );

    let mut search_results = search_api(None).await;

    loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char(c) if search_enabled => {
                        search_input.push(c);
                        let mut search_results = search_api(Some(search_input)).await;
                        search_items = match_search(&search_input, search_results);
                        search_pos = 0;

                        search_box(
                            search_enabled,
                            &search_input,
                            &search_items,
                            &search_pos,
                            Some(true),
                        );
                    }
                    KeyCode::Backspace if search_enabled => {
                        search_input.pop();
                        let mut search_results = search_api(Some(search_input)).await;
                        search_items = match_search(&search_input, search_results);
                        search_pos = 0;

                        search_box(
                            search_enabled,
                            &search_input,
                            &search_results.unwrap(),
                            &search_pos,
                            Some(true),
                        );
                    }
                    KeyCode::Char('s') => {
                        search_enabled = !search_enabled;
                        let mut search_results = search_api(Some(search_input)).await;
                        search_items = match_search(&search_input, search_results);

                        render(
                            search_enabled,
                            &search_input,
                            &search_results.unwrap(),
                            &search_pos,
                            &pages,
                            &favorites,
                            favorite_pos,
                        );
                    }
                    KeyCode::Esc => {
                        if search_enabled {
                            search_input.clear();
                            search_enabled = !search_enabled;

                            render(
                                search_enabled,
                                &search_input,
                                &search_results.unwrap(),
                                &search_pos,
                                &pages,
                                &favorites,
                                favorite_pos,
                            );
                        } else {
                            if *SECTION.lock().unwrap() == 1 && page_selected {
                                page_selected = false;
                                render_content(
                                    search_enabled,
                                    &pages,
                                    Some(page_pos),
                                    None,
                                    None,
                                    &favorites,
                                    Some(favorite_pos),
                                );
                            } else {
                                set_section(6);
                                render_content(
                                    search_enabled,
                                    &pages,
                                    None,
                                    None,
                                    None,
                                    &favorites,
                                    Some(favorite_pos),
                                );
                            }
                        }
                    }
                    KeyCode::Up => {
                        if search_enabled {
                            let mut search_results = search_api(Some(search_input)).await;
                            search_items = match_search(&search_input, search_results);
                            
                            if search_pos + 1 < search_items.len() {
                                search_pos += 1
                            } else {
                                search_pos = 0
                            }

                            search_box(
                                search_enabled,
                                &search_input,
                                &search_items,
                                &search_pos,
                                None,
                            );
                        } else {
                            if *SECTION.lock().unwrap() == 1 {
                                if page_selected {
                                    let tables_len = pages.get_index(page_pos).unwrap().1.len();
                                    table_pos = if table_pos > 0 {
                                        table_pos - 1
                                    } else {
                                        tables_len - 1
                                    };
                                    render_content(
                                        search_enabled,
                                        &pages,
                                        Some(page_pos),
                                        Some(page_selected),
                                        Some(table_pos),
                                        &favorites,
                                        Some(favorite_pos),
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
                                        &pages,
                                        Some(page_pos),
                                        None,
                                        None,
                                        &favorites,
                                        Some(favorite_pos),
                                    )
                                }
                            } else if *SECTION.lock().unwrap() == 2 {
                                let tables_len = tables.len();
                                table_pos = if table_pos > 0 {
                                    table_pos - 1
                                } else {
                                    tables_len - 1
                                };
                                render_content(
                                    search_enabled,
                                    &pages,
                                    None,
                                    None,
                                    Some(table_pos),
                                    &favorites,
                                    Some(favorite_pos),
                                )
                            } else if *SECTION.lock().unwrap() == 3 {
                                let favorites_len = favorites.len();
                                favorite_pos = if favorite_pos > 0 {
                                    favorite_pos - 1
                                } else {
                                    favorites_len - 1
                                };
                                render_content(
                                    search_enabled,
                                    &pages,
                                    None,
                                    None,
                                    Some(table_pos),
                                    &favorites,
                                    Some(favorite_pos),
                                )
                            }
                        }
                    }
                    KeyCode::Down => {
                        if search_enabled {
                            let mut search_results = search_api(Some(search_input)).await;
                            search_items = match_search(&search_input, search_results);

                            if search_pos > 0 {
                                search_pos -= 1
                            } else {
                                search_pos = search_items.len() - 1
                            }

                            search_box(
                                search_enabled,
                                &search_input,
                                &search_items,
                                &search_pos,
                                None,
                            );
                        } else {
                            if *SECTION.lock().unwrap() == 1 {
                                if page_selected {
                                    let tables_len = pages.get_index(page_pos).unwrap().1.len();
                                    table_pos = if table_pos < tables_len - 1 {
                                        table_pos + 1
                                    } else {
                                        0
                                    };
                                    render_content(
                                        search_enabled,
                                        &pages,
                                        Some(page_pos),
                                        Some(page_selected),
                                        Some(table_pos),
                                        &favorites,
                                        Some(favorite_pos),
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
                                        &pages,
                                        Some(page_pos),
                                        None,
                                        None,
                                        &favorites,
                                        Some(favorite_pos),
                                    );
                                }
                            } else if *SECTION.lock().unwrap() == 2 {
                                let tables_len = tables.len();
                                table_pos = if table_pos < tables_len - 1 {
                                    table_pos + 1
                                } else {
                                    0
                                };
                                render_content(
                                    search_enabled,
                                    &pages,
                                    None,
                                    None,
                                    Some(table_pos),
                                    &favorites,
                                    Some(favorite_pos),
                                )
                            } else if *SECTION.lock().unwrap() == 3 {
                                let favorites_len = favorites.len();
                                favorite_pos = if favorite_pos < favorites_len - 1 {
                                    favorite_pos + 1
                                } else {
                                    0
                                };
                                render_content(
                                    search_enabled,
                                    &pages,
                                    None,
                                    None,
                                    Some(table_pos),
                                    &favorites,
                                    Some(favorite_pos),
                                )
                            }
                        }
                    }
                    KeyCode::Enter if *SECTION.lock().unwrap() == 1 => {
                        if !page_selected {
                            page_selected = true;
                            table_pos = 0;
                        }
                        render_content(
                            search_enabled,
                            &pages,
                            Some(page_pos),
                            Some(page_selected),
                            Some(table_pos),
                            &favorites,
                            Some(favorite_pos),
                        );
                    }
                    KeyCode::Char('l') => {
                        if !*AUTHENTICATED.lock().unwrap() {
                            set_section(5);
                            render_content(
                                search_enabled,
                                &pages,
                                None,
                                None,
                                None,
                                &favorites,
                                Some(favorite_pos),
                            );
                        } else {
                            logout();
                        }
                    }
                    KeyCode::Char('h') => {
                        set_section(6);
                        render_content(
                            search_enabled,
                            &pages,
                            None,
                            None,
                            None,
                            &favorites,
                            Some(favorite_pos),
                        );
                    }
                    KeyCode::Char('p') => {
                        set_section(1);
                        render_content(
                            search_enabled,
                            &pages,
                            None,
                            None,
                            None,
                            &favorites,
                            Some(favorite_pos),
                        );
                    }
                    KeyCode::Char('t') => {
                        table_pos = 0;
                        set_section(2);
                        render_content(
                            search_enabled,
                            &pages,
                            None,
                            None,
                            None,
                            &favorites,
                            Some(favorite_pos),
                        );
                    }
                    KeyCode::Char('f') => {
                        favorite_pos = 0;
                        set_section(3);
                        render_content(
                            search_enabled,
                            &pages,
                            None,
                            None,
                            None,
                            &favorites,
                            Some(favorite_pos),
                        );
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
                &pages,
                &favorites,
                favorite_pos,
            );
        }
    }

    disable_raw_mode().unwrap();
    execute!(stdout(), Show, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
}
