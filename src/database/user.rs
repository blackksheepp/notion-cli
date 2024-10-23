use crate::database::get_connection;
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub access_token: String,
    pub bot_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub workspace_id: String,
}

pub fn save_user(user: User) {
    let conn = get_connection();

    conn.execute("DELETE FROM user", [])
        .expect("Failed to clear user table");

    conn.execute(
        "INSERT INTO user (access_token, bot_id, user_id, user_name, user_email, workspace_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            user.access_token,
            user.bot_id,
            user.user_id,
            user.user_name,
            user.user_email,
            user.workspace_id
        ],
    ).expect("Failed to save user to database");
}

pub fn get_user() -> Result<Option<User>> {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT access_token, bot_id, user_id, user_name, user_email, workspace_id FROM user LIMIT 1")?;

    match stmt.query_row([], |row| {
        Ok(User {
            access_token: row.get(0)?,
            bot_id: row.get(1)?,
            user_id: row.get(2)?,
            user_name: row.get(3)?,
            user_email: row.get(4)?,
            workspace_id: row.get(5)?,
        })
    }) {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_access_token() -> String {
    match get_user() {
        Ok(Some(user)) => user.access_token,
        _ => String::new(),
    }
}