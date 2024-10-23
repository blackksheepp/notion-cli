pub mod user;

use rusqlite::Connection;
use user::get_user;
use std::sync::Once;

use crate::{auth_completed, set_auth};

static INIT: Once = Once::new();
static mut DATABASE: Option<Connection> = None;

pub fn initialize_db(db_path: &str) {
    INIT.call_once(|| {
        let conn = Connection::open(db_path).expect("Failed to open database");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                access_token TEXT NOT NULL,
                bot_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                user_name TEXT NOT NULL,
                user_email TEXT NOT NULL,
                workspace_id TEXT NOT NULL
            )",
            [],
        ).expect("Failed to create table");
        unsafe {
            DATABASE = Some(conn);
        }
    });

    if let Some(_user) = get_user().unwrap() {
        set_auth(true);
        auth_completed();
    }
}

pub fn get_connection() -> &'static Connection {
    unsafe { DATABASE.as_ref().expect("Database not initialized") }
}
