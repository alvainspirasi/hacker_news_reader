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
    // Store the parameters for the next page of the "new" tab
    pub(crate) next_page_params: std::sync::Mutex<Option<(String, String)>>,
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
            next_page_params: std::sync::Mutex::new(None),
        }
    }
    
    // Allow configuring the cache TTL
    #[allow(dead_code)]
    pub fn set_cache_ttl(&mut self, seconds: u64) {
        self.cache_ttl_secs = seconds;
    }
    
    // Method to check if cache is valid
    #[allow(dead_code)]
    fn has_valid_stories_cache(&self) -> bool {
        if let Ok(cache) = self.cache.lock() {
            cache.is_stories_cache_valid(self.cache_ttl_secs)
        } else {
            false
        }
    }
    
    // Method to get cache age in seconds
    #[allow(dead_code)]
    fn get_cache_age(&self) -> Option<u64> {
        if let Ok(cache) = self.cache.lock() {
            if cache.stories.is_empty() {
                return None;
            }
            
            Some(cache.timestamp.elapsed().as_secs())
        } else {
            None
        }
    }
    
    // This method is now replaced by fetch_stories_by_tab, but kept for backward compatibility
    #[allow(dead_code)]
    fn fetch_top_stories(&self) -> Result<Vec<HackerNewsItem>> {
        self.fetch_stories_by_tab("hot")
    }
    
    pub fn fetch_stories_by_tab(&self, tab: &str) -> Result<Vec<HackerNewsItem>> {
        // Default to page 1
        self.fetch_stories_by_tab_and_page(tab, 1)
    }
    
    // Method to fetch stories by page number (defaults to "hot" tab)
    #[allow(dead_code)]
    pub fn fetch_stories_by_page(&self, page: usize) -> Result<Vec<HackerNewsItem>> {
        self.fetch_stories_by_tab_and_page("hot", page)
    }
    
    pub fn fetch_stories_by_tab_and_page(&self, tab: &str, page: usize) -> Result<Vec<HackerNewsItem>> {
        // Only check cache for "hot" tab page 1 to keep it simple
        if tab == "hot" && page == 1 {
            if let Ok(cache) = self.cache.try_lock() {
                if cache.is_stories_cache_valid(self.cache_ttl_secs) {
                    return Ok(cache.stories.clone());
                }
            }
        }
        
        // If cache check fails or cache is not valid, fetch fresh data
        let stories = self.fetch_fresh_stories_by_tab_and_page(tab, page)?;
        
        // Only cache "hot" tab page 1 to keep it simple
        if tab == "hot" && page == 1 {
            // Now try to update the cache, but don't block if we can't get the lock
            if let Ok(mut cache) = self.cache.try_lock() {
                cache.update_stories(stories.clone());
                println!("Updated stories cache with {} items", stories.len());
            }
        }
        
        Ok(stories)
    }
    
    // Method to directly fetch stories without checking cache (used as fallback)
    #[allow(dead_code)]
    fn fetch_fresh_stories(&self) -> Result<Vec<HackerNewsItem>> {
        self.fetch_fresh_stories_by_tab("hot")
    }
    
    // Method to directly fetch stories by page without checking cache
    #[allow(dead_code)]
    pub fn fetch_fresh_stories_by_page(&self, page: usize) -> Result<Vec<HackerNewsItem>> {
        self.fetch_fresh_stories_by_tab_and_page("hot", page)
    }
    
    // Method to fetch stories from a specific tab
    pub fn fetch_fresh_stories_by_tab(&self, tab: &str) -> Result<Vec<HackerNewsItem>> {
        // Default to page 1
        self.fetch_fresh_stories_by_tab_and_page(tab, 1)
    }
    
    // Helper method to extract "More" link parameters from HTML
    fn extract_more_link_params(&self, html: &str) -> Option<(String, String)> {
        // Look for the "More" link which contains the "next" and "n" parameters
        if let Some(more_link_pos) = html.find("class=\"morelink\"") {
            // Look for the href attribute before the class
            let href_start = html[..more_link_pos].rfind("href=\"")?;
            let href_end = html[href_start + 6..].find("\"")?;
            
            // Extract the href value
            let href = &html[href_start + 6..href_start + 6 + href_end];
            
            // Parse the href to extract the next and n parameters
            if href.contains("newest?next=") && href.contains("&n=") {
                // Extract the next parameter
                let next_start = href.find("next=")?;
                let next_end = href[next_start + 5..].find("&")?;
                let next_param = &href[next_start + 5..next_start + 5 + next_end];
                
                // Extract the n parameter
                let n_start = href.find("&n=")?;
                let n_param = &href[n_start + 3..];
                
                return Some((next_param.to_string(), n_param.to_string()));
            }
        }
        
        None
    }
    

    pub fn fetch_fresh_stories_by_tab_and_page(&self, tab: &str, page: usize) -> Result<Vec<HackerNewsItem>> {
        let base_url = match tab {
            "hot" => "https://news.ycombinator.com/",
            "new" => "https://news.ycombinator.com/newest",
            "show" => "https://news.ycombinator.com/show",
            "ask" => "https://news.ycombinator.com/ask",
            "jobs" => "https://news.ycombinator.com/jobs",
            "best" => "https://news.ycombinator.com/best",
            _ => "https://news.ycombinator.com/", // Default to hot
        };
        
        // Add page parameter if page > 1
        let url = if page > 1 {
            // Special handling for "new" tab which uses a different pagination mechanism
            if tab == "new" {
                // For "new" tab pagination, we need to use the format:
                // newest?next=ITEM_ID&n=N
                // where n increments by 30 for each page (n=31, n=61, n=91, etc.)
                
                // For page 2, we use a simple n=31 approach if we don't have stored params
                if page == 2 {
                    format!("{}?n=31", base_url)
                } else {
                    // For pages > 2, we should have extracted parameters from the previous page
                    if let Ok(params_guard) = self.next_page_params.lock() {
                        if let Some((next_param, n_param)) = params_guard.as_ref() {
                            format!("{}?next={}&n={}", base_url, next_param, n_param)
                        } else {
                            // Fallback if we don't have stored params
                            let n_param = 1 + (page - 1) * 30;
                            format!("{}?n={}", base_url, n_param)
                        }
                    } else {
                        // Fallback if we can't get the lock
                        let n_param = 1 + (page - 1) * 30;
                        format!("{}?n={}", base_url, n_param)
                    }
                }
            } else {
                // For other tabs, use the standard p parameter
                format!("{}?p={}", base_url, page)
            }
        } else {
            base_url.to_string()
        };
        
        let response = self.client.get(&url).send()?;

        let html = response.text()?;
        
        // Save the HTML to a file for debugging
        let _ = std::fs::write("hn_debug.html", &html);
        
        // If this is the "new" tab, try to extract the "More" link parameters for next page
        if tab == "new" {
            if let Some((next_param, n_param)) = self.extract_more_link_params(&html) {
                // Store the parameters for the next page
                if let Ok(mut params_guard) = self.next_page_params.lock() {
                    *params_guard = Some((next_param, n_param));
                }
            }
        }
        
        let stories = Self::parse_stories(&html)?;
        // Debug output turned off
        // println!("SUCCESSFULLY LOADED {} STORIES FROM {} TAB, PAGE {}", stories.len(), tab, page);
        
        // Debug output turned off - story titles
        // for (i, story) in stories.iter().enumerate() {
        //     println!("  Story {}: {} (by {})", i+1, story.title, story.by);
        // }
        
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
    
    // Method to directly fetch latest comments first using the undocumented /latest endpoint
    pub fn fetch_latest_comments(&self, item_id: &str) -> Result<Vec<HackerNewsComment>> {
        let url = format!("https://news.ycombinator.com/item?id={}", item_id);
        let latest_url = format!("https://news.ycombinator.com/latest?id={}", item_id);
        
        // First try the /latest endpoint
        let response = self.client.get(&latest_url).send()?;
        let html = response.text()?;
        let comments = Self::parse_comments(&html)?;
        
        // If we got comments successfully, return them
        if !comments.is_empty() {
            println!("Successfully loaded {} latest comments", comments.len());
            return Ok(comments);
        }
        
        // If no comments were found with the latest endpoint, fall back to the regular endpoint
        println!("No comments found with /latest endpoint, falling back to standard endpoint");
        let fallback_response = self.client.get(&url).send()?;
        let fallback_html = fallback_response.text()?;
        let fallback_comments = Self::parse_comments(&fallback_html)?;
        
        println!("Fallback loaded {} comments", fallback_comments.len());
        Ok(fallback_comments)
    }
    
    fn parse_stories(html: &str) -> Result<Vec<HackerNewsItem>> {
        // Extract the page number from the URL if present (to calculate correct indices)
        let _page_number = if html.contains("?p=") {
            // Try to extract page number from HTML by looking for links containing ?p=
            if let Some(start_idx) = html.find("?p=") {
                let page_str = &html[start_idx + 3..start_idx + 5]; // Get up to 2 digits after ?p=
                page_str.chars().take_while(|c| c.is_digit(10)).collect::<String>().parse::<usize>().unwrap_or(1)
            } else {
                1 // Default to page 1 if not found
            }
        } else {
            1 // Default to page 1 if no page parameter
        };
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
            
            // Calculate the original index based on page and position
            // For page 1, indices are 0, 1, 2, ..., 29
            // For page 2, indices are 30, 31, 32, ..., 59
            // IMPORTANT: Adjust to 0-based for proper display indexing
            let original_index = i;
            
            stories.push(HackerNewsItem {
                id,
                title,
                url,
                domain,
                by,
                score,
                time_ago,
                comments_count,
                original_index,
            });
        }
        
        Ok(stories)
    }
    
    fn parse_comments(html: &str) -> Result<Vec<HackerNewsComment>> {
        let document = Html::parse_document(html);
        let comment_selector = Selector::parse(".comtr").map_err(|e| anyhow!("Selector error: {:?}", e))?;
        
        // Extract all comments as flat list with their levels
        let mut comment_list = Vec::new();
        
        // Set to keep track of comment IDs we've already processed to avoid duplicates
        let mut processed_ids = std::collections::HashSet::new();
        
        for comment_row in document.select(&comment_selector) {
            let id = comment_row.value().attr("id").unwrap_or_default().to_string();
            
            // Skip if we've already seen this comment ID
            if !processed_ids.insert(id.clone()) {
                continue;
            }
            
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

        // Create simple recursive structure with improved child finding that prevents duplicates
        fn build_comments_tree(comments: &[(i32, HackerNewsComment)]) -> Vec<HackerNewsComment> {
            if comments.is_empty() {
                return Vec::new();
            }
            
            // Simple approach: build the tree by finding children for each parent
            let mut result = Vec::new();
            let mut used_indices = std::collections::HashSet::new();
            
            // Start with top-level comments (level 0)
            for (i, (level, comment)) in comments.iter().enumerate() {
                if *level == 0 && !used_indices.contains(&i) {
                    let mut top_comment = comment.clone();
                    used_indices.insert(i);
                    
                    // Find all children for this top-level comment
                    top_comment.children = find_children_recursive(comments, i, *level, &mut used_indices);
                    
                    result.push(top_comment);
                }
            }
            
            result
        }
        
        // Helper function to recursively find all children of a comment
        fn find_children_recursive(
            comments: &[(i32, HackerNewsComment)], 
            parent_idx: usize, 
            parent_level: i32,
            used_indices: &mut std::collections::HashSet<usize>
        ) -> Vec<HackerNewsComment> {
            let mut children = Vec::new();
            let expected_child_level = parent_level + 1;
            
            // Look for direct children after the parent
            for i in (parent_idx + 1)..comments.len() {
                if used_indices.contains(&i) {
                    continue;
                }
                
                let (level, comment) = &comments[i];
                
                // If we hit a comment at or above parent level, stop looking for children
                if *level <= parent_level {
                    break;
                }
                
                // If this is a direct child (exactly one level deeper)
                if *level == expected_child_level {
                    used_indices.insert(i);
                    let mut child = comment.clone();
                    
                    // Recursively find children for this child
                    child.children = find_children_recursive(comments, i, *level, used_indices);
                    
                    children.push(child);
                }
            }
            
            children
        }
        
        // Use the recursive approach to build the tree properly
        let tree = build_comments_tree(&comment_list);
        Ok(tree)
    }
}