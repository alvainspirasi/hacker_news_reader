# Show HN: A native Hacker News reader with integrated todo/done tracking

Hey HN! I'm excited to share a tool I've been working on - a native Hacker News reader built with Rust and egui.

## Why I built this

As a daily HN reader, I've always struggled with keeping track of interesting posts I want to read later. Browser tabs pile up, bookmarks get forgotten, and I lose track of what I've already read. I needed a way to:

1. Browse HN efficiently (across all sections - hot, new, show, ask, jobs, best)
2. Quickly mark posts as "todo" for later reading
3. Mark posts as "done" when finished
4. Filter and search effectively

I couldn't find a tool that combined all these features, so I built one. It's been tremendously helpful for my own HN reading workflow, and I thought others might find it useful too.

## Features

- **Integrated todo tracking**: Mark stories as "todo" and "done" to manage your reading progress
- **Search functionality**: Filter stories by keyword in title, domain, or author
- **Multiple sections**: Browse all HN sections (hot, new, show, ask, jobs, best)
- **Threaded comments**: View comments in a Reddit-like threaded format
- **Comment sorting**: Toggle between default order and latest comments first
- **Sharing options**: Easily share links to social media or copy to clipboard
- **History tracking**: Automatically keeps track of articles you've viewed
- **Dark/light mode**: Easy on the eyes in any environment
- **Keyboard shortcuts**: Efficient navigation with keyboard-centric design (1-6 for tabs, Ctrl+F for search, Ctrl+L to copy link)
- **Auto-loading**: Automatically loads more content when scrolling
- **Color-coding**: Stories color-coded by score for easy scanning
- **Native app**: Fast, responsive, and works offline with local caching

## Tech Stack

Built with Rust and the egui UI framework, with SQLite for local storage. The app scrapes Hacker News HTML directly rather than using the official API to capture the full story context.

## Screenshot

![Hacker News Reader Screenshot](logo/logo.png)

## Try it out

Check out the [GitHub repo](https://github.com/haojiang99/hacker_news_reader) for installation instructions and source code. Built and tested on macOS, Linux, and Windows.

I'd love your feedback, feature suggestions, or contributions!

---

This started as a personal tool to solve my own HN reading habits, but I hope others find it useful too. The code is MIT licensed and contributions are welcome.