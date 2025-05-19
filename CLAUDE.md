# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Hacker News reader application built with Rust and eGUI. It displays Hacker News stories in a table format, allows users to open articles in their default browser, and view comments in a threaded, Reddit-like format within the app.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Build for release
cargo build --release

# Run tests
cargo test

# Check for errors without building
cargo check

# Format code
cargo fmt

# Run lints
cargo clippy
```

## Project Architecture

The application follows a simple architecture with three main components:

1. **UI Layer** (`main.rs`):
   - Contains the main application structure (`HackerNewsReaderApp`)
   - Handles rendering the UI with eGUI
   - Manages application state and user interactions

2. **Data Models** (`models.rs`):
   - Defines the core data structures:
     - `HackerNewsItem`: Represents a story/post
     - `HackerNewsComment`: Represents a comment with nested children

3. **Hacker News Client** (`hn_client.rs`):
   - Handles HTTP requests to fetch Hacker News content
   - Parses HTML responses using the scraper library
   - Transforms raw HTML into application data models

## Key Dependencies

- `eframe` and `egui`: UI framework (v0.31.1)
- `reqwest`: HTTP client (v0.12.15)
- `scraper`: HTML parsing (v0.23.1)
- `tokio`: Async runtime
- `anyhow`: Error handling
- `open`: For opening URLs in the default browser

## Implementation Notes

- The application scrapes Hacker News HTML directly rather than using the official API
- Comments are parsed and displayed in a nested structure
- External links open in the system's default browser
- The UI is built with eGUI's tables and scrollable areas

## Features

### Navigation Tabs
The app includes a tabbed navigation system to browse different sections of Hacker News:
- **Hot**: Front page stories sorted by popularity (default)
- **New**: Recently submitted stories
- **Show**: Show HN posts where users share projects

### Theme Support
- The application supports both light and dark themes
- Theme preference is persisted between sessions
- Toggle between themes with the sun/moon button in the header

### Comments
- Threaded comment display with collapsible threads
- Comment pagination for better performance with large threads
- Keyboard shortcuts for navigating comments:
  - C: Collapse all top-level comments
  - Shift+C: Expand all comments
  - Arrow keys: Navigate between comment pages
  - Home/End: Jump to first/last comment page
  
### UI Enhancements
- Modern design with proper spacing and typography
- Responsive layout that adapts to window size
- Visual indicators for story scores and comment counts
- Tooltips for buttons and interactive elements

## Code Organization

### Theme System
The `AppTheme` struct manages visual styling with:
- Color palettes for both light and dark modes
- Consistent spacing and corner radius settings
- Shadow settings optimized for each theme variant

### Data Fetching
- Network requests run in background threads to maintain UI responsiveness
- Data is cached with configurable TTL (Time To Live) values
- Force refresh option (hold Shift while clicking refresh) to bypass cache

### Display and Interaction
- Stories and comments are rendered with proper hierarchy
- Comments can be collapsed individually or all at once
- Pagination controls for navigating large comment threads
- Integration with system browser for opening external links