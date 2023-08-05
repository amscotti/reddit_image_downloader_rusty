use std::{borrow::BorrowMut, error, fmt, io, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use reqwest::Client;

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
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        } else {
            return Err(DownloadError::IoError(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot create directory: Invalid path",
            )));
        }

        let client = Client::new();
        let mut response = client.get(&self.url).send().await?;

        let mut file = fs::File::create(&self.path).await?;

        while let Some(mut chunk) = response.chunk().await? {
            let mut buffer = chunk.borrow_mut();
            file.write_all_buf(&mut buffer).await?;
        }

        Ok(())
    }
}
