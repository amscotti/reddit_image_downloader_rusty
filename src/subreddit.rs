use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::fmt;

// We'll use the same client for all subreddit requests
use once_cell::sync::Lazy;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent("reddit_image_downloader_rusty/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
});

#[derive(Debug)]
pub enum SubredditError {
    RequestFailed(reqwest::Error),
    ParseFailed(reqwest::Error),
    BadResponse(String),
}

impl fmt::Display for SubredditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubredditError::RequestFailed(e) => write!(f, "Request failed: {}", e),
            SubredditError::ParseFailed(e) => write!(f, "Failed to parse response: {}", e),
            SubredditError::BadResponse(msg) => write!(f, "Bad response: {}", msg),
        }
    }
}

impl std::error::Error for SubredditError {}

impl From<reqwest::Error> for SubredditError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_decode() {
            SubredditError::ParseFailed(e)
        } else {
            SubredditError::RequestFailed(e)
        }
    }
}

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

#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
pub struct Subreddit {
    pub name: String,
}

impl Subreddit {
    pub async fn get_image_files_urls(
        &self,
        file_ext: &HashMap<String, bool>,
    ) -> Result<Vec<String>, SubredditError> {
        let response = CLIENT
            .get(format!("https://www.reddit.com/r/{}.json", self.name))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SubredditError::BadResponse(format!(
                "HTTP error: {}", response.status()
            )));
        }

        let deserialized = response.json::<Root>().await?;

        let urls: Vec<String> = deserialized
            .data
            .children
            .into_iter()
            .filter(|c| {
                if let Some(ext) = Path::new(&c.data.url).extension() {
                    if let Some(ext_str) = ext.to_str() {
                        return file_ext.contains_key(ext_str);
                    }
                }
                false
            })
            .map(|f| f.data.url)
            .collect();

        Ok(urls)
    }
}
