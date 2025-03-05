use futures::StreamExt;
use std::{error, fmt, io, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use reqwest::Client;

// Static client to be reused across downloads
static CLIENT: once_cell::sync::Lazy<Client> = once_cell::sync::Lazy::new(|| {
    Client::builder()
        .user_agent("reddit_image_downloader_rusty/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
});

pub struct ImageFile {
    pub url: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum DownloadError {
    ReqwestError(reqwest::Error),
    IoError(io::Error),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadError::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            DownloadError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl error::Error for DownloadError {}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::ReqwestError(e)
    }
}

impl From<io::Error> for DownloadError {
    fn from(e: io::Error) -> Self {
        DownloadError::IoError(e)
    }
}

impl ImageFile {
    pub async fn download(&self) -> Result<(), DownloadError> {
        // Note: We assume parent directory is already created in main.rs
        // But we check again here just to be safe
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        } else {
            return Err(DownloadError::IoError(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot create directory: Invalid path",
            )));
        }

        // Use the static client instead of creating a new one for each download
        let response = CLIENT.get(&self.url).send().await?;
        
        // Check for successful response
        if !response.status().is_success() {
            return Err(DownloadError::IoError(
                io::Error::new(
                    io::ErrorKind::Other, 
                    format!("HTTP error: {}", response.status())
                )
            ));
        }

        let mut file = fs::File::create(&self.path).await?;
        
        // Use the stream API for efficient downloading
        let mut stream = response.bytes_stream();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
        }

        Ok(())
    }
}
