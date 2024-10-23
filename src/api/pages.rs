use std::fs;

use crate::database::user::get_access_token;

use reqwest::Client;


pub async fn get_pages() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let access_token = get_access_token();
    if access_token.is_empty() {
        // logout();
    }

    let client = Client::new();

    let response = client
        .post("https://api.notion.com/v1/search")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Notion-Version", "2022-06-28")
        .json(&serde_json::json!({  "query": "" }))
        .send()
        .await?;

    let mut json_data = serde_json::json!({});
    if response.status().is_success() {
        json_data = response.json::<serde_json::Value>().await?;
    }

    fs::write("output.txt", json_data.to_string()).unwrap();

    return Ok(json_data);
}



