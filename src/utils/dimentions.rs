
const MAX_WIDTH: u16 = 60;
const MAX_HEIGHT: u16 = 30;

use crossterm::terminal::size;

// get terminal dimensions
pub fn get_dimensions() -> (u16, u16, u16, u16) {
    let (cols, rows) = size().unwrap();

    let content_width = cols.min(MAX_WIDTH);
    let content_height = rows.min(MAX_HEIGHT);

    let x_center = cols / 2;
    let y_center = rows / 2;

    (content_width, content_height, x_center, y_center)
}