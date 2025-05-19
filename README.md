# Hacker News Reader

A desktop application for browsing Hacker News built with Rust and eGUI.

## Features

- View top stories from Hacker News
- Open original articles in your default browser
- View comments in a threaded, Reddit-like format
- Clean, native UI with eGUI

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0+)
- Cargo (comes with Rust)

## Getting Started

1. Clone this repository:
   ```
   git clone <repository-url>
   cd hacker_news_reader
   ```

2. Build and run the application:
   ```
   cargo run
   ```

3. For a release build:
   ```
   cargo build --release
   ```
   The executable will be in `target/release/hacker_news_reader`

## Development

- Run tests:
  ```
  cargo test
  ```

- Check for common errors:
  ```
  cargo check
  ```

## Architecture

The application has three main components:

1. **UI Layer** (`main.rs`): Handles the eGUI interface and user interactions
2. **Data Models** (`models.rs`): Defines the data structures for stories and comments
3. **HN Client** (`hn_client.rs`): Responsible for fetching and parsing Hacker News pages

## License

MIT