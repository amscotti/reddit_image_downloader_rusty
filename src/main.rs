use clap::Parser;
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path, sync::Arc};

mod image_file;
use image_file::ImageFile;

mod subreddit;
use subreddit::Subreddit;

#[derive(Debug, Deserialize)]
struct Config {
    pub subreddits: Vec<Subreddit>,
    pub file_ext: HashMap<String, bool>,
    pub download_path: String,
    #[serde(default = "default_concurrent_downloads")]
    pub concurrent_downloads: usize,
}

fn default_concurrent_downloads() -> usize {
    5 // Default to 5 concurrent downloads per subreddit
}

/// Application to download images from Reddit
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[clap(short, long, value_parser, default_value = "config.toml")]
    config: String,
    
    /// Number of concurrent downloads per subreddit
    #[clap(short, long, value_parser)]
    concurrent: Option<usize>,
}

async fn download_subreddit_images(config: Config) {
    // Create progress bar infrastructure
    let m = MultiProgress::new();
    
    let sty = ProgressStyle::default_bar()
        .template("{prefix:<15.bold.bright.green} {spinner:.bright.green} [{bar:50.green/black}] {pos:>3}/{len:<3} ({percent:>3}%) {msg:.bright.green}")
        .unwrap()
        .progress_chars("█▓▒░▁");
    
    // First collect all images that need downloading
    let mut all_tasks = Vec::new();
    
    for subreddit in config.subreddits {
        let file_ext = config.file_ext.clone();
        let download_path = config.download_path.clone();
        let concurrent_downloads = config.concurrent_downloads;
        let subreddit_name = subreddit.name.clone();
        let m = m.clone();
        let sty = sty.clone();
        
        let task = tokio::spawn(async move {
            // Get image URLs
            let url_result = subreddit.get_image_files_urls(&file_ext).await;
            
            match url_result {
                Ok(urls) => {
                    // Create subreddit directory
                    let subreddit_path = Path::new(&download_path).join(&subreddit_name);
                    if let Err(e) = tokio::fs::create_dir_all(&subreddit_path).await {
                        eprintln!("\nFailed to create directory for {}: {}", subreddit_name, e);
                        return;
                    }
                    
                    // Filter image URLs for new files only
                    let images: Vec<ImageFile> = urls
                        .iter()
                        .filter_map(|u| {
                            let filename = match Path::new(&u).file_name() {
                                Some(name) => name,
                                None => {
                                    return None;
                                }
                            };
                            
                            let path = subreddit_path.join(filename);
                            if path.exists() {
                                return None; // Skip existing files
                            }
                            
                            Some(ImageFile {
                                url: u.to_string(),
                                path,
                            })
                        })
                        .collect();
                    
                    if images.is_empty() {
                        return;
                    }
                    
                    // Create progress bar with formatting specifically for this task
                    let pb = m.add(ProgressBar::new(images.len() as u64));
                    pb.set_style(sty);
                    pb.set_prefix(format!("[{}]", subreddit_name));
                    pb.set_message("fetching data...");
                    
                    // Enable tick for smoother updates
                    pb.enable_steady_tick(std::time::Duration::from_millis(120));
                    
                    // Use Arc to safely share the progress bar
                    let pb = Arc::new(pb);
                    
                    // Download images concurrently
                    stream::iter(images)
                        .for_each_concurrent(concurrent_downloads, |image| {
                            let pb = Arc::clone(&pb);
                            let url = image.url.clone();
                            
                            async move {
                                match image.download().await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        // Write error on a new line to avoid progress bar corruption
                                        pb.suspend(|| {
                                            eprintln!("\nFailed to download {}: {}", url, e);
                                        });
                                    }
                                }
                                pb.inc(1);
                            }
                        })
                        .await;
                    
                    pb.finish_with_message("download complete");
                },
                Err(e) => {
                    // Print errors without interfering with progress display
                    eprintln!("\nError for subreddit {}: {:?}", subreddit_name, e);
                }
            }
        });
        
        all_tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in all_tasks {
        if let Err(e) = task.await {
            eprintln!("\nTask panicked: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("\x1b[32mReading config file at {}\x1b[0m", args.config);

    let config_text = match fs::read_to_string(&args.config) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            return;
        }
    };
    
    let mut config: Config = match toml::from_str(&config_text) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            return;
        }
    };
    
    // Override config with command line arg if provided
    if let Some(concurrent) = args.concurrent {
        config.concurrent_downloads = concurrent;
    }

    println!("\x1b[32mUsing {} concurrent downloads per subreddit\x1b[0m", config.concurrent_downloads);
    println!("\x1b[1;32mDownloading from {} subreddits...\x1b[0m", config.subreddits.len());
    
    download_subreddit_images(config).await;

    println!("\x1b[1;32mFinished downloading all images\x1b[0m");
}
