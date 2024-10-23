use std::io::stdout;
use webbrowser;

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, SetForegroundColor},
};

use crate::utils::http::{initialize_server, start_server};


pub fn auth_view(
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
    let text = "authorizing...";
    let auth_url = "http://127.0.0.1:55678/login";
    let subtext = "open link for manual login";

    initialize_server();
    start_server();

    if let Err(e) = webbrowser::open(&auth_url) {
        eprintln!("Failed to open browser: {}", e);
    }

    execute!(
        stdout(),
        MoveTo(x_center - (text.len() as u16 / 2), y_center - 0),
        SetForegroundColor(Color::White),
        Print(text),
        MoveTo(x_center - (auth_url.len() as u16 / 2), y_center + 8),
        SetForegroundColor(Color::Cyan),
        Print(auth_url),
        MoveTo(x_center - (subtext.len() as u16 / 2), y_center + 9),
        SetForegroundColor(Color::DarkGrey),
        Print(subtext)
    )
    .unwrap()
}
