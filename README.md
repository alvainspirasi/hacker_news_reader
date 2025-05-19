# Hacker News Reader

A native desktop application for browsing Hacker News with a clean, modern interface. Built with Rust and egui.

![Hacker News Reader](logo/logo.png)

## Features

- Browse top stories from different Hacker News sections:
  - Hot Stories (front page)
  - New Stories
  - Show HN
  - Ask HN
  - Jobs
  - Best Stories
- View comments in a threaded, Reddit-like format
- Search and filter stories by title, domain, or author
- Automatically loads more content when scrolling to the bottom
- Color-coded stories based on score
- Dark and light mode support
- Offline capability with local caching
- Favorite stories for later reading
- Open articles in your default browser

## Installation

### Prerequisites

- Rust (1.70.0 or newer)
- Cargo package manager

### Building from Source

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd hacker_news_reader
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run --release
   ```

The compiled binary will be available at `target/release/hacker_news_reader`.

## Usage

### Navigation

- **Tabs**: Click the tabs at the top to switch between different Hacker News sections (Hot, New, Show, Ask, Jobs, Best).
- **Stories**: Click on a story title to open it in your default web browser.
- **Comments**: Click on the comments count to view the comments for a story.
- **Back**: Use the back button or press Backspace to return to the story list from comments view.
- **Refresh**: Click the refresh button to reload the current section. Hold Shift while clicking to bypass the cache.
- **Theme**: Toggle between dark and light themes using the theme button.
- **Favorites**: Click the hamburger menu (☰) to show or hide your favorite stories.

### Keyboard Shortcuts

- **Arrow Keys**: Use arrow keys to scroll.
- **Space / Page Down**: Scroll down a page.
- **Page Up**: Scroll up a page.
- **Home**: Scroll to the top.
- **End**: Scroll to the bottom.
- **Backspace**: Return to the story list from the comments view.
- **C**: When viewing comments, collapse all top-level comments.
- **Shift+C**: When viewing comments, expand all comments.
- **Ctrl+F**: Show search interface to filter stories.
- **Escape**: Close search interface.
- **1-6 Number Keys**: Switch between tabs (1=Hot, 2=New, 3=Show, 4=Ask, 5=Jobs, 6=Best).

### Story List

The story list displays up to 150 stories per section (5 pages of 30 stories each). Each story shows:

- Story number
- Title (color-coded by score)
- Source domain
- Author
- Score
- Time posted
- Comments count

### Comments View

The comments view shows a threaded display of comments. Features include:

- Collapsible comment threads
- Author highlighting
- Nested replies
- Comment age display
- HTML formatting preserved from original comments

### Favorites

To save a story to your favorites:

1. Click the star icon next to a story.
2. Access your favorites by clicking the hamburger menu (☰) in the upper left.

Favorites are stored locally in a SQLite database.

## Development

- Run tests:
  ```bash
  cargo test
  ```

- Check for errors without building:
  ```bash
  cargo check
  ```

- Format code:
  ```bash
  cargo fmt
  ```

- Run lints:
  ```bash
  cargo clippy
  ```

## Architecture

The application follows a simple architecture with three main components:

1. **UI Layer** (`main.rs`): Contains the main application structure (`HackerNewsReaderApp`) and handles rendering with egui.

2. **Data Models** (`models.rs`): Defines the core data structures:
   - `HackerNewsItem`: Represents a story/post
   - `HackerNewsComment`: Represents a comment with nested children

3. **Hacker News Client** (`hn_client.rs`): Handles HTTP requests to fetch Hacker News content and parses HTML responses using the scraper library.

## License

MIT License - see the LICENSE file for details.

## Acknowledgments

- [Hacker News](https://news.ycombinator.com/) for the content
- [egui](https://github.com/emilk/egui) for the UI framework
- [reqwest](https://docs.rs/reqwest/latest/reqwest/) for HTTP requests
- [scraper](https://docs.rs/scraper/latest/scraper/) for HTML parsing