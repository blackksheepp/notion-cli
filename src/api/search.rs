use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};use reqwest::Client;
use serde_json::Value;

use crate::database::user::get_access_token;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub children: Vec<Object>,
}

async fn get_parent_id(client: &Client, parent: Value) -> Result<Option<String>, Box<dyn Error>> {
    if parent["type"] == "workspace" {
        return Ok(None);
    } else {
        if parent["type"] == "page_id" {
            return Ok(Some(parent["page_id"].as_str().unwrap().to_string()));
        } else if parent["type"] == "database_id" {
            return Ok(Some(parent["database_id"].as_str().unwrap().to_string()));
        } else if parent["type"] == "block_id" {
            let mut current_block_id = parent["block_id"].as_str().unwrap().to_string();
            loop {
                let response = client
                    .get(format!(
                        "https://api.notion.com/v1/blocks/{}",
                        current_block_id
                    ))
                    .header("Authorization", format!("Bearer {}", get_access_token()))
                    .header("Notion-Version", "2022-06-28")
                    .send()
                    .await?;

                let mut result = serde_json::json!({});
                if response.status().is_success() {
                    result = response.json::<serde_json::Value>().await?;
                }

                let parent_type = result["parent"]["type"].as_str().unwrap().to_string();
                if parent_type == "page_id".to_string() {
                    return Ok(Some(
                        result["parent"]["page_id"].as_str().unwrap().to_string(),
                    ));
                } else if parent_type == "block_id".to_string() {
                    current_block_id = result["parent"]["block_id"].as_str().unwrap().to_string();
                } else {
                    return Err("Unexpected parent type".into());
                }
            }
        } else {
            return Err("Invalid parent type".into());
        }
    }
}

fn get_title(object_type: String, result: &Value) -> Result<String, Box<dyn Error>> {
    let title: Option<Value> = if object_type == "page" {
        let parent_type = result["parent"]["type"].as_str().unwrap().to_string();
        if parent_type == "page_id" || parent_type == "block_id" {
            Some(result["properties"]["title"]["title"][0]["plain_text"].clone())
        } else {
            let mut prop_title = None;
            for prop in result["properties"]
                .as_object()
                .unwrap()
                .keys()
                .map(|x| x.as_str().to_string())
                .collect::<Vec<String>>()
            {
                if result["properties"][&prop]
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|x| x.as_str().to_string())
                    .collect::<Vec<String>>()
                    .contains(&"title".to_string())
                {
                    prop_title =
                        Some(result["properties"][&prop]["title"][0]["plain_text"].clone());
                    break;
                }
            }
            prop_title
        }
    } else if object_type == "database" {
        Some(result["title"][0]["plain_text"].clone())
    } else {
        None
    };

    Ok(title
        .unwrap_or(Value::String("Untitled".to_string()))
        .as_str()
        .unwrap_or("Untitled")
        .to_string())
}

pub async fn search_api(query: Option<String>) -> Result<HashMap<String, Object>, reqwest::Error> {
    let search_query = query.unwrap_or("".to_string());
    let mut objects = HashMap::<String, Object>::new();

    let client = Client::new();
    let response = client
        .post("https://api.notion.com/v1/search")
        .header("Authorization", format!("Bearer {}", get_access_token()))
        .header("Notion-Version", "2022-06-28")
        .json(
            &serde_json::json!({  "query": search_query, "page_size": 20, "sort":{
              "direction":"ascending",
              "timestamp":"last_edited_time"
            }}),
        )
        .send()
        .await?;

    
    let mut json_data = serde_json::json!({});
    if response.status().is_success() {
        json_data = response.json::<serde_json::Value>().await?;
    }

    if let Some(results) = json_data["results"].as_array() {
        for result in results {
            let id = result["id"].as_str().unwrap().to_string();
            let parent_id = match get_parent_id(&client, result["parent"].clone()).await {
                Ok(parent_id) => parent_id,
                Err(_) => None,
            };

            let object_type = result["object"].as_str().unwrap().to_string();
            let title = match get_title(object_type.clone(), result) {
                Ok(title) => title,
                Err(_) => "Untitled".to_string(),
            };

            let object = Object {
                id: id.clone(),
                parent_id,
                title: title.clone(),
                object_type,
                children: Vec::new(),
            };

            objects.insert(id, object);
        }
    }

    Ok(objects)
}
