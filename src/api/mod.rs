pub mod pages;
pub mod auth;
pub mod search;

use std::sync::Once;
use notion::NotionApi;

use crate::database::user::get_user;

static INIT: Once = Once::new();
static mut NOTION: Option<NotionApi> = None;

pub fn initialize_notion() {
    INIT.call_once(|| {
        let mut access_token = String::new();
        if let Some(user) = get_user().expect("Failed to get user from database") {
            access_token = user.access_token;
        }
        let client = NotionApi::new(access_token).expect("Failed to create Notion client");
        unsafe {
            NOTION = Some(client);
        }
    });
}

pub fn get_notion() -> &'static NotionApi {
    unsafe { NOTION.as_ref().expect("Database not initialized") }
}