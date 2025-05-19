#[derive(Debug, Clone)]
pub struct HackerNewsItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub domain: String,
    pub by: String,
    pub score: i32,
    pub time_ago: String,
    pub comments_count: i32,
    #[allow(dead_code)]
    pub original_index: usize, // Track original index for stable numbering
}

#[derive(Debug, Clone)]
pub struct HackerNewsComment {
    pub id: String,
    pub by: String,
    pub text: String,
    pub time_ago: String,
    #[allow(dead_code)]
    pub level: i32,
    pub children: Vec<HackerNewsComment>,
}

pub struct StoriesCache {
    pub stories: Vec<HackerNewsItem>,
    pub timestamp: std::time::Instant,
    pub comments_cache: std::collections::HashMap<String, (Vec<HackerNewsComment>, std::time::Instant)>,
}

impl StoriesCache {
    pub fn new() -> Self {
        Self {
            stories: Vec::new(),
            timestamp: std::time::Instant::now(),
            comments_cache: std::collections::HashMap::new(),
        }
    }
    
    pub fn is_stories_cache_valid(&self, ttl_secs: u64) -> bool {
        if self.stories.is_empty() {
            return false;
        }
        
        let elapsed = self.timestamp.elapsed().as_secs();
        elapsed < ttl_secs
    }
    
    pub fn is_comments_cache_valid(&self, story_id: &str, ttl_secs: u64) -> bool {
        if let Some((_, timestamp)) = self.comments_cache.get(story_id) {
            let elapsed = timestamp.elapsed().as_secs();
            elapsed < ttl_secs
        } else {
            false
        }
    }
    
    pub fn update_stories(&mut self, stories: Vec<HackerNewsItem>) {
        self.stories = stories;
        self.timestamp = std::time::Instant::now();
    }
    
    pub fn update_comments(&mut self, story_id: String, comments: Vec<HackerNewsComment>) {
        self.comments_cache.insert(story_id, (comments, std::time::Instant::now()));
    }
    
    pub fn get_cached_comments(&self, story_id: &str) -> Option<&Vec<HackerNewsComment>> {
        self.comments_cache.get(story_id).map(|(comments, _)| comments)
    }
}