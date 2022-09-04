use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};
use tokio::task::JoinHandle;

mod image_file;
use image_file::ImageFile;

mod subreddit;
use subreddit::Subreddit;

#[derive(Debug, Deserialize)]
struct Config {
    pub subreddits: Vec<Subreddit>,
    pub file_ext: HashMap<String, bool>,
    pub download_path: String,
}

/// Application to download images from Reddit
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[clap(short, long, value_parser, default_value = "config.toml")]
    config: String,
}

async fn download_subreddit_images(config: Config) {
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "{spinner:.green} {bar:50.green/green} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("#>-");

    let mut handles: Vec<JoinHandle<()>> = vec![];
    for subreddit in config.subreddits {
        let m_clone = m.clone();
        let sty_clone = sty.clone();
        let file_ext = config.file_ext.clone();
        let download_path = config.download_path.clone();

        let handle = tokio::spawn(async move {
            let image_urls = subreddit.get_image_files_urls(&file_ext).await;
            match image_urls {
                Ok(urls) => {
                    let images: Vec<ImageFile> = urls
                        .iter()
                        .map(|u| {
                            let filename = Path::new(&u).file_name().unwrap();
                            ImageFile {
                                url: u.to_string(),
                                path: Path::new(&download_path)
                                    .join(&subreddit.name)
                                    .join(filename),
                            }
                        })
                        .filter(|i| !i.path.exists())
                        .collect();

                    if !images.is_empty() {
                        let pb = m_clone.add(ProgressBar::new(images.len() as u64));
                        pb.set_style(sty_clone);
                        pb.set_message(subreddit.name);

                        for image in images {
                            _ = image.download().await;
                            pb.inc(1);
                        }
                        pb.finish_with_message("Done");
                    }
                }
                Err(_) => println!("Unable to get file URLs for {}", subreddit.name),
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Reading config file at {}", args.config);

    let config_text = fs::read_to_string(args.config).expect("Unable to find config file");
    let config: Config = toml::from_str(&config_text).expect("Issue parsing config file");

    download_subreddit_images(config).await;

    println!("Finish");
}
