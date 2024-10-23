use base64;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::{env, fs};

use crate::{
    auth_completed,
    database::user::{save_user, User},
    logout_completed, set_auth,
};

pub async fn authorize(code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client_id = env::var("OAUTH_CLIENT_ID")?;
    let client_secret = env::var("OAUTH_CLIENT_SECRET")?;
    let redirect_uri = env::var("OAUTH_REDIRECT_URI")?;

    let encoded = base64::encode(format!("{}:{}", client_id, client_secret));

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", encoded))?,
    );

    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": code,
        "redirect_uri": redirect_uri,
    });

    let response = client
        .post("https://api.notion.com/v1/oauth/token")
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        let json_data = response.json::<serde_json::Value>().await?;

        let user = User {
            access_token: json_data["access_token"].as_str().unwrap().to_string(),
            bot_id: json_data["bot_id"].as_str().unwrap().to_string(),
            user_id: json_data["owner"]["user"]["id"]
                .as_str()
                .unwrap()
                .to_string(),
            user_name: json_data["owner"]["user"]["name"]
                .as_str()
                .unwrap()
                .to_string(),
            user_email: json_data["owner"]["user"]["person"]["email"]
                .as_str()
                .unwrap()
                .to_string(),
            workspace_id: json_data["workspace_id"].as_str().unwrap().to_string(),
        };

        save_user(user);
        set_auth(true);
        auth_completed();
    } else {
        println!("Failed to get token: {:?}", response.status());
    }

    Ok(())
}

pub fn logout() {
    match fs::remove_file("./ncli.db") {
        Ok(_) => set_auth(false),
        Err(_) => set_auth(false),
    };

    logout_completed();
}
