use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use dirs_next;

use crate::models::HackerNewsItem;

#[derive(Debug, Clone)]
pub struct FavoriteStory {
    pub id: String,
    pub title: String,
    pub url: String,
    pub domain: String,
    pub by: String,
    pub score: i32,
    pub time_ago: String,
    pub comments_count: i32,
    pub added_at: DateTime<Utc>,
    pub done: bool,
}

// Structure to hold a viewed story with details
#[derive(Debug, Clone)]
pub struct ViewedStory {
    pub id: String,
    pub title: String,
    pub viewed_at: DateTime<Utc>,
}

impl From<HackerNewsItem> for FavoriteStory {
    fn from(item: HackerNewsItem) -> Self {
        Self {
            id: item.id,
            title: item.title,
            url: item.url,
            domain: item.domain,
            by: item.by,
            score: item.score,
            time_ago: item.time_ago,
            comments_count: item.comments_count,
            added_at: Utc::now(),
            done: false,
        }
    }
}

impl From<FavoriteStory> for HackerNewsItem {
    fn from(fav: FavoriteStory) -> Self {
        Self {
            id: fav.id,
            title: fav.title,
            url: fav.url,
            domain: fav.domain,
            by: fav.by,
            score: fav.score,
            time_ago: fav.time_ago,
            comments_count: fav.comments_count,
            original_index: 0, // Default to 0 for favorites since they don't have a natural ordering
        }
    }
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let app_data_dir = Self::get_app_data_dir()?;
        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir)?;
        }

        let db_path = app_data_dir.join("favorites.db");
        let conn = Connection::open(db_path)?;

        // Create the favorites table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorites (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                domain TEXT NOT NULL,
                by TEXT NOT NULL,
                score INTEGER NOT NULL,
                time_ago TEXT NOT NULL,
                comments_count INTEGER NOT NULL,
                added_at TEXT NOT NULL,
                done INTEGER DEFAULT 0
            )",
            [],
        )?;
        
        // Create the viewed_stories table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS viewed_stories (
                id TEXT PRIMARY KEY,
                viewed_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create the story_details table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS story_details (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create the settings table for app preferences
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        
        // Add the 'done' column if it doesn't exist (for existing databases)
        let columns = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('favorites') WHERE name = 'done'",
            [],
            |row| row.get::<_, i32>(0)
        )?;
        
        if columns == 0 {
            // The 'done' column doesn't exist, add it
            conn.execute(
                "ALTER TABLE favorites ADD COLUMN done INTEGER DEFAULT 0",
                [],
            )?;
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn get_app_data_dir() -> Result<PathBuf> {
        let home_dir = dirs_next::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".hn_reader"))
    }

    pub fn add_favorite(&self, story: &HackerNewsItem) -> Result<()> {
        let favorite = FavoriteStory::from(story.clone());
        
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        conn.execute(
            "INSERT OR REPLACE INTO favorites (id, title, url, domain, by, score, time_ago, comments_count, added_at, done) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                favorite.id,
                favorite.title,
                favorite.url,
                favorite.domain,
                favorite.by,
                favorite.score,
                favorite.time_ago,
                favorite.comments_count,
                favorite.added_at.to_rfc3339(),
                0, // not done by default
            ],
        )?;

        Ok(())
    }
    
    pub fn toggle_favorite_done(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        
        // Get current done status
        let done: i32 = conn.query_row(
            "SELECT done FROM favorites WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        
        // Toggle done status
        let new_done = if done == 0 { 1 } else { 0 };
        
        conn.execute(
            "UPDATE favorites SET done = ?1 WHERE id = ?2",
            params![new_done, id],
        )?;
        
        Ok(())
    }

    pub fn remove_favorite(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        conn.execute("DELETE FROM favorites WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn is_favorite(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM favorites WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
    
    pub fn clear_done_favorites(&self) -> Result<usize> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        let deleted_count = conn.execute("DELETE FROM favorites WHERE done = 1", [])?;
        Ok(deleted_count)
    }

    pub fn get_all_favorites(&self) -> Result<Vec<FavoriteStory>> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        let mut stmt = conn.prepare(
            "SELECT id, title, url, domain, by, score, time_ago, comments_count, added_at, done 
             FROM favorites 
             ORDER BY done ASC, added_at DESC"
        )?;

        let favorites_iter = stmt.query_map([], |row| {
            let added_at_str: String = row.get(8)?;
            let added_at = match DateTime::parse_from_rfc3339(&added_at_str) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => Utc::now(), // Fallback if parsing fails
            };
            
            let done_int: i32 = row.get(9).unwrap_or(0);
            let done = done_int != 0;

            Ok(FavoriteStory {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                domain: row.get(3)?,
                by: row.get(4)?,
                score: row.get(5)?,
                time_ago: row.get(6)?,
                comments_count: row.get(7)?,
                added_at,
                done,
            })
        })?;

        let mut favorites = Vec::new();
        for favorite in favorites_iter {
            favorites.push(favorite?);
        }

        Ok(favorites)
    }
    
    // Add a story to viewed stories
    pub fn mark_story_as_viewed(&self, story_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        conn.execute(
            "INSERT OR REPLACE INTO viewed_stories (id, viewed_at) VALUES (?1, ?2)",
            params![story_id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }
    
    // Check if a story has been viewed
    pub fn is_story_viewed(&self, story_id: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM viewed_stories WHERE id = ?1",
            params![story_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
    
    // Get all viewed story IDs
    pub fn get_viewed_story_ids(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        let mut stmt = conn.prepare("SELECT id FROM viewed_stories")?;
        let story_ids_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;
        
        let mut story_ids = Vec::new();
        for story_id in story_ids_iter {
            story_ids.push(story_id?);
        }
        
        Ok(story_ids)
    }
}

impl Database {
    // Get viewed stories with basic details
    pub fn get_viewed_stories(&self) -> Result<Vec<ViewedStory>> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        
        // First, create a temporary table with story details if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS story_details (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL
            )",
            [],
        )?;
        
        // Get all viewed stories with their timestamps
        let mut stmt = conn.prepare(
            "SELECT v.id, COALESCE(s.title, 'Unknown Title'), v.viewed_at 
             FROM viewed_stories v
             LEFT JOIN story_details s ON v.id = s.id
             ORDER BY v.viewed_at DESC"
        )?;
        
        let stories_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let viewed_at_str: String = row.get(2)?;
            
            let viewed_at = match DateTime::parse_from_rfc3339(&viewed_at_str) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => Utc::now(), // Fallback if parsing fails
            };
            
            Ok(ViewedStory {
                id,
                title,
                viewed_at,
            })
        })?;
        
        let mut stories = Vec::new();
        for story in stories_iter {
            stories.push(story?);
        }
        
        Ok(stories)
    }
    
    // Add or update story details (title, etc.)
    pub fn save_story_details(&self, id: &str, title: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        conn.execute(
            "INSERT OR REPLACE INTO story_details (id, title) VALUES (?1, ?2)",
            params![id, title],
        )?;
        Ok(())
    }
    
    // Save a setting to the database
    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
    
    // Get a setting from the database
    #[allow(dead_code)]
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().map_err(|_| anyhow!("Failed to lock database connection"))?;
        
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0)
        );
        
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow!(e)),
        }
    }
}