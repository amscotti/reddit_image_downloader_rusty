use std::{borrow::BorrowMut, fs, path::PathBuf};

use tokio::io::AsyncWriteExt;

pub struct ImageFile {
    pub url: String,
    pub path: PathBuf,
}

impl ImageFile {
    pub async fn download(&self) -> Result<(), ()> {
        fs::create_dir_all(self.path.parent().unwrap()).unwrap();

        let client = reqwest::Client::new();
        let mut response = client.get(&self.url).send().await.unwrap();
        let mut file = tokio::fs::File::create(&self.path).await.unwrap();

        while let Some(mut item) = response.chunk().await.unwrap() {
            file.write_all_buf(item.borrow_mut()).await.unwrap();
        }

        Ok(())
    }
}
