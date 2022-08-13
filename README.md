# Reddit Image Downloader

This is a command line tool that uses a config file to download images from a list of subreddits.

This is a Rust port of a [Go app](https://github.com/amscotti/reddit_image_downloader) used to help better understand Rust, and async using [Tokio](https://tokio.rs/)

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
    -c, --config <CONFIG>    Path to config file [default: config.toml]
    -h, --help               Print help information
    -V, --version            Print version information
```

## Config 
* subreddits: List of Subreddits to look for images in
* file_ext: What file type to download
* download_path: Path to download files into

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

[file_ext]
jpg = true
png = true
gif = true
```