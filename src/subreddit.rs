use reqwest::{self, header::USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub data: Data,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub children: Vec<Children>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Children {
    pub data: Data2,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data2 {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Subreddit {
    pub name: String,
}

impl Subreddit {
    pub async fn get_image_files_urls(
        &self,
        file_ext: &HashMap<String, bool>,
    ) -> Result<Vec<String>, ()> {
        let client = reqwest::Client::new();
        let deserialized = client
            .get(format!("https://www.reddit.com/r/{}.json", self.name))
            .header(USER_AGENT, "Rust/0.1")
            .send()
            .await
            .unwrap()
            .json::<Root>()
            .await
            .unwrap();

        let urls: Vec<String> = deserialized
            .data
            .children
            .into_iter()
            .filter(|c| match Path::new(&c.data.url).extension() {
                Some(ext) => file_ext.contains_key(ext.to_str().unwrap()),
                None => false,
            })
            .map(|f| f.data.url)
            .collect();

        Ok(urls)
    }
}
