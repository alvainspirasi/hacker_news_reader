use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use std::sync::{Arc, Mutex};

use crate::models::{HackerNewsItem, HackerNewsComment, StoriesCache};

pub struct HackerNewsClient {
    client: Client,
    pub(crate) cache: Arc<Mutex<StoriesCache>>,
    pub(crate) cache_ttl_secs: u64,
}

impl HackerNewsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        // Create a mutex with default stories cache
        let cache = Arc::new(Mutex::new(StoriesCache::new()));
        
        // Set a timeout for HTTP requests to be safer
        let _timeout = Duration::from_secs(30);
            
        Self { 
            client,
            cache,
            cache_ttl_secs: 300, // 5 minutes TTL by default
        }
    }
    
    // Allow configuring the cache TTL
    #[allow(dead_code)]
    pub fn set_cache_ttl(&mut self, seconds: u64) {
        self.cache_ttl_secs = seconds;
    }
    
    // Method to check if cache is valid
    pub fn has_valid_stories_cache(&self) -> bool {
        if let Ok(cache) = self.cache.lock() {
            cache.is_stories_cache_valid(self.cache_ttl_secs)
        } else {
            false
        }
    }
    
    // Method to get cache age in seconds
    pub fn get_cache_age(&self) -> Option<u64> {
        if let Ok(cache) = self.cache.lock() {
            if cache.stories.is_empty() {
                return None;
            }
            
            Some(cache.timestamp.elapsed().as_secs())
        } else {
            None
        }
    }
    
    pub fn fetch_top_stories(&self) -> Result<Vec<HackerNewsItem>> {
        // First check the cache with a shorter timeout
        if let Ok(cache) = self.cache.try_lock() {
            if cache.is_stories_cache_valid(self.cache_ttl_secs) {
                return Ok(cache.stories.clone());
            }
        }
        
        // If cache check fails or cache is not valid, fetch fresh data
        let stories = self.fetch_fresh_stories()?;
        
        // Now try to update the cache, but don't block if we can't get the lock
        if let Ok(mut cache) = self.cache.try_lock() {
            cache.update_stories(stories.clone());
            println!("Updated stories cache with {} items", stories.len());
        }
        
        Ok(stories)
    }
    
    pub fn fetch_stories_by_tab(&self, tab: &str) -> Result<Vec<HackerNewsItem>> {
        // First check the cache with a shorter timeout
        // Only check cache for "hot" tab to keep it simple
        if tab == "hot" {
            if let Ok(cache) = self.cache.try_lock() {
                if cache.is_stories_cache_valid(self.cache_ttl_secs) {
                    return Ok(cache.stories.clone());
                }
            }
        }
        
        // If cache check fails or cache is not valid, fetch fresh data
        let stories = self.fetch_fresh_stories_by_tab(tab)?;
        
        // Only cache "hot" tab to keep it simple
        if tab == "hot" {
            // Now try to update the cache, but don't block if we can't get the lock
            if let Ok(mut cache) = self.cache.try_lock() {
                cache.update_stories(stories.clone());
                println!("Updated stories cache with {} items", stories.len());
            }
        }
        
        Ok(stories)
    }
    
    // Method to directly fetch stories without checking cache (used as fallback)
    pub fn fetch_fresh_stories(&self) -> Result<Vec<HackerNewsItem>> {
        self.fetch_fresh_stories_by_tab("hot")
    }
    
    // Method to fetch stories from a specific tab
    pub fn fetch_fresh_stories_by_tab(&self, tab: &str) -> Result<Vec<HackerNewsItem>> {
        let url = match tab {
            "hot" => "https://news.ycombinator.com/",
            "new" => "https://news.ycombinator.com/newest",
            "show" => "https://news.ycombinator.com/show",
            _ => "https://news.ycombinator.com/", // Default to hot
        };
        
        let response = self.client.get(url).send()?;

        let html = response.text()?;
        
        // Save the HTML to a file for debugging
        let _ = std::fs::write("hn_debug.html", &html);
        
        let stories = Self::parse_stories(&html)?;
        println!("Successfully loaded {} stories from {} tab", stories.len(), tab);
        
        Ok(stories)
    }
    
    pub fn fetch_comments(&self, item_id: &str) -> Result<Vec<HackerNewsComment>> {
        // First check the cache with a shorter timeout
        if let Ok(cache) = self.cache.try_lock() {
            if cache.is_comments_cache_valid(item_id, self.cache_ttl_secs) {
                return Ok(cache.get_cached_comments(item_id).unwrap().clone());
            }
        }
        
        // If cache check fails or cache is not valid, fetch fresh data
        let comments = self.fetch_fresh_comments(item_id)?;
        
        // Now try to update the cache, but don't block if we can't get the lock
        if let Ok(mut cache) = self.cache.try_lock() {
            cache.update_comments(item_id.to_string(), comments.clone());
            println!("Successfully loaded {} comments", comments.len());
        }
        
        Ok(comments)
    }
    
    // Method to directly fetch comments without checking or updating cache
    pub fn fetch_fresh_comments(&self, item_id: &str) -> Result<Vec<HackerNewsComment>> {
        let url = format!("https://news.ycombinator.com/item?id={}", item_id);
        let response = self.client.get(&url)
            .send()?;
        
        let html = response.text()?;
        let comments = Self::parse_comments(&html)?;
        
        println!("Successfully loaded {} comments", comments.len());
        Ok(comments)
    }
    
    fn parse_stories(html: &str) -> Result<Vec<HackerNewsItem>> {
        let document = Html::parse_document(html);
        let story_selector = match Selector::parse(".athing") {
            Ok(selector) => selector,
            Err(e) => return Err(anyhow!("Selector error: {:?}", e)),
        };
        
        let mut stories = Vec::new();
        let story_elements = document.select(&story_selector).collect::<Vec<_>>();
        
        for (i, story_row) in story_elements.into_iter().enumerate() {
            let id = story_row.value().attr("id").unwrap_or_default().to_string();
            
            // Title and URL
            let title_selector = match Selector::parse(".titleline > a") {
                Ok(selector) => selector,
                Err(e) => return Err(anyhow!("Title selector error: {:?}", e)),
            };
            
            // Check if we found the title element
            let title_element = story_row.select(&title_selector).next();
            
            let title = title_element
                .map(|e| {
                    let html = e.inner_html();
                    html
                })
                .unwrap_or_default();
                
            let url = title_element
                .and_then(|e| {
                    let url = e.value().attr("href");
                    url
                })
                .unwrap_or_default()
                .to_string();
                
            // Domain (if external link)
            let domain_selector = Selector::parse(".sitestr").map_err(|e| anyhow!("Selector error: {:?}", e))?;
            let domain = story_row.select(&domain_selector).next()
                .map(|e| e.inner_html())
                .unwrap_or_default();
            
            // Get subtext row (contains author, score, age, comment count)
            let subtext_selector = Selector::parse(".subtext").map_err(|e| anyhow!("Selector error: {:?}", e))?;
            let subtext = document.select(&subtext_selector).nth(i);
            
            let by = subtext
                .and_then(|e| {
                    let by_selector = Selector::parse(".hnuser").ok()?;
                    e.select(&by_selector).next().map(|e| e.inner_html())
                })
                .unwrap_or_default();
                
            let score = subtext
                .and_then(|e| {
                    let score_selector = Selector::parse(".score").ok()?;
                    let score_text = e.select(&score_selector).next()?.inner_html();
                    score_text.split_whitespace().next()?
                        .parse::<i32>().ok()
                })
                .unwrap_or(0);
                
            // Get time_ago with improved parsing
            let time_ago = subtext
                .and_then(|e| {
                    // First find the age element
                    let age_selector = Selector::parse(".age").ok()?;
                    let age_element = e.select(&age_selector).next()?;
                    
                    // Check if we can extract from the inner a tag
                    if let Ok(a_selector) = Selector::parse("a") {
                        if let Some(link) = age_element.select(&a_selector).next() {                        
                            // Format like '4 hours ago' is inside the <a> tag
                            let time_text = link.inner_html();
                            
                            // Clean up any &nbsp; or other HTML entities
                            return Some(html_escape::decode_html_entities(&time_text).to_string());
                        }
                    }
                    
                    // Fallback to just using the inner HTML
                    Some(age_element.inner_html())
                })
                .unwrap_or_default();
                
            // Improved comment count parsing
            let comments_count = subtext
                .and_then(|e| {
                    let link_selector = Selector::parse("a").ok()?;
                    for link in e.select(&link_selector) {
                        let link_text = link.inner_html();
                        
                        if link_text.contains("comment") || link_text.contains("discuss") {
                            // Parse more carefully - handle formats like "123 comments" or "discuss"
                            if link_text.contains("discuss") {
                                return Some(0);
                            }
                            
                            // Extract number from strings like "29&nbsp;comments"
                            let re = regex::Regex::new(r"(\d+)(?:&nbsp;|\s+)comments").unwrap();
                            if let Some(caps) = re.captures(&link_text) {
                                if let Some(count_match) = caps.get(1) {
                                    if let Ok(count) = count_match.as_str().parse::<i32>() {
                                        return Some(count);
                                    }
                                }
                            }
                            
                            // Also try to find the number at start of string (older format)
                            let parts: Vec<&str> = link_text.split_whitespace().collect();
                            if parts.len() >= 1 {
                                if let Ok(count) = parts[0].parse::<i32>() {
                                    return Some(count);
                                }
                            }
                            
                            // Fallback to 0
                            return Some(0);
                        }
                    }
                    Some(0)
                })
                .unwrap_or(0);
            
            stories.push(HackerNewsItem {
                id,
                title,
                url,
                domain,
                by,
                score,
                time_ago,
                comments_count,
            });
        }
        
        Ok(stories)
    }
    
    fn parse_comments(html: &str) -> Result<Vec<HackerNewsComment>> {
        let document = Html::parse_document(html);
        let comment_selector = Selector::parse(".comtr").map_err(|e| anyhow!("Selector error: {:?}", e))?;
        
        // Extract all comments as flat list with their levels
        let mut comment_list = Vec::new();
        
        for comment_row in document.select(&comment_selector) {
            let id = comment_row.value().attr("id").unwrap_or_default().to_string();
            
            let level = comment_row
                .select(&Selector::parse(".ind").map_err(|e| anyhow!("Selector error: {:?}", e))?)
                .next()
                .and_then(|e| e.value().attr("indent"))
                .and_then(|indent| indent.parse::<i32>().ok())
                .unwrap_or(0);
                
            let by = comment_row
                .select(&Selector::parse(".hnuser").map_err(|e| anyhow!("Selector error: {:?}", e))?)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();
                
            let time_ago = comment_row
                .select(&Selector::parse(".age").map_err(|e| anyhow!("Selector error: {:?}", e))?)
                .next()
                .map(|e| {
                    if let Ok(a_selector) = Selector::parse("a") {
                        if let Some(link) = e.select(&a_selector).next() {                        
                            let time_text = link.inner_html();
                            return html_escape::decode_html_entities(&time_text).to_string();
                        }
                    }
                    e.inner_html()
                })
                .unwrap_or_default();
                
            let text = comment_row
                .select(&Selector::parse(".commtext").map_err(|e| anyhow!("Selector error: {:?}", e))?)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();
                
            comment_list.push((level, HackerNewsComment {
                id,
                by,
                text,
                time_ago,
                level,
                children: Vec::new(),
            }));
        }

        // Most direct approach - use the recursive function below to build the comment tree
        
        // Most direct approach - create simple recursive structure
        fn build_comments_tree(comments: &[(i32, HackerNewsComment)]) -> Vec<HackerNewsComment> {
            // Start with top-level comments
            let mut result = Vec::new();
            
            for (i, (level, comment)) in comments.iter().enumerate() {
                if *level == 0 {
                    // Add top-level comment
                    let mut top_comment = comment.clone();
                    
                    // Find all direct children of this comment
                    let children = find_children(comments, i);
                    top_comment.children = children;
                    
                    result.push(top_comment);
                }
            }
            
            return result;
            
            // Helper function to find all direct children of a comment
            fn find_children(comments: &[(i32, HackerNewsComment)], parent_idx: usize) -> Vec<HackerNewsComment> {
                let (parent_level, _) = comments[parent_idx];
                let child_level = parent_level + 1;
                
                let mut children = Vec::new();
                let mut _last_direct_child_idx = 0;
                
                // Look for comments after the parent
                for i in (parent_idx + 1)..comments.len() {
                    let (level, comment) = &comments[i];
                    
                    if *level < parent_level {
                        // This is a comment above our parent's level, so we're done
                        break;
                    } else if *level == child_level {
                        // This is a direct child
                        let mut child = comment.clone();
                        
                        // Recursively find this child's children
                        child.children = find_children(comments, i);
                        
                        children.push(child);
                        _last_direct_child_idx = i;
                    } else if *level > child_level {
                        // This is a deeper nested comment, skip it
                        // (it will be handled by recursive calls)
                        continue;
                    }
                }
                
                children
            }
        }
        
        // Use the recursive approach to build the tree properly
        let tree = build_comments_tree(&comment_list);
        Ok(tree)
    }
}