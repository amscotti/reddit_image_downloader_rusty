# Reddit Image Downloader

This is a command line tool that uses a config file to download images from a list of subreddits.

This is a Rust port of a [Go app](https://github.com/amscotti/reddit_image_downloader) used to help better understand Rust, and async using [Tokio](https://tokio.rs/)

[![asciicast](https://asciinema.org/a/tbsjBsJSwGlRDDiCOb3mi0SD0.svg)](https://asciinema.org/a/tbsjBsJSwGlRDDiCOb3mi0SD0)

## Features

- Parallel downloading of images from multiple subreddits
- Concurrent downloads within each subreddit with configurable limits
- Efficient streaming downloads
- Progress bars for each subreddit
- Automatic skipping of already downloaded images
- Configurable file extensions to download

## Building

To build, run
```bash
$ cargo build --release
```

## Usage

```
reddit_image_downloader 0.1.0
Application to download images from Reddit

USAGE:
    reddit_image_downloader [OPTIONS]

OPTIONS:
    -c, --config <CONFIG>         Path to config file [default: config.toml]
    -h, --help                    Print help information
    -V, --version                 Print version information
    --concurrent <CONCURRENT>     Number of concurrent downloads per subreddit
```

## Config 
* subreddits: List of Subreddits to look for images in
* file_ext: What file type to download
* download_path: Path to download files into
* concurrent_downloads: Maximum number of concurrent downloads per subreddit (default: 5)

### Example

```TOML
subreddits = [
    "EarthPorn",
    "Wallpapers",
    "ServerPorn",
    "unixporn",
    "CoffeePorn",
    "battlestations",
    "Aww",
    "Beerwithaview",
    "OldSchoolCool",
    "TheWayWeWere",
    "SkyPorn",
    "spaceporn",
    "itookapicture",
]

download_path = "download"

# Maximum number of concurrent downloads per subreddit
concurrent_downloads = 5

[file_ext]
jpg = true
png = true
gif = true
```

## Performance

The application downloads images from multiple subreddits in parallel, and within each subreddit, it downloads multiple images concurrently. This allows for high throughput and efficient use of your network connection.

You can adjust the concurrency level with the `--concurrent` command line option or in the config file to find the optimal setting for your system.