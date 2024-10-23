use std::collections::HashMap;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::api::search::Object;

// fuzzy search query in given titles
pub fn match_search(
    query: &str,
    options: &Result<HashMap<String, Object>, reqwest::Error>,
) -> HashMap<String, Object> {
    let mut titles = Vec::<String>::new();
    let objects;
    match options {
        Ok(options) => {
            for (title, _) in options.clone() {
                titles.push(title);
            }
            objects = options;
        }
        Err(_) => return HashMap::new(),
    }

    if query.is_empty() {
        return objects.clone();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(&str, i64)> = titles
        .iter()
        .filter_map(|title| {
            matcher
                .fuzzy_match(&title.as_str(), query)
                .map(|score| (title.as_str(), score))
        })
        .collect();

    results.sort_by_key(|&(_, score)| -score);
    let mut sorted_options = HashMap::<String, Object>::new();

    for title in results.iter().map(|(title, _)| title.to_string()) {
        sorted_options.insert(title.clone(), objects.get(&title).unwrap().clone());
    }

    sorted_options
}
