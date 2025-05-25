use eframe::egui;
use egui::{Color32, RichText, ScrollArea, Ui, ViewportBuilder, Stroke, CornerRadius};
use std::thread;
use std::sync::{Arc, Mutex};
use image::ImageReader;

mod hn_client;
mod models;
mod db;

use crate::hn_client::HackerNewsClient;
use crate::models::{HackerNewsItem, HackerNewsComment};
use crate::db::{Database, FavoriteStory};

// Create a global font size with proper synchronization
lazy_static::lazy_static! {
    static ref GLOBAL_FONT_SIZE: Mutex<f32> = Mutex::new(15.0);
}

// Function to load an image as an icon
fn load_icon(path: &str) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    // Open the image file
    let img = ImageReader::open(path)?.decode()?;
    
    // Convert the image to RGBA format
    let rgba_image = img.into_rgba8();
    let (width, height) = rgba_image.dimensions();
    
    // Create IconData from the image
    let icon_data = egui::IconData {
        rgba: rgba_image.into_raw(),
        width: width as u32,
        height: height as u32,
    };
    
    Ok(icon_data)
}

fn main() -> Result<(), eframe::Error> {
    // Load the icon image
    let icon_data = match load_icon("logo/logo.png") {
        Ok(icon) => Some(icon),
        Err(e) => {
            eprintln!("Failed to load icon: {}", e);
            None
        }
    };
    
    // Create application options with the icon
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Hacker News Reader"),
        ..Default::default()
    };
    
    // Set the icon if loaded successfully
    if let Some(icon) = icon_data {
        // Wrap the IconData in an Arc as required by eframe
        options.viewport.icon = Some(Arc::new(icon));
    }
    
    eframe::run_native(
        "Hacker News Reader",
        options,
        Box::new(|cc| {
            // Load saved app state if it exists
            let mut app = HackerNewsReaderApp::new();
            
            if let Some(storage) = cc.storage {
                // Try to load saved theme preference
                if let Some(theme_str) = storage.get_string("is_dark_mode") {
                    if let Ok(is_dark_mode) = theme_str.parse::<bool>() {
                        // Set the theme according to the saved preference
                        app.is_dark_mode = is_dark_mode;
                        app.theme = if is_dark_mode {
                            AppTheme::dark()
                        } else {
                            AppTheme::light()
                        };
                    }
                }
                
                // Try to load saved font size preference
                if let Some(font_size_str) = storage.get_string("comment_font_size") {
                    if let Ok(font_size) = font_size_str.parse::<f32>() {
                        // Set the global font size within valid range (10.0-24.0)
                        if let Ok(mut global_font_size) = GLOBAL_FONT_SIZE.lock() {
                            *global_font_size = font_size.max(10.0).min(24.0);
                        }
                    }
                }
            }
            
            Ok(Box::new(app))
        }),
    )
}

struct AppTheme {
    background: Color32,
    card_background: Color32,
    #[allow(dead_code)]
    header_background: Color32,
    text: Color32,
    secondary_text: Color32,
    highlight: Color32,
    accent: Color32,
    separator: Color32,
    score_high: Color32,
    score_medium: Color32,
    score_low: Color32,
    #[allow(dead_code)]
    link_color: Color32,
    button_background: Color32,
    button_foreground: Color32,
    button_active_background: Color32,
    button_hover_background: Color32,
}

impl AppTheme {
    // Returns a grayish color for viewed stories
    fn get_viewed_story_color(&self) -> Color32 {
        // Check if we're in dark mode or light mode
        let is_dark_mode = self.background.r() <= 128 || self.background.g() <= 128 || self.background.b() <= 128;
        
        if is_dark_mode {
            // Grayer text in dark mode (less bright)
            Color32::from_rgb(150, 150, 155)
        } else {
            // Grayer text in light mode (less contrast)
            Color32::from_rgb(120, 120, 125)
        }
    }
    
    fn dark() -> Self {
        Self {
            background: Color32::from_rgb(18, 18, 18),
            card_background: Color32::from_rgb(30, 30, 30),
            header_background: Color32::from_rgb(42, 42, 42),
            text: Color32::from_rgb(240, 240, 240),
            secondary_text: Color32::from_rgb(180, 180, 180),
            highlight: Color32::from_rgb(255, 102, 0), // HN orange
            accent: Color32::from_rgb(255, 153, 51),
            separator: Color32::from_rgb(60, 60, 60),
            score_high: Color32::from_rgb(76, 175, 80),    // Green
            score_medium: Color32::from_rgb(255, 193, 7),  // Yellow
            score_low: Color32::from_rgb(158, 158, 158),   // Gray
            link_color: Color32::from_rgb(100, 181, 246),  // Blue
            button_background: Color32::from_rgb(66, 66, 66),
            button_foreground: Color32::from_rgb(240, 240, 240),
            button_active_background: Color32::from_rgb(255, 102, 0),
            button_hover_background: Color32::from_rgb(80, 80, 80),
        }
    }
    
    fn light() -> Self {
        Self {
            background: Color32::from_rgb(245, 245, 245),
            card_background: Color32::from_rgb(255, 255, 255),
            header_background: Color32::from_rgb(235, 235, 235),
            text: Color32::from_rgb(20, 20, 20),
            secondary_text: Color32::from_rgb(90, 90, 90),  // Darker for better contrast
            highlight: Color32::from_rgb(235, 92, 0),       // Slightly darker orange for better contrast
            accent: Color32::from_rgb(220, 110, 20),        // Darker orange for better contrast
            separator: Color32::from_rgb(200, 200, 200),    // Darker separator for better visibility
            score_high: Color32::from_rgb(30, 110, 40),     // Darker green for better contrast
            score_medium: Color32::from_rgb(190, 130, 0),   // Darker yellow for better contrast
            score_low: Color32::from_rgb(80, 80, 80),       // Darker gray for better contrast
            link_color: Color32::from_rgb(20, 100, 200),    // Darker blue for better contrast
            button_background: Color32::from_rgb(235, 235, 235),
            button_foreground: Color32::from_rgb(20, 20, 20),
            button_active_background: Color32::from_rgb(235, 92, 0),  // Match highlight color
            button_hover_background: Color32::from_rgb(210, 210, 210), // More contrast for hover state
        }
    }
    
    fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Set base colors
        style.visuals.panel_fill = self.background;
        style.visuals.window_fill = self.card_background;
        style.visuals.window_stroke = Stroke::new(1.0, self.separator);
        style.visuals.widgets.noninteractive.bg_fill = self.card_background;
        
        // Set text colors
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text);
        
        // Set button styles
        style.visuals.widgets.inactive.bg_fill = self.button_background;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.button_foreground);
        style.visuals.widgets.active.bg_fill = self.button_active_background;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.button_foreground);
        style.visuals.widgets.hovered.bg_fill = self.button_hover_background;
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.button_foreground);
        
        // Set selection color
        style.visuals.selection.bg_fill = self.highlight;
        style.visuals.selection.stroke = Stroke::new(1.0, self.highlight);
        
        // Set various rounding amounts
        style.visuals.window_corner_radius = CornerRadius::same(8);
        style.visuals.menu_corner_radius = CornerRadius::same(6);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(4);
        
        // Determine if this is light or dark theme by checking background brightness
        let is_light_theme = self.background.r() > 128 && self.background.g() > 128 && self.background.b() > 128;
        
        // Set shadows based on theme
        if is_light_theme {
            // Light theme needs stronger shadows for depth
            style.visuals.popup_shadow = egui::epaint::Shadow {
                offset: [2, 2],
                blur: 8,
                spread: 1,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 30),
            };
            style.visuals.window_shadow = egui::epaint::Shadow {
                offset: [3, 3],
                blur: 12,
                spread: 2,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
            };
        } else {
            // Dark theme needs more subtle shadows
            style.visuals.popup_shadow = egui::epaint::Shadow {
                offset: [1, 1],
                blur: 6,
                spread: 0,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 50),
            };
            style.visuals.window_shadow = egui::epaint::Shadow {
                offset: [2, 2],
                blur: 10,
                spread: 1,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 40),
            };
        }
        
        // Apply the style
        ctx.set_style(style);
    }
    
    fn score_color(&self, score: i32) -> Color32 {
        // Determine if this is light or dark theme
        let is_dark_mode = self.background.r() <= 128 || self.background.g() <= 128 || self.background.b() <= 128;
        
        if score >= 500 {
            // Very high scores get an extra bright/saturated color
            if is_dark_mode {
                Color32::from_rgb(
                    self.score_high.r().saturating_add(20),
                    self.score_high.g().saturating_add(20),
                    self.score_high.b().saturating_add(5)
                )
            } else {
                Color32::from_rgb(15, 100, 30) // Darker, richer green for light mode
            }
        } else if score >= 300 {
            self.score_high
        } else if score >= 100 {
            self.score_medium
        } else {
            self.score_low
        }
    }
    
    // Get a color for story titles based on score, but with better readability
    fn get_title_color(&self, score: i32) -> Color32 {
        // Determine if this is light or dark theme by checking background brightness
        let is_dark_mode = self.background.r() <= 128 || self.background.g() <= 128 || self.background.b() <= 128;
        
        // For light theme, we need to ensure titles are dark enough to read
        // For dark theme, we need to ensure titles are bright enough
        if is_dark_mode {
            // In dark mode, brighten the colors a bit for better readability
            if score >= 500 {
                // Very high scores - brighter high score color
                Color32::from_rgb(
                    self.score_high.r().saturating_add(30),
                    self.score_high.g().saturating_add(30),
                    self.score_high.b().saturating_add(10)
                )
            } else if score >= 300 {
                // High scores - use high score color
                self.score_high
            } else if score >= 100 {
                // Medium scores - use medium score color
                self.score_medium
            } else {
                // Default color is brighter than secondary text
                self.text
            }
        } else {
            // In light mode, darken the colors a bit for better readability
            if score >= 500 {
                // Very high scores - darker high score color for contrast
                Color32::from_rgb(
                    self.score_high.r().saturating_sub(30),
                    self.score_high.g().saturating_sub(30),
                    self.score_high.b().saturating_sub(10)
                )
            } else if score >= 300 {
                // High scores - use high score color
                self.score_high
            } else if score >= 100 {
                // Medium scores - use medium score color
                self.score_medium
            } else {
                // Low scores - use normal text color for readability
                self.text
            }
        }
    }
    
    // Helper function to get the background color for story cards based on score
    fn get_card_background(&self, score: i32) -> Color32 {
        // Determine if this is light or dark theme by checking background brightness
        let is_dark_mode = self.background.r() <= 128 || self.background.g() <= 128 || self.background.b() <= 128;
        
        if score >= 500 {
            // Very high score - custom highlight
            if is_dark_mode {
                // Subtle green tint in dark mode
                Color32::from_rgba_premultiplied(40, 70, 40, 255)
            } else {
                // Very subtle green tint in light mode
                Color32::from_rgba_premultiplied(240, 250, 240, 255)
            }
        } else if score >= 300 {
            // High score - green highlight
            if is_dark_mode {
                // Slightly lighter background in dark mode with green tint
                Color32::from_rgba_premultiplied(
                    self.card_background.r().saturating_add(5),
                    self.card_background.g().saturating_add(15),
                    self.card_background.b().saturating_add(5),
                    255
                )
            } else {
                // Slightly darker background in light mode with green tint
                Color32::from_rgba_premultiplied(
                    self.card_background.r().saturating_sub(5),
                    self.card_background.g().saturating_sub(0), // Less reduction for green channel
                    self.card_background.b().saturating_sub(5),
                    255
                )
            }
        } else if score >= 100 {
            // Medium score - yellow/amber highlight
            if is_dark_mode {
                // Yellow/amber tint in dark mode
                Color32::from_rgba_premultiplied(
                    self.card_background.r().saturating_add(15),
                    self.card_background.g().saturating_add(10),
                    self.card_background.b().saturating_add(0),
                    255
                )
            } else {
                // Yellow/amber tint in light mode
                Color32::from_rgba_premultiplied(
                    253, 253, 235, 255 // Very subtle yellow tint
                )
            }
        } else {
            // Regular score - normal background
            self.card_background
        }
    }
    
    // Helper function to get the border stroke for story cards based on score
    fn get_card_stroke(&self, score: i32) -> Stroke {
        // Determine if this is light or dark theme by checking background brightness
        let is_dark_mode = self.background.r() <= 128 || self.background.g() <= 128 || self.background.b() <= 128;
        
        if score >= 500 {
            // Very high score - custom highlight border
            let color = if is_dark_mode {
                // Brighter green border in dark mode
                Color32::from_rgb(76, 175, 80) // Match score_high
            } else {
                // Darker green border in light mode
                Color32::from_rgb(46, 125, 50) // Darker green
            };
            Stroke::new(2.0, color)
        } else if score >= 300 {
            // High score - green border highlight
            let color = if is_dark_mode {
                // Green-tinted border in dark mode
                Color32::from_rgba_premultiplied(
                    self.separator.r().saturating_add(5),
                    self.separator.g().saturating_add(30),
                    self.separator.b().saturating_add(5),
                    255
                )
            } else {
                // Green-tinted border in light mode
                Color32::from_rgb(70, 150, 70) // Medium green
            };
            Stroke::new(1.5, color)
        } else if score >= 100 {
            // Medium score - yellow/amber border highlight
            let color = if is_dark_mode {
                // Yellow/amber border in dark mode
                Color32::from_rgba_premultiplied(
                    self.separator.r().saturating_add(40),
                    self.separator.g().saturating_add(35),
                    self.separator.b().saturating_add(0),
                    255
                )
            } else {
                // Yellow/amber border in light mode
                Color32::from_rgb(190, 150, 30) // Medium amber
            };
            Stroke::new(1.2, color)
        } else {
            // Regular score - normal border
            Stroke::new(1.0, self.separator)
        }
    }
    
}

// Define an enum for the different main tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Hot,
    New,
    Show,
    Ask,
    Jobs,
    Best,
}

// Define an enum for the side panel tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SidePanelTab {
    Favorites,
    History,
}

struct HackerNewsReaderApp {
    hn_client: HackerNewsClient,
    stories: Vec<HackerNewsItem>,
    selected_story: Option<HackerNewsItem>,
    // Index of the currently selected story for keyboard navigation
    selected_story_index: Option<usize>,
    comments: Vec<HackerNewsComment>,
    loading: bool,
    theme: AppTheme,
    is_dark_mode: bool,
    // Current active tab
    current_tab: Tab,
    // Current page for stories (for infinite scrolling)
    current_page: usize,
    // Flag to indicate if more stories are being loaded
    loading_more_stories: bool, 
    // Flag to indicate if we've reached the end of available stories
    end_of_stories: bool,
    // Change the thread type to handle any type of result
    load_thread: Option<thread::JoinHandle<Box<dyn std::any::Any + Send>>>,
    needs_repaint: bool,
    collapsed_comments: std::collections::HashSet<String>,
    stories_receiver: Option<std::sync::mpsc::Receiver<Option<Vec<HackerNewsItem>>>>,
    comments_receiver: Option<std::sync::mpsc::Receiver<Option<Vec<HackerNewsComment>>>>,
    story_fetch_receiver: Option<std::sync::mpsc::Receiver<Option<HackerNewsItem>>>,
    // Pagination for comments
    comments_page: usize,
    comments_per_page: usize,
    total_comments_count: usize,
    // ScrollArea control
    stories_scroll_offset: f32,
    comments_scroll_offset: f32,
    // Favorites
    database: Arc<Database>,
    favorites: Vec<FavoriteStory>,
    show_favorites_panel: bool,
    favorites_loading: bool,
    favorites_scroll_offset: f32,
    // Pending actions to avoid borrow checker issues
    pending_favorites_toggle: Option<String>,  // Story ID to toggle
    pending_todo_toggle: Option<String>,      // Story ID to toggle todo
    pending_done_toggle: Option<String>,      // Story ID to toggle done
    // Search functionality
    search_query: String,
    filtered_stories: Vec<HackerNewsItem>,
    show_search_ui: bool,
    // Filter options
    show_todo_only: bool,
    show_done_only: bool,
    // Status message for user feedback
    status_message: String,
    last_status_update_time: f64,
    // Focus control
    request_search_focus: bool,
    // Flag to auto-collapse comments when loading
    auto_collapse_on_load: bool,
    // Cache for cleaned HTML to improve performance with large comment threads
    clean_html_cache: std::collections::HashMap<String, String>,
    // Toggle to show latest comments first
    show_latest_comments_first: bool,
    // We'll remove the comment_font_size field from the struct
    // and use the global GLOBAL_FONT_SIZE instead
    // Set of story IDs that the user has viewed
    viewed_story_ids: std::collections::HashSet<String>,
    // Current side panel tab
    current_side_panel_tab: SidePanelTab,
    // Viewed stories for the history tab
    history_stories: Vec<db::ViewedStory>,
    // Flag to indicate if history is loading
    history_loading: bool,
    // Scroll offset for history panel
    history_scroll_offset: f32,
    // Search query for history
    history_search_query: String,
    // Share modal dialog state
    show_share_modal: bool,
    share_link_copied: bool
}

impl HackerNewsReaderApp {
    fn new() -> Self {
        // Create a test item for immediate display to check UI rendering
        let test_stories = vec![
            crate::models::HackerNewsItem {
                id: "debug1".to_string(),
                title: "Debug Story 1".to_string(),
                url: "https://example.com/1".to_string(),
                domain: "example.com".to_string(),
                by: "debug_user".to_string(),
                score: 123,
                time_ago: "2 hours ago".to_string(),
                comments_count: 45,
                original_index: 0,
            },
            crate::models::HackerNewsItem {
                id: "debug2".to_string(),
                title: "Debug Story 2".to_string(),
                url: "https://example.com/2".to_string(),
                domain: "example.com".to_string(),
                by: "debug_user2".to_string(),
                score: 234,
                time_ago: "3 hours ago".to_string(),
                comments_count: 67,
                original_index: 1,
            },
        ];
        
        // Initialize the database
        let database = match Database::new() {
            Ok(db) => Arc::new(db),
            Err(e) => {
                eprintln!("Failed to initialize database: {}", e);
                // Create a placeholder database - we'll still function without favorites
                Arc::new(Database::new().unwrap_or_else(|_| panic!("Failed to create placeholder database")))
            }
        };
        
        // Load initial favorites
        let favorites = match database.get_all_favorites() {
            Ok(favs) => favs,
            Err(e) => {
                eprintln!("Failed to load favorites: {}", e);
                Vec::new()
            }
        };
        
        Self {
            hn_client: HackerNewsClient::new(),
            // Uncomment to use test_stories for debugging
            stories: test_stories, // Use empty Vec::new() for network loading
            selected_story_index: None, // No story selected initially
            selected_story: None,
            comments: Vec::new(),
            loading: false,
            theme: AppTheme::dark(),
            is_dark_mode: true,
            current_tab: Tab::Hot, // Start with the Hot tab
            current_page: 1, // Start with page 1
            loading_more_stories: false,
            end_of_stories: false,
            load_thread: None,
            needs_repaint: false,
            collapsed_comments: std::collections::HashSet::new(),
            stories_receiver: None,
            comments_receiver: None,
            story_fetch_receiver: None,
            // Initialize pagination with reasonable defaults
            comments_page: 0,
            comments_per_page: 20, // Display 20 top-level comments per page
            total_comments_count: 0,
            // Initialize scroll offsets
            stories_scroll_offset: 0.0,
            comments_scroll_offset: 0.0,
            // Initialize favorites
            database: database.clone(),
            favorites,
            show_favorites_panel: false,
            favorites_loading: false,
            favorites_scroll_offset: 0.0,
            pending_favorites_toggle: None,
            pending_todo_toggle: None,
            pending_done_toggle: None,
            // Initialize search functionality
            search_query: String::new(),
            filtered_stories: Vec::new(),
            show_search_ui: false,
            show_todo_only: false,
            show_done_only: false,
            status_message: String::new(),
            last_status_update_time: 0.0,
            request_search_focus: false,
            // Initialize auto-collapse flag
            auto_collapse_on_load: true,
            // Initialize HTML cleaning cache
            clean_html_cache: std::collections::HashMap::new(),
            // Initialize comments order toggle (default to false - chronological order)
            show_latest_comments_first: false,
            // comment_font_size removed - using global value
            // Initialize viewed stories set
            viewed_story_ids: {
                // Load viewed stories from database
                let mut viewed_ids = std::collections::HashSet::new();
                match database.clone().get_viewed_story_ids() {
                    Ok(ids) => {
                        for id in ids {
                            viewed_ids.insert(id);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to load viewed stories: {}", e);
                    }
                }
                viewed_ids
            },
            // Initialize side panel tab
            current_side_panel_tab: SidePanelTab::Favorites,
            // Initialize history
            history_stories: Vec::new(),
            history_loading: false,
            history_scroll_offset: 0.0,
            history_search_query: String::new(),
            show_share_modal: false,
            share_link_copied: false
        }
    }
    
    fn load_stories(&mut self) {
        if self.loading {
            return; // Don't start another load if we're already loading
        }
        
        // Reset search state when loading fresh stories
        if self.show_search_ui {
            self.toggle_search_ui();
        } else {
            self.reset_all_filters();
        }
        
        self.loading = true;
        self.current_page = 1; // Reset to page 1 when loading fresh stories
        self.end_of_stories = false; // Reset end of stories flag
        
        // Create a new thread for loading
        let client = self.hn_client.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Convert the tab enum to a string
        let tab_str = match self.current_tab {
            Tab::Hot => "hot",
            Tab::New => "new",
            Tab::Show => "show",
            Tab::Ask => "ask",
            Tab::Jobs => "jobs",
            Tab::Best => "best",
        };
        
        let handle = thread::spawn(move || {
            let result: Box<dyn std::any::Any + Send> = match client.fetch_stories_by_tab(tab_str) {
                Ok(stories) => {
                    let _ = tx.send(Some(stories));
                    Box::new(())
                }
                Err(_) => {
                    let _ = tx.send(None::<Vec<HackerNewsItem>>);
                    Box::new(())
                }
            };
            result
        });
        
        self.load_thread = Some(handle);
        
        // Store the receiver for later checks
        self.stories_receiver = Some(rx);
    }
    
    fn load_more_stories(&mut self) {
        // Debug output turned off
        // println!("load_more_stories called with: loading={}, loading_more={}, end_reached={}, current_page={}", 
        //          self.loading, self.loading_more_stories, self.end_of_stories, self.current_page);
                 
        // Don't start another load if:
        // 1. We're already loading
        // 2. We've reached the end of stories
        // 3. We've reached the maximum page limit (5 pages = 150 stories)
        if self.loading || self.loading_more_stories || self.end_of_stories {
            // Debug output turned off
            // println!("  → ABORT: Already loading or reached end of stories");
            return;
        }
        
        // Check if we've reached the maximum number of pages (5 pages = 150 stories)
        const MAX_PAGES: usize = 5;
        if self.current_page >= MAX_PAGES {
            // Debug output turned off
            // println!("  → ABORT: Reached maximum page limit ({} pages)", MAX_PAGES);
            self.end_of_stories = true;
            return;
        }
        
        // Increment the page number
        self.current_page += 1;
        self.loading_more_stories = true;
        
        // Debug output turned off
        // println!("STARTING TO LOAD MORE STORIES (PAGE {}/{}) - loading_more_stories set to true", 
        //          self.current_page, MAX_PAGES);
        
        // Create a new thread for loading more stories
        let client = self.hn_client.clone();
        let page = self.current_page;
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Convert the tab enum to a string
        let tab_str = match self.current_tab {
            Tab::Hot => "hot",
            Tab::New => "new",
            Tab::Show => "show",
            Tab::Ask => "ask",
            Tab::Jobs => "jobs",
            Tab::Best => "best",
        };
        
        let handle = thread::spawn(move || {
            let result: Box<dyn std::any::Any + Send> = match client.fetch_stories_by_tab_and_page(tab_str, page) {
                Ok(stories) => {
                    let _ = tx.send(Some(stories));
                    Box::new(())
                }
                Err(_) => {
                    let _ = tx.send(None::<Vec<HackerNewsItem>>);
                    Box::new(())
                }
            };
            result
        });
        
        self.load_thread = Some(handle);
        
        // Store the receiver for later checks
        self.stories_receiver = Some(rx);
    }
    
    fn check_loading_thread(&mut self) {
        // Check for stories from the receiver
        if let Some(rx) = &self.stories_receiver {
            match rx.try_recv() {
                Ok(Some(stories)) => {
                    // Debug output turned off
                    // println!("RECEIVED {} STORIES FROM LOADING THREAD", stories.len());
                    
                    if self.loading_more_stories {
                        // Debug output turned off
                        // println!("Processing as additional stories (current_page={})", self.current_page);
                        
                        let _current_count = self.stories.len();
                        
                        // If we're loading more stories, append them to the existing list regardless of count
                        // We'll set end_of_stories only if we get zero stories
                        if stories.is_empty() {
                            // Only mark as end of stories if we get zero stories
                            // Debug output turned off
                            // println!("REACHED END OF STORIES (received 0 stories)");
                            self.end_of_stories = true;
                        } else {
                            // Otherwise, keep adding stories as normal
                            // Debug output turned off
                            // println!("ADDING {} MORE STORIES FOR PAGE {}", stories.len(), self.current_page);
                            
                            // Create set of existing IDs to avoid duplicates
                            let mut existing_ids = std::collections::HashSet::new();
                            for story in &self.stories {
                                existing_ids.insert(story.id.clone());
                            }
                            
                            // Count stories and store their length before iterating
                            let _stories_len = stories.len();
                            
                            // Only add stories that aren't already in our list
                            let mut added = 0;
                            for story in stories {
                                if !existing_ids.contains(&story.id) {
                                    self.stories.push(story);
                                    added += 1;
                                }
                            }
                            
                            // Debug output turned off
                            // println!("Added {} new stories (filtered out {} duplicates)", 
                            //          added, _stories_len - added);
                            // println!("Story count: {} → {}", _current_count, self.stories.len());
                            
                            // Handle different cases for detecting end of stories:
                            // 1. If we added ZERO new stories, we've reached the end
                            // 2. If we added very few stories and we're on a high page number
                            if added == 0 {
                                // Debug output turned off
                                // println!("NO new stories added, marking as end of content");
                                self.end_of_stories = true;
                            } 
                            // If we're on page 3+ and added fewer than 5 stories, likely the end
                            else if added < 5 && self.current_page >= 3 {
                                // Debug output turned off
                                // println!("Very few new stories ({}) added on page {}, marking as end of content", 
                                //          added, self.current_page);
                                self.end_of_stories = true;
                            }
                            // Allow first few pages to have fewer stories without ending
                            else if added < 2 && self.current_page >= 2 {
                                // Debug output turned off
                                // println!("Almost no new stories on page {}, marking as end of content", 
                                //          self.current_page);
                                self.end_of_stories = true;
                            }
                        }
                        self.loading_more_stories = false;
                        // Debug output turned off
                        // println!("loading_more_stories set to false");
                    } else {
                        // Otherwise, replace the existing stories
                        // Debug output turned off
                        // println!("Replacing existing stories with {} new stories", stories.len());
                        self.stories = stories;
                    }
                    self.loading = false;
                    self.stories_receiver = None; // Consume the receiver
                    self.needs_repaint = true;
                    // Debug output turned off
                    // println!("Loading completed, ready for next scroll event");
                }
                Ok(None) => {
                    if !self.loading_more_stories {
                        // Add a test item for debugging only if we're not loading more
                        self.stories = vec![
                            crate::models::HackerNewsItem {
                                id: "1".to_string(),
                                title: "Test Item - Loading Failed".to_string(),
                                url: "https://example.com".to_string(),
                                domain: "example.com".to_string(),
                                by: "test_user".to_string(),
                                score: 100,
                                time_ago: "1 hour ago".to_string(),
                                comments_count: 10,
                                original_index: 0,
                            }
                        ];
                    }
                    self.loading = false;
                    self.loading_more_stories = false;
                    self.stories_receiver = None; // Consume the receiver
                    self.needs_repaint = true;
                }
                Err(_) => {
                    // Still waiting for results
                }
            }
        }
        
        // Check for comments from the receiver
        if let Some(rx) = &self.comments_receiver {
            match rx.try_recv() {
                Ok(Some(comments)) => {
                    // Optimize comments for large threads
                    if comments.len() > 300 {
                        // Large comment thread - apply optimizations
                        let optimized = self.optimize_large_comment_thread(comments);
                        self.comments = optimized;
                    } else {
                        self.comments = comments;
                    }
                    
                    self.loading = false;
                    self.comments_receiver = None; // Consume the receiver
                    
                    // Auto-collapse top-level comments if the flag is set, but unfold the first one
                    if self.auto_collapse_on_load {
                        // Only process if we have comments
                        if !self.comments.is_empty() {
                            // First, collapse all top-level comments
                            for comment in &self.comments {
                                self.collapsed_comments.insert(comment.id.clone());
                            }
                            
                            // Then, if there's at least one comment, unfold the first one
                            if let Some(first_comment) = self.comments.first() {
                                self.collapsed_comments.remove(&first_comment.id);
                            }
                        }
                        
                        // Only auto-collapse once when comments are first loaded
                        self.auto_collapse_on_load = false;
                    }
                    
                    self.needs_repaint = true;
                }
                Ok(None) => {
                    // Failed to load comments, empty comments list is fine
                    self.comments = Vec::new();
                    self.loading = false;
                    self.comments_receiver = None; // Consume the receiver
                    self.needs_repaint = true;
                }
                Err(_) => {
                    // Still waiting for results
                }
            }
        }
        
        // Check for fetched individual story from the receiver
        if let Some(rx) = &self.story_fetch_receiver {
            match rx.try_recv() {
                Ok(Some(story)) => {
                    // Story fetched successfully, view its comments
                    self.view_comments(story, false);
                    self.story_fetch_receiver = None; // Consume the receiver
                    self.needs_repaint = true;
                }
                Ok(None) => {
                    // Failed to fetch story
                    eprintln!("Failed to fetch story from history");
                    self.story_fetch_receiver = None; // Consume the receiver
                    self.needs_repaint = true;
                }
                Err(_) => {
                    // Still waiting for results
                }
            }
        }
        
        // Check if the thread is finished
        if let Some(handle) = &self.load_thread {
            if handle.is_finished() {
                // Thread is done, reset the thread handle
                let thread = std::mem::take(&mut self.load_thread);
                
                // Try to join the thread, but we won't use its result
                // since we're using channels to communicate results
                if let Some(thread) = thread {
                    let _ = thread.join();
                }
                
                // Add fallback stories if we've lost the messages somehow
                if self.selected_story.is_none() && self.stories.is_empty() && self.stories_receiver.is_none() {
                    // If we still don't have stories, add a fallback one
                    self.stories = vec![
                        crate::models::HackerNewsItem {
                            id: "1".to_string(),
                            title: "Test Item - Loading Failed".to_string(),
                            url: "https://example.com".to_string(),
                            domain: "example.com".to_string(),
                            by: "test_user".to_string(),
                            score: 100,
                            time_ago: "1 hour ago".to_string(),
                            comments_count: 10,
                            original_index: 0,
                        }
                    ];
                    self.loading = false;
                    self.needs_repaint = true;
                }
            }
        }
    }
    
    fn load_comments(&mut self, item_id: &str) {
        if self.loading {
            return; // Don't start another load if we're already loading
        }
        
        self.loading = true;
        
        // Clone the client and item_id for the thread
        let client = self.hn_client.clone();
        let item_id = item_id.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Check if we should show latest comments first and use appropriate endpoint
        let show_latest = self.show_latest_comments_first;
        
        // Create a new thread for loading comments
        let handle = thread::spawn(move || {
            let result: Box<dyn std::any::Any + Send> = if show_latest {
                // Use the latest comments endpoint
                match client.fetch_latest_comments(&item_id) {
                    Ok(comments) => {
                        let _ = tx.send(Some(comments));
                        Box::new(())
                    }
                    Err(_) => {
                        let _ = tx.send(None);
                        Box::new(())
                    }
                }
            } else {
                // Use the standard comments endpoint
                match client.fetch_comments(&item_id) {
                    Ok(comments) => {
                        let _ = tx.send(Some(comments));
                        Box::new(())
                    }
                    Err(_) => {
                        let _ = tx.send(None);
                        Box::new(())
                    }
                }
            };
            result
        });
        
        self.load_thread = Some(handle);
        
        // Store the receiver for later checks
        self.comments_receiver = Some(rx);
    }
    
    fn view_comments(&mut self, story: HackerNewsItem, force_refresh: bool) {
        // Mark the story as viewed, including title
        self.mark_story_as_viewed(&story.id, Some(&story.title));
        
        self.selected_story = Some(story.clone());
        
        // Clear collapsed comments when loading a new story
        self.collapsed_comments.clear();
        
        // Reset pagination when loading a new story
        self.comments_page = 0;
        self.total_comments_count = story.comments_count as usize;
        
        // We'll set a flag to auto-collapse comments once they're loaded
        self.auto_collapse_on_load = true;
        
        if force_refresh {
            // Force refresh comments (bypass cache)
            if self.loading {
                return; // Don't start another load if we're already loading
            }
            
            self.loading = true;
            
            // Clone the client and item_id for the thread
            let client = self.hn_client.clone();
            let item_id = story.id.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            
            // Check if we should show latest comments first and use appropriate endpoint
            let show_latest = self.show_latest_comments_first;
            
            // Create a new thread for loading comments with bypass cache
            let handle = thread::spawn(move || {
                let result: Box<dyn std::any::Any + Send> = if show_latest {
                    // Use the latest comments endpoint
                    match client.fetch_latest_comments(&item_id) {
                        Ok(comments) => {
                            let _ = tx.send(Some(comments));
                            Box::new(())
                        }
                        Err(_) => {
                            let _ = tx.send(None::<Vec<HackerNewsComment>>);
                            Box::new(())
                        }
                    }
                } else {
                    // Use standard fetch without cache
                    match client.fetch_fresh_comments(&item_id) {
                        Ok(comments) => {
                            let _ = tx.send(Some(comments));
                            Box::new(())
                        }
                        Err(_) => {
                            let _ = tx.send(None::<Vec<HackerNewsComment>>);
                            Box::new(())
                        }
                    }
                };
                result
            });
            
            self.load_thread = Some(handle);
            
            // Store the receiver for later checks
            self.comments_receiver = Some(rx);
        } else {
            // Normal load using cache if available
            self.load_comments(&story.id);
        }
    }
    
    fn open_link(&self, url: &str) {
        if let Err(e) = open::that(url) {
            eprintln!("Failed to open URL: {}", e);
        }
    }
    
    fn toggle_theme(&mut self) {
        self.is_dark_mode = !self.is_dark_mode;
        self.theme = if self.is_dark_mode {
            AppTheme::dark()
        } else {
            AppTheme::light()
        };
        self.needs_repaint = true;
    }
    
    // Increase comment font size
    fn increase_comment_font_size(&mut self) {
        // Maximum font size to prevent UI issues
        const MAX_FONT_SIZE: f32 = 24.0;
        
        if let Ok(mut font_size) = GLOBAL_FONT_SIZE.lock() {
            // Increase by 1 point (use the global value)
            *font_size = (*font_size + 1.0).min(MAX_FONT_SIZE);
            
            // Save the new font size to the database
            self.save_font_size_setting(*font_size);
        }
        
        self.needs_repaint = true;
    }
    
    // Decrease comment font size
    fn decrease_comment_font_size(&mut self) {
        // Minimum font size for readability
        const MIN_FONT_SIZE: f32 = 10.0;
        
        if let Ok(mut font_size) = GLOBAL_FONT_SIZE.lock() {
            // Decrease by 1 point (use the global value)
            *font_size = (*font_size - 1.0).max(MIN_FONT_SIZE);
            
            // Save the new font size to the database
            self.save_font_size_setting(*font_size);
        }
        
        self.needs_repaint = true;
    }
    
    // Save the font size setting to the database
    fn save_font_size_setting(&self, font_size: f32) {
        if let Err(e) = self.database.save_setting("comment_font_size", &font_size.to_string()) {
            eprintln!("Failed to save font size setting: {}", e);
        }
    }
    
    // Load the font size setting from the database
    #[allow(dead_code)]
    fn load_font_size_setting(&self) -> Option<f32> {
        match self.database.get_setting("comment_font_size") {
            Ok(Some(value)) => {
                match value.parse::<f32>() {
                    Ok(font_size) => Some(font_size),
                    Err(_) => None,
                }
            },
            _ => None,
        }
    }
    
    fn switch_tab(&mut self, tab: Tab) {
        if self.current_tab != tab {
            self.current_tab = tab;
            
            // Clear any selected story when switching tabs
            self.selected_story = None;
            self.selected_story_index = None; // Reset the selected story index
            self.comments.clear();
            
            // Reset search state when switching tabs
            self.reset_all_filters();
            self.show_search_ui = false;
            
            // Reset pagination variables
            self.current_page = 1;
            self.end_of_stories = false;
            self.loading_more_stories = false; // Explicitly reset this flag to avoid getting stuck
            self.stories_scroll_offset = 0.0; // Reset scroll position
            
            // Debug output turned off
            // println!("Tab switched to {:?} - Reset pagination (page=1, end_of_stories=false)", tab);
            
            // Reload stories for the new tab
            self.load_stories();
            self.needs_repaint = true;
        }
    }
    
    // Toggle the search UI visibility
    fn toggle_search_ui(&mut self) {
        self.show_search_ui = !self.show_search_ui;
        if !self.show_search_ui {
            // Clear search and filters when hiding the search UI
            self.reset_all_filters();
        } else {
            // Request focus on the search field when showing it
            self.request_search_focus = true;
            self.needs_repaint = true;
        }
    }
    
    // Reset all filters (search, todo, done)
    fn reset_all_filters(&mut self) {
        self.search_query.clear();
        self.show_todo_only = false;
        self.show_done_only = false;
        self.filtered_stories.clear();
    }
    
    // Update status message with current time
    fn set_status_message(&mut self, message: String) {
        self.status_message = message;
        self.last_status_update_time = self.get_current_time();
        self.needs_repaint = true;
    }
    
    // Get current time in seconds
    fn get_current_time(&self) -> f64 {
        let now = std::time::SystemTime::now();
        let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap_or(std::time::Duration::from_secs(0));
        since_epoch.as_secs_f64()
    }
    
    fn toggle_todo_filter(&mut self) {
        self.show_todo_only = !self.show_todo_only;
        
        // Ensure we don't have both filters active at the same time
        if self.show_todo_only && self.show_done_only {
            self.show_done_only = false;
        }
        
        // Reapply filters
        self.apply_filters();
        self.needs_repaint = true;
    }
    
    fn toggle_done_filter(&mut self) {
        self.show_done_only = !self.show_done_only;
        
        // Ensure we don't have both filters active at the same time
        if self.show_done_only && self.show_todo_only {
            self.show_todo_only = false;
        }
        
        // Reapply filters
        self.apply_filters();
        self.needs_repaint = true;
    }
    
    // Apply the search filter to stories
    fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() && !self.show_todo_only && !self.show_done_only {
            // If no filters are active, clear filtered results
            self.filtered_stories.clear();
            return;
        }
        
        // Apply all filters (search + todo/done)
        self.apply_filters();
    }
    
    // Apply all filters (search, todo, done)
    fn apply_filters(&mut self) {
        // Start with all stories
        let mut filtered = self.stories.clone();
        
        // Apply search filter if there's a query
        if !self.search_query.is_empty() {
            // Convert search query to lowercase for case-insensitive search
            let query = self.search_query.to_lowercase();
            
            filtered = filtered.into_iter()
                .filter(|story| {
                    // Search in title, domain, and author
                    story.title.to_lowercase().contains(&query) || 
                    story.domain.to_lowercase().contains(&query) || 
                    story.by.to_lowercase().contains(&query)
                })
                .collect();
        }
        
        // Apply todo filter if active
        if self.show_todo_only {
            filtered = filtered.into_iter()
                .filter(|story| self.is_todo(&story.id))
                .collect();
        }
        
        // Apply done filter if active
        if self.show_done_only {
            filtered = filtered.into_iter()
                .filter(|story| self.is_done(&story.id))
                .collect();
        }
        
        self.filtered_stories = filtered;
    }
}

impl HackerNewsClient {
    pub fn clone(&self) -> Self {
        // Create a new client instance, but with the same cache
        let mut client = Self::new();
        client.cache = self.cache.clone();
        client.cache_ttl_secs = self.cache_ttl_secs;
        client
    }
}

// Add a trait implementation for saving app state
impl eframe::App for HackerNewsReaderApp {
    // Save the app state when the app is closing
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save theme preference
        storage.set_string("is_dark_mode", self.is_dark_mode.to_string());
        
        // Save font size preference from global value
        if let Ok(font_size) = GLOBAL_FONT_SIZE.lock() {
            storage.set_string("comment_font_size", font_size.to_string());
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply our custom theme
        self.theme.apply_to_ctx(ctx);
        
        // Check if we have finished loading
        self.check_loading_thread();
        
        // Initialize loading on first frame
        static mut FIRST_FRAME: bool = true;
        unsafe {
            if FIRST_FRAME {
                self.load_stories();
                self.reload_favorites();
                FIRST_FRAME = false;
            }
        }
        
        // Automatic reload if we have no stories and aren't currently loading
        if self.stories.is_empty() && !self.loading && self.load_thread.is_none() {
            self.load_stories();
        }
        
        // Process comment collapse/expand buttons
        self.process_comment_collapse_buttons(ctx);
        
        // Process keyboard shortcuts
        self.process_keyboard_shortcuts(ctx);
        
        // Process any pending actions
        if let Some(story_id) = self.pending_favorites_toggle.take() {
            // Find the story by ID either in stories or selected story
            let story_opt = 
                if let Some(ref selected) = self.selected_story {
                    if selected.id == story_id {
                        Some(selected.clone())
                    } else {
                        None
                    }
                } else {
                    self.stories.iter().find(|s| s.id == story_id).cloned()
                };
                
            if let Some(story) = story_opt {
                // Call the toggle_favorite method
                self.toggle_favorite(&story);
            }
            
            self.needs_repaint = true;
        }
        
        // Process pending todo toggle
        if let Some(story_id) = self.pending_todo_toggle.take() {
            let story_opt = 
                if let Some(ref selected) = self.selected_story {
                    if selected.id == story_id {
                        Some(selected.clone())
                    } else {
                        None
                    }
                } else {
                    self.stories.iter().find(|s| s.id == story_id).cloned()
                };
                
            if let Some(story) = story_opt {
                // Add to todo list (add to favorites and ensure not marked as done)
                self.add_to_todo(&story);
                
                // Show status message
                self.set_status_message(format!("Added '{}' to your todo list", story.title));
                
                self.needs_repaint = true;
            }
        }
        
        // Process pending done toggle
        if let Some(story_id) = self.pending_done_toggle.take() {
            let story_opt = 
                if let Some(ref selected) = self.selected_story {
                    if selected.id == story_id {
                        Some(selected.clone())
                    } else {
                        None
                    }
                } else {
                    self.stories.iter().find(|s| s.id == story_id).cloned()
                };
                
            if let Some(story) = story_opt {
                // Check current done status for the message
                let is_done = self.is_done(&story_id);
                
                // Toggle done status
                self.toggle_done(&story);
                
                // Show status message
                if is_done {
                    self.set_status_message(format!("Marked '{}' as not done", story.title));
                } else {
                    self.set_status_message(format!("Marked '{}' as done", story.title));
                }
                
                self.needs_repaint = true;
            }
        }
        
        // Removed debug code and runtime storage saving
        
        // Request repaint if needed
        if self.needs_repaint {
            ctx.request_repaint();
            self.needs_repaint = false;
        }
        
        // Render side panel if it's visible
        if self.show_favorites_panel {
            self.render_side_panel(ctx);
        }
        
        // Render status message if present
        if !self.status_message.is_empty() {
            // Create a small panel at the bottom for status messages
            egui::TopBottomPanel::bottom("status_panel")
                .frame(egui::Frame::new()
                    .fill(self.theme.card_background)
                    .stroke(Stroke::new(1.0, self.theme.separator))
                    .inner_margin(8.0)
                    .outer_margin(0.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&self.status_message)
                                .color(self.theme.text)
                                .size(14.0)
                        );
                        
                        // Add a clear button on the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("×").clicked() {
                                self.status_message.clear();
                            }
                        });
                    });
                });
                
            // Clear status message after 3 seconds
            if ctx.input(|i| i.time - self.last_status_update_time > 3.0) {
                self.status_message.clear();
                self.needs_repaint = true;
            }
        }
        
        // Set up main layout
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a top header bar
            ui.horizontal(|ui| {
                // Side panel toggle button
                let panel_btn = ui.add(
                    egui::Button::new(
                        RichText::new("☰")  // Hamburger menu icon
                            .color(if self.show_favorites_panel { self.theme.highlight } else { self.theme.button_foreground })
                            .size(22.0)
                    )
                    .min_size(egui::Vec2::new(32.0, 32.0))
                    .corner_radius(CornerRadius::same(6))
                    .fill(self.theme.button_background)
                );
                
                if panel_btn.clicked() {
                    self.toggle_favorites_panel();
                }
                
                if panel_btn.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    
                    // Show tooltip
                    let tooltip_pos = egui::pos2(
                        panel_btn.rect.left(),
                        panel_btn.rect.bottom() + 5.0
                    );
                    
                    egui::Area::new(egui::Id::new("panel_tooltip_area"))
                        .order(egui::Order::Tooltip)
                        .fixed_pos(tooltip_pos)
                        .show(ctx, |ui| {
                            egui::Frame::popup(ui.style())
                                .fill(self.theme.card_background)
                                .stroke(Stroke::new(1.0, self.theme.separator))
                                .corner_radius(CornerRadius::same(6))
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(
                                        if self.show_favorites_panel {
                                            "Hide Favorites"
                                        } else {
                                            "Show Favorites"
                                        }
                                    ));
                                });
                        });
                }
                
                ui.add_space(8.0);
                ui.heading(
                    RichText::new("Hacker News Reader")
                        .color(self.theme.highlight)
                        .size(24.0)
                );
                
                ui.add_space(20.0);
                
                // Navigation bar for tabs
                ui.horizontal(|ui| {
                    self.render_tab_buttons(ui);
                });
                
                // Push buttons to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Search button
                    let search_icon = "🔍"; // Magnifying glass icon
                    let search_btn = ui.add(
                        egui::Button::new(
                            RichText::new(search_icon)
                                .color(if self.show_search_ui { self.theme.highlight } else { self.theme.button_foreground })
                                .size(18.0)
                        )
                        .min_size(egui::Vec2::new(32.0, 32.0))
                        .corner_radius(CornerRadius::same(16)) // Make it circular
                        .fill(self.theme.button_background)
                    );
                    
                    if search_btn.clicked() {
                        self.toggle_search_ui();
                    }
                    
                    if search_btn.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        
                        // Show search tooltip
                        let tooltip_pos = egui::pos2(
                            search_btn.rect.left() - 80.0,
                            search_btn.rect.bottom() + 5.0
                        );
                        
                        egui::Area::new(egui::Id::new("search_tooltip_area"))
                            .order(egui::Order::Tooltip)
                            .fixed_pos(tooltip_pos)
                            .show(ctx, |ui| {
                                egui::Frame::popup(ui.style())
                                    .fill(self.theme.card_background)
                                    .stroke(Stroke::new(1.0, self.theme.separator))
                                    .corner_radius(CornerRadius::same(6))
                                    .show(ui, |ui| {
                                        ui.add(egui::Label::new(
                                            if self.show_search_ui {
                                                "Hide Search"
                                            } else {
                                                "Show Search"
                                            }
                                        ));
                                    });
                            });
                    }
                    
                    ui.add_space(12.0);
                    
                    // Theme toggle button
                    let theme_icon = if self.is_dark_mode { "☀" } else { "☾" }; // Sun for light mode, moon for dark mode
                    let theme_btn = ui.add(
                        egui::Button::new(
                            RichText::new(theme_icon)
                                .color(self.theme.button_foreground)
                                .size(22.0)
                        )
                        .min_size(egui::Vec2::new(32.0, 32.0))
                        .corner_radius(CornerRadius::same(16)) // Make it circular
                        .fill(self.theme.button_background)
                    );
                    
                    // Add hover effect for theme button
                    if theme_btn.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        
                        // Show theme tooltip
                        let screen_rect = ctx.screen_rect();
                        let tooltip_pos = egui::pos2(
                            screen_rect.right() - 200.0, 
                            screen_rect.top() + 50.0
                        );
                        
                        egui::Window::new("theme_tooltip")
                            .id(egui::Id::new("stable_theme_window"))
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .fixed_pos(tooltip_pos)
                            .fixed_size([140.0, 30.0])
                            .frame(egui::Frame::window(&ctx.style())
                                .fill(self.theme.card_background)
                                .stroke(Stroke::new(1.0, self.theme.separator))
                                .corner_radius(CornerRadius::same(6))
                                .shadow(egui::epaint::Shadow::NONE))
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    let text = if self.is_dark_mode { "Switch to Light Mode" } else { "Switch to Dark Mode" };
                                    ui.add(egui::Label::new(RichText::new(text).size(14.0)));
                                });
                            });
                    }
                    
                    // Handle theme toggle
                    if theme_btn.clicked() {
                        self.toggle_theme();
                        // Request immediate repaint to avoid a frame with the old theme
                        ctx.request_repaint();
                    }
                    
                    ui.add_space(12.0);
                
                    // Refresh button
                    let refresh_btn = ui.add(
                        egui::Button::new(
                            RichText::new("↻") // Unicode refresh symbol
                                .color(self.theme.button_foreground)
                                .size(22.0)
                        )
                        .min_size(egui::Vec2::new(32.0, 32.0))
                        .corner_radius(CornerRadius::same(16)) // Make it circular
                        .fill(self.theme.button_background)
                    );
                    
                    // Add hover effect
                    if refresh_btn.hovered() {
                        // Set cursor on hover without consuming the response
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                    
                    // Always force refresh (bypass cache) when refresh button is clicked
                    if refresh_btn.clicked() && !self.loading {
                        self.refresh_current_view(true); // Force refresh (bypass cache)
                    }
                    
                    // Show tooltip for refresh with maximum stability
                    // Only show the tooltip when hovering and not refreshing
                    if refresh_btn.hovered() && !self.loading {
                        // Use a more stable fixed position that doesn't depend on the button's position
                        // This helps prevent flickering caused by layout recalculations
                        let screen_rect = ctx.screen_rect();
                        let tooltip_pos = egui::pos2(
                            screen_rect.right() - 150.0,  // Fixed distance from right edge
                            screen_rect.top() + 50.0      // Fixed distance from top
                        );
                        
                        // Create a completely stable tooltip area with no dynamic positioning
                        egui::Window::new("refresh_tooltip")
                            .id(egui::Id::new("stable_refresh_window"))
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .fixed_pos(tooltip_pos)
                            .fixed_size([140.0, 40.0])  // Fixed size to prevent any layout changes
                            .frame(egui::Frame::window(&ctx.style())
                                .fill(self.theme.card_background)
                                .stroke(Stroke::new(1.0, self.theme.separator))
                                .corner_radius(CornerRadius::same(6))
                                .shadow(egui::epaint::Shadow::NONE))  // No shadow to prevent flicker
                            .show(ctx, |ui| {
                                // Use fixed size labels to prevent layout shifts
                                ui.set_max_width(140.0);
                                ui.vertical_centered(|ui| {
                                    ui.add(egui::Label::new(RichText::new("Refresh").size(14.0)));
                                    ui.add(egui::Label::new(
                                        RichText::new("Ctrl+R to refresh")
                                            .size(12.0)
                                            .color(self.theme.secondary_text)
                                    ));
                                });
                            });
                    }
                    ui.add_space(8.0);
                }); // End right-to-left layout
            });
            
            ui.add(egui::Separator::default().spacing(12.0));
            
            // Show search field when search UI is enabled
            if self.show_search_ui {
                // Search UI
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Search:").color(self.theme.text).size(16.0));
                    ui.add_space(8.0);
                    
                    // Text field for search query
                    // If focus was requested, request it from egui
                    let search_input_id = egui::Id::new("search_input");
                    if self.request_search_focus {
                        // Request focus on the search input
                        ui.ctx().memory_mut(|mem| mem.request_focus(search_input_id));
                        // Also clear the current search to provide a fresh start
                        self.search_query.clear();
                        self.request_search_focus = false;
                    }
                    
                    let text_edit = ui.add_sized(
                        [ui.available_width() - 260.0, 32.0], // Make room for filter buttons
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Enter keywords to filter stories...")
                            .text_color(self.theme.text)
                            .cursor_at_end(true)
                            .frame(true)
                            .id(search_input_id) // Use the same ID for focus control
                    );
                    
                    // If the text edit gained focus, update UI
                    if text_edit.gained_focus() {
                        self.needs_repaint = true;
                    }
                    
                    // Apply search filter when text changes
                    if text_edit.changed() {
                        self.apply_search_filter();
                    }
                    
                    // Todo filter button
                    ui.add_space(8.0);
                    
                    // Todo button color based on active state
                    let todo_btn_color = if self.show_todo_only {
                        Color32::from_rgb(46, 204, 113) // Green for active
                    } else {
                        self.theme.button_foreground
                    };
                    
                    let todo_btn = ui.add_sized(
                        [80.0, 32.0],
                        egui::Button::new(
                            RichText::new("TODO")
                                .color(todo_btn_color)
                                .size(14.0)
                        )
                        .corner_radius(CornerRadius::same(6))
                        .fill(self.theme.button_background)
                    );
                    
                    if todo_btn.clicked() {
                        self.toggle_todo_filter();
                    }
                    
                    // Done filter button
                    ui.add_space(8.0);
                    
                    // Done button color based on active state
                    let done_btn_color = if self.show_done_only {
                        Color32::from_rgb(52, 152, 219) // Blue for active
                    } else {
                        self.theme.button_foreground
                    };
                    
                    let done_btn = ui.add_sized(
                        [80.0, 32.0],
                        egui::Button::new(
                            RichText::new("DONE")
                                .color(done_btn_color)
                                .size(14.0)
                        )
                        .corner_radius(CornerRadius::same(6))
                        .fill(self.theme.button_background)
                    );
                    
                    if done_btn.clicked() {
                        self.toggle_done_filter();
                    }
                    
                    // Clear button
                    if !self.search_query.is_empty() {
                        ui.add_space(8.0);
                        let clear_btn = ui.add_sized(
                            [60.0, 28.0],
                            egui::Button::new(
                                RichText::new("Clear")
                                    .color(self.theme.button_foreground)
                                    .size(14.0)
                            )
                            .fill(self.theme.button_background)
                        );
                        
                        if clear_btn.clicked() {
                            self.reset_all_filters();
                        }
                    }
                });
                
                // Display results summary if there's a search query or active filters
                if !self.search_query.is_empty() || self.show_todo_only || self.show_done_only {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let results_count = self.filtered_stories.len();
                        let total_count = self.stories.len();
                        
                        // Build filter info text
                        let mut filter_text = String::new();
                        
                        if !self.search_query.is_empty() {
                            filter_text.push_str(&format!("search query \"{}\"", self.search_query));
                        }
                        
                        if self.show_todo_only {
                            if !filter_text.is_empty() {
                                filter_text.push_str(" and ");
                            }
                            filter_text.push_str("TODO filter");
                        }
                        
                        if self.show_done_only {
                            if !filter_text.is_empty() {
                                filter_text.push_str(" and ");
                            }
                            filter_text.push_str("DONE filter");
                        }
                        
                        ui.label(
                            RichText::new(format!("Found {} results from {} stories using {}", 
                                results_count, total_count, filter_text))
                                .color(self.theme.secondary_text)
                                .size(14.0)
                                .italics()
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Keyboard hint
                            ui.label(
                                RichText::new("Press ESC to close search")
                                    .color(self.theme.secondary_text)
                                    .size(13.0)
                                    .italics()
                            );
                        });
                    });
                }
                
                ui.add_space(8.0);
                ui.add(egui::Separator::default().spacing(8.0));
            }
            
            // Loading indicator with a more modern spinner
            if self.loading {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.spinner();
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Loading...")
                            .color(self.theme.secondary_text)
                            .size(18.0)
                    );
                });
                return;
            }
            
            let clear_story = if let Some(_story) = &self.selected_story {
                // No need to store the title if we're not using it
                let mut clear = false;
                
                // Back button
                ui.horizontal(|ui| {
                    let back_btn = ui.add_sized(
                        [40.0, 30.0],
                        egui::Button::new(
                            RichText::new("⬅") // Left arrow (U+2B05) instead of ← (U+2190)
                                .size(18.0)
                                .color(self.theme.button_foreground)
                        )
                        .corner_radius(CornerRadius::same(6))
                        .fill(self.theme.button_background)
                    );
                    
                    // Add tooltip for the back button with improved stability
                    if back_btn.hovered() {
                        // Use a fixed tooltip position relative to the button
                        let tooltip_pos = back_btn.rect.left_top() + egui::vec2(0.0, -30.0);
                        
                        egui::Area::new("back_tooltip_area".into())
                            .order(egui::Order::Tooltip)
                            .fixed_pos(tooltip_pos)
                            .show(ui.ctx(), |ui| {
                                egui::Frame::popup(ui.style())
                                    .fill(self.theme.card_background)
                                    .stroke(Stroke::new(1.0, self.theme.separator))
                                    .corner_radius(CornerRadius::same(6))
                                    .show(ui, |ui| {
                                        ui.add(egui::Label::new("Back to Stories (or press Backspace)"));
                                    });
                            });
                    }
                    
                    if back_btn.clicked() {
                        clear = true;
                    }
                    
                    // Add backspace hint
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Press Backspace to return")
                            .size(13.0)
                            .color(self.theme.secondary_text)
                            .italics()
                    );
                });
                
                clear
            } else {
                false
            };

            if clear_story {
                self.selected_story = None;
                self.comments.clear();
            }

            if let Some(ref selected_story) = self.selected_story {
                // Clone the story to avoid borrow checker issues
                let story = selected_story.clone();
                
                // Story title with color based on score
                ui.add_space(8.0);
                let title_color = self.theme.get_title_color(story.score);
                // Use a different color for viewed stories
                let color = if self.is_story_viewed(&story.id) {
                    // Use grayish color for viewed stories
                    self.theme.get_viewed_story_color()
                } else {
                    // Use normal color based on score
                    title_color
                };
                
                ui.label(
                    RichText::new(&story.title)
                        .size(22.0)
                        .color(color)
                        .strong()
                );
                ui.add_space(8.0);
                
                // Get card background based on score using our helper method
                let card_background = self.theme.get_card_background(story.score);
                
                // Get the appropriate border stroke based on score
                let card_stroke = self.theme.get_card_stroke(story.score);
                
                // Story details card with background and border based on score
                egui::Frame::new()
                    .fill(card_background)
                    .corner_radius(CornerRadius::same(8))
                    .stroke(card_stroke)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        // Top row with score and domain
                        ui.horizontal(|ui| {
                            let score_color = self.theme.score_color(story.score);
                            ui.label(
                                RichText::new(format!("{} points", story.score))
                                    .color(score_color)
                                    .strong()
                            );
                            
                            if !story.domain.is_empty() {
                                ui.label(RichText::new("|").color(self.theme.separator));
                                ui.label(
                                    RichText::new(&story.domain)
                                        .color(self.theme.secondary_text)
                                        .italics()
                                );
                            }
                            
                            ui.label(RichText::new("|").color(self.theme.separator));
                            ui.label(
                                RichText::new(&story.time_ago)
                                    .color(self.theme.secondary_text)
                            );
                        });
                        
                        // Second row with author and buttons
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("by").color(self.theme.secondary_text));
                            ui.label(
                                RichText::new(&story.by)
                                    .color(self.theme.text)
                                    .strong()
                            );
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Open article button
                                if !story.url.is_empty() {
                                    let article_btn = ui.add_sized(
                                        [40.0, 30.0],
                                        egui::Button::new(
                                            RichText::new("↗")
                                                .size(18.0)
                                                .color(self.theme.button_foreground)
                                        )
                                        .corner_radius(CornerRadius::same(6))
                                        .fill(self.theme.button_background)
                                    );
                                    
                                    // Add tooltip for the article button with improved stability
                                    if article_btn.hovered() {
                                        let tooltip_pos = article_btn.rect.left_top() + egui::vec2(0.0, -30.0);
                                        
                                        egui::Area::new("article_tooltip_area".into())
                                            .order(egui::Order::Tooltip)
                                            .fixed_pos(tooltip_pos)
                                            .show(ui.ctx(), |ui| {
                                                egui::Frame::popup(ui.style())
                                                    .fill(self.theme.card_background)
                                                    .stroke(Stroke::new(1.0, self.theme.separator))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .show(ui, |ui| {
                                                        ui.add(egui::Label::new("Open Article"));
                                                    });
                                            });
                                    }
                                    
                                    if article_btn.clicked() {
                                        self.open_link(&story.url);
                                    }
                                    
                                    ui.add_space(8.0);
                                }
                                
                                // Favorite button
                                let story_id = &story.id;
                                let is_favorite = self.is_favorite(story_id);
                                let favorite_color = if is_favorite {
                                    Color32::from_rgb(255, 204, 0) // Gold star color for favorited
                                } else {
                                    self.theme.secondary_text // Gray star for not favorited
                                };
                                
                                let favorite_btn = ui.add_sized(
                                    [40.0, 30.0],
                                    egui::Button::new(
                                        RichText::new("★") // Star symbol
                                            .size(18.0)
                                            .color(favorite_color)
                                    )
                                    .corner_radius(CornerRadius::same(6))
                                    .fill(self.theme.button_background)
                                );
                                
                                // Add tooltip for the favorite button
                                if favorite_btn.hovered() {
                                    let tooltip_pos = favorite_btn.rect.left_top() + egui::vec2(0.0, -30.0);
                                    
                                    egui::Area::new("favorite_tooltip_area".into())
                                        .order(egui::Order::Tooltip)
                                        .fixed_pos(tooltip_pos)
                                        .show(ui.ctx(), |ui| {
                                            egui::Frame::popup(ui.style())
                                                .fill(self.theme.card_background)
                                                .stroke(Stroke::new(1.0, self.theme.separator))
                                                .corner_radius(CornerRadius::same(6))
                                                .show(ui, |ui| {
                                                    ui.add(egui::Label::new(
                                                        if is_favorite {
                                                            "Remove from Favorites"
                                                        } else {
                                                            "Add to Favorites"
                                                        }
                                                    ));
                                                });
                                        });
                                }
                                
                                if favorite_btn.clicked() {
                                    // Set pending toggle
                                    self.pending_favorites_toggle = Some(story.id.clone());
                                }
                                
                                // Add space between buttons
                                ui.add_space(8.0);
                                
                                // Share button with improved icon
                                let share_btn = ui.add_sized(
                                    [40.0, 30.0],
                                    egui::Button::new(
                                        RichText::new("S")  // Simple "S" for Share - guaranteed to display in all fonts
                                            .size(18.0)
                                            .color(self.theme.button_foreground)
                                    )
                                    .corner_radius(CornerRadius::same(6))
                                    .fill(self.theme.button_background)
                                );
                                
                                // Add tooltip for the share button
                                if share_btn.hovered() {
                                    let tooltip_pos = share_btn.rect.left_top() + egui::vec2(0.0, -30.0);
                                    
                                    egui::Area::new("share_tooltip_area".into())
                                        .order(egui::Order::Tooltip)
                                        .fixed_pos(tooltip_pos)
                                        .show(ui.ctx(), |ui| {
                                            egui::Frame::popup(ui.style())
                                                .fill(self.theme.card_background)
                                                .stroke(Stroke::new(1.0, self.theme.separator))
                                                .corner_radius(CornerRadius::same(6))
                                                .show(ui, |ui| {
                                                    ui.add(egui::Label::new("Share Article"));
                                                });
                                        });
                                }
                                
                                if share_btn.clicked() {
                                    // Open sharing modal dialog
                                    self.show_share_modal = true;
                                }
                            });
                        });
                    });
                
                ui.add_space(12.0);
                ui.add(egui::Separator::default().spacing(8.0));
                ui.add_space(4.0);
                
                // Comments header with keyboard shortcuts
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{} Comments", story.comments_count))
                            .size(18.0)
                            .color(self.theme.text)
                    );
                    
                    // Display keyboard navigation hint
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Keyboard: Arrows to scroll, Space for Page Down, Backspace to go back, Ctrl+O to open article, Ctrl+L to copy link")
                                .size(13.0)
                                .color(self.theme.secondary_text)
                                .italics()
                        );
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Help button with keyboard shortcuts
                        let help_btn = ui.add(
                            egui::Button::new(
                                RichText::new("?")
                                    .color(self.theme.secondary_text)
                                    .size(16.0)
                            )
                            .min_size(egui::Vec2::new(24.0, 24.0))
                            .corner_radius(CornerRadius::same(12))
                            .fill(self.theme.button_background)
                        );
                        
                        if help_btn.hovered() {
                            // Get screen dimensions for stable positioning
                            let screen_rect = ctx.screen_rect();
                            
                            // Position tooltip at a fixed distance from the top-right corner
                            // This creates more stability than positioning relative to the button
                            let tooltip_pos = egui::pos2(
                                screen_rect.right() - 250.0,  // Fixed distance from right edge
                                screen_rect.top() + 140.0     // Fixed distance from top
                            );
                            
                            // Use a completely stable window with fixed position and size
                            egui::Window::new("shortcuts_help")
                                .id(egui::Id::new("stable_shortcuts_window"))
                                .title_bar(false)
                                .resizable(false)
                                .collapsible(false)
                                .fixed_pos(tooltip_pos)
                                .fixed_size([220.0, 240.0])  // Fixed size to prevent any layout changes
                                .frame(egui::Frame::window(&ctx.style())
                                    .fill(self.theme.card_background)
                                    .stroke(Stroke::new(1.0, self.theme.separator))
                                    .corner_radius(CornerRadius::same(6))
                                    .shadow(egui::epaint::Shadow::NONE))  // No shadow to prevent flicker
                                .show(ctx, |ui| {
                                    ui.set_max_width(220.0);
                                    ui.vertical(|ui| {
                                        ui.add(egui::Label::new(RichText::new("Keyboard Shortcuts:").strong()));
                                        ui.add_space(4.0);
                                        
                                        ui.add(egui::Label::new(RichText::new("Comment Controls:").strong()));
                                        ui.add(egui::Label::new("C - Collapse all top-level comments"));
                                        ui.add(egui::Label::new("Shift+C - Expand all comments"));
                                        ui.add(egui::Label::new("Backspace - Return to stories"));
                                        
                                        ui.add_space(4.0);
                                        ui.add(egui::Label::new(RichText::new("General Controls:").strong()));
                                        ui.add(egui::Label::new("Ctrl+R - Refresh current view"));
                                        ui.add(egui::Label::new("Ctrl+L - Copy article link to clipboard"));
                                        ui.add(egui::Label::new("Ctrl+O - Open article in browser"));
                                        
                                        ui.add_space(4.0);
                                        ui.add(egui::Label::new(RichText::new("Navigation:").strong()));
                                        ui.add(egui::Label::new("Left/Right - Previous/Next page"));
                                        ui.add(egui::Label::new("Up/Down - Scroll up/down"));
                                        ui.add(egui::Label::new("Space - Page down"));
                                        ui.add(egui::Label::new("Home/End - First/Last page"));
                                        
                                        ui.add_space(4.0);
                                        ui.add(egui::Label::new(RichText::new("Mouse:").strong()));
                                        ui.add(egui::Label::new("Click [+]/[-] to collapse/expand"));
                                    });
                                });
                        }
                    });
                });
                
                ui.add_space(8.0);
                
                // Pagination controls at the top
                self.render_pagination_controls(ui);
                
                // Comments section with scrolling - use ID for persistent state
                // Set up a regular scroll area without virtual scrolling
                // Get comments for the current page only
                let page_comments = self.get_current_page_comments();
                
                // Create a simple ScrollArea without the virtual list logic
                // This provides more stable scrolling behavior
                let scroll_response = ScrollArea::vertical()
                    .id_salt("comments_scroll_area")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(self.comments_scroll_offset)
                    .show(ui, |ui| {
                        // Just render all comments directly without height estimates or viewport checks
                        // This eliminates scroll position jumps when nearing the bottom
                        for comment in &page_comments {
                            self.render_comment(ui, comment, 0);
                        }
                        
                        // Add some padding at the bottom for better UI
                        ui.add_space(40.0);
                    });
                    
                // Store the actual scroll position after the user might have scrolled manually
                let scroll_offset = scroll_response.state.offset.y;
                self.comments_scroll_offset = scroll_offset;
                
                
                // Pagination controls at the bottom (duplicated for convenience)
                ui.add_space(8.0);
                self.render_pagination_controls(ui);
            } else {
                // Stories table with scrolling
                ui.add_space(4.0);
                
                // Show the current tab name
                let tab_name = match self.current_tab {
                    Tab::Hot => "Hot Stories",
                    Tab::New => "New Stories",
                    Tab::Show => "Show HN",
                    Tab::Ask => "Ask HN",
                    Tab::Jobs => "Jobs",
                    Tab::Best => "Best Stories",
                };
                
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new(tab_name)
                            .size(18.0)
                            .color(self.theme.text)
                    );
                    
                    // Display keyboard navigation hint
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Keyboard: Arrows to scroll, Space for Page Down, Backspace to go back, Ctrl+O to open article, Ctrl+L to copy link")
                                .size(13.0)
                                .color(self.theme.secondary_text)
                                .italics()
                        );
                    });
                });
                
                ui.add_space(8.0);
                
                // Stories section with scrolling - use ID for persistent state
                let scroll_response = ScrollArea::vertical()
                    .id_salt("stories_scroll_area") // Using id_salt instead of id_source
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(self.stories_scroll_offset)
                    .show(ui, |ui| {
                        self.render_stories_table(ui);
                        
                        // Show loading indicator at the bottom if loading more stories
                        if self.loading_more_stories {
                            ui.add_space(10.0);
                            ui.vertical_centered(|ui| {
                                ui.spinner();
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Loading more stories...")
                                        .color(self.theme.secondary_text)
                                        .size(14.0)
                                );
                            });
                        } else if self.end_of_stories {
                            // Show message when we've reached the end
                            ui.add_space(10.0);
                            ui.vertical_centered(|ui| {
                                // Determine if we reached the end due to max pages or no more content
                                let message = if self.current_page >= 5 {
                                    format!("Showing maximum of {} stories. Scroll up to view.", self.stories.len())
                                } else {
                                    "End of stories.".to_string()
                                };
                                
                                ui.label(
                                    RichText::new(message)
                                        .color(self.theme.secondary_text)
                                        .size(14.0)
                                );
                            });
                        }
                        
                        // Allow extra space for scrolling at the bottom
                        ui.add_space(20.0);
                    });
                    
                // Store the actual scroll position after the user might have scrolled manually
                let scroll_offset = scroll_response.state.offset.y;
                self.stories_scroll_offset = scroll_offset;

                // Detect when we're at the bottom and should load more stories
                // Calculate an approximate threshold based on the current stories and UI layout
                let stories_count = self.stories.len();
                
                // Get the viewport height first
                let viewport_height = scroll_response.inner_rect.height();
                
                // Based on the debug info, we need to adjust our story height calculation
                // Looking at your scroll values, it seems the stories might be taller than we thought
                let average_story_height = 140.0; // Adjusted down based on your debug output
                let header_height = 60.0;
                let footer_height = 60.0;  
                
                // Calculate a more accurate estimate of the content height
                let estimated_content_height = 
                    if stories_count == 0 {
                        // Avoid division by zero
                        viewport_height + 100.0
                    } else {
                        // The calculation below is based on:
                        // Total height = Header + (Stories * Height per story) + Footer
                        header_height + (stories_count as f32 * average_story_height) + footer_height
                    };
                
                // IMPORTANT: Your debug output shows your offset is consistently near 2049.5
                // which suggests we might be hitting a limit in the scroll behavior.
                // Let's adjust our content calculation based on this observation:
                
                // Calculate distance to bottom for debugging
                let distance_to_bottom = estimated_content_height - scroll_offset - viewport_height;
                let _scroll_percentage = if estimated_content_height > viewport_height {
                    scroll_offset / (estimated_content_height - viewport_height)
                } else {
                    1.0
                };
                
                // Calculate max possible scroll position (content height minus viewport height)
                let max_scroll = (estimated_content_height - viewport_height).max(0.0);
                
                // Calculate how close we are to the bottom as a percentage (0% = top, 100% = bottom)
                // This is more intuitive than the previous percentage calculation
                let bottom_proximity_pct = if max_scroll > 0.0 {
                    (scroll_offset / max_scroll) * 100.0
                } else {
                    100.0 // If content fits in viewport, we're at the bottom
                };
                
                // Based on your debug output, we need a completely different approach:
                // Your debug shows your maximum scroll appears to be around 2049.5 consistently
                // This suggests there may be some scroll limit in the eGUI framework
                
                // Calculate where we think the bottom is
                let _visible_bottom = scroll_offset + viewport_height;
                
                // Instead of comparing with estimated content height, use a set of better indicators:
                // 1. User's specific situation - your debug shows ~2049.5 is max scroll
                // 2. If offset is very close to max_scroll (within 5% or 100px)
                // 3. If we have a reasonable number of stories and are past a specific scroll threshold
                let at_bottom = 
                    // Your specific case - around 2049.5 seems to be max scroll based on debug output
                    (scroll_offset > 2000.0) ||
                    
                    // General cases that should work in most situations
                    (max_scroll > 0.0 && scroll_offset > (max_scroll * 0.95)) ||
                    (max_scroll - scroll_offset < 100.0) ||
                    
                    // If we have more than 20 stories and scrolled significantly
                    (self.stories.len() > 20 && scroll_offset > 1500.0);
                
                // Print scroll debug info every time to diagnose issues
                // Debug output turned off
                // println!("Scroll debug: offset={:.1}, viewport={:.1}, content={:.1}, visible_bottom={:.1}, max_scroll={:.1}, distance_to_bottom={:.1}, bottom_proximity={:.1}%, at_bottom={}, loading={}, more={}, end={}", 
                //     scroll_offset, viewport_height, estimated_content_height, 
                //     _visible_bottom, max_scroll, distance_to_bottom, 
                //     bottom_proximity_pct, at_bottom,
                //     self.loading, self.loading_more_stories, self.end_of_stories);
                
                // Make the loading trigger less aggressive to avoid loading too early
                if !self.loading && !self.loading_more_stories && !self.end_of_stories {
                    // We don't want to load more than once per "session" of scrolling,
                    // so we'll track if we're close enough to trigger loading soon
                    
                    // We want to only trigger when actually at the bottom, not during normal scrolling
                    let should_load = 
                        // Only trigger when we're REALLY at the bottom
                        at_bottom ||                       // At bottom detection
                        
                        // Specific case based on your debug values, but with higher threshold
                        // to prevent triggering too early
                        (scroll_offset > 2030.0) ||        // Only when VERY close to max scroll
                        
                        // Only when we're 85% scrolled down (much less aggressive)
                        (bottom_proximity_pct > 85.0) ||
                        
                        // Very close to bottom in pixels (much less aggressive)
                        (distance_to_bottom < 300.0);
                    
                    if should_load {
                        #[allow(dead_code)]
                        const MAX_PAGES: usize = 5; // Keep in sync with the limit in load_more_stories
                        
                        // Debug output turned off
                        // println!("==========================================");
                        // println!("AUTO-LOADING MORE STORIES - Page {} -> {} (max: {})", 
                        //          self.current_page, self.current_page + 1, MAX_PAGES);
                        // println!("SCROLL STATS:");
                        // println!("  At bottom: {}", at_bottom);
                        // println!("  Bottom proximity: {:.1}%", bottom_proximity_pct);
                        // println!("  Distance to bottom: {:.1}px", distance_to_bottom);
                        // println!("  Offset: {:.1}/{:.1} ({}%)", scroll_offset, max_scroll, 
                        //          if max_scroll > 0.0 { (scroll_offset/max_scroll) * 100.0 } else { 100.0 });
                        // println!("  Story count: {}/{} ({}%)", 
                        //          self.stories.len(), MAX_PAGES * 30,
                        //          (self.stories.len() as f32 / (MAX_PAGES * 30) as f32) * 100.0);
                        // println!("==========================================");
                        self.load_more_stories();
                    }
                }
            }
        });
        
        // Show the share modal dialog if it's enabled
        if self.show_share_modal {
            // Use a modal overlay
            let mut modal_open = true;
            
            // Get screen dimensions for proper positioning
            let screen_rect = ctx.screen_rect();
            let modal_width = 300.0;
            let modal_height = 250.0;
            
            // Center the modal dialog
            let modal_pos = egui::pos2(
                screen_rect.center().x - modal_width / 2.0,
                screen_rect.center().y - modal_height / 2.0
            );
            
            // Clone story details to avoid borrow checker issues
            let story_title = self.selected_story.as_ref().map(|s| s.title.clone()).unwrap_or_default();
            let story_id = self.selected_story.as_ref().map(|s| s.id.clone()).unwrap_or_default();
            let button_foreground = self.theme.button_foreground;
            let button_background = self.theme.button_background;
            let is_link_copied = self.share_link_copied;
            
            // Create the modal window
            egui::Window::new("Share")
                .id(egui::Id::new("share_modal"))
                .title_bar(true)
                .resizable(false)
                .collapsible(false)
                .fixed_pos(modal_pos)
                .fixed_size([modal_width, modal_height])
                .open(&mut modal_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        
                        // X (formerly Twitter) share button
                        if ui.add_sized([260.0, 40.0], egui::Button::new(
                            RichText::new("Share on X")
                                .size(16.0)
                                .color(Color32::WHITE)
                        ).fill(Color32::from_rgb(0, 0, 0))).clicked() {  // X uses black as brand color
                            // Create X share URL (still uses twitter.com domain)
                            let twitter_url = format!(
                                "https://twitter.com/intent/tweet?text={}&url={}",
                                urlencoding::encode(&story_title),
                                urlencoding::encode(&format!("https://news.ycombinator.com/item?id={}", story_id))
                            );
                            // Use pointer cast to get mutable access
                            let this = self as *const _ as *mut Self;
                            unsafe {
                                (*this).open_link(&twitter_url);
                                (*this).show_share_modal = false;
                            }
                        }
                        
                        ui.add_space(10.0);
                        
                        // Facebook share button
                        if ui.add_sized([260.0, 40.0], egui::Button::new(
                            RichText::new("Share on Facebook")
                                .size(16.0)
                                .color(Color32::WHITE)
                        ).fill(Color32::from_rgb(66, 103, 178))).clicked() {
                            // Create Facebook share URL
                            let facebook_url = format!(
                                "https://www.facebook.com/sharer/sharer.php?u={}",
                                urlencoding::encode(&format!("https://news.ycombinator.com/item?id={}", story_id))
                            );
                            // Use pointer cast to get mutable access
                            let this = self as *const _ as *mut Self;
                            unsafe {
                                (*this).open_link(&facebook_url);
                                (*this).show_share_modal = false;
                            }
                        }
                        
                        ui.add_space(10.0);
                        
                        // Copy link button
                        let copy_btn_text = if is_link_copied {
                            "Link Copied!"
                        } else {
                            "Copy Link to Clipboard"
                        };
                        
                        if ui.add_sized([260.0, 40.0], egui::Button::new(
                            RichText::new(copy_btn_text)
                                .size(16.0)
                                .color(button_foreground)
                        ).fill(button_background)).clicked() {
                            // Generate the HN link
                            let hn_link = format!("https://news.ycombinator.com/item?id={}", story_id);
                            
                            // Copy to clipboard using clipboard crate
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                if clipboard.set_text(hn_link).is_ok() {
                                    // Use pointer cast to get mutable access
                                    let this = self as *const _ as *mut Self;
                                    unsafe {
                                        (*this).share_link_copied = true;
                                    }
                                }
                            }
                        }
                        
                        ui.add_space(10.0);
                        
                        // Close button
                        if ui.add_sized([260.0, 30.0], egui::Button::new(
                            RichText::new("Close")
                                .size(14.0)
                        )).clicked() {
                            // Use pointer cast to get mutable access
                            let this = self as *const _ as *mut Self;
                            unsafe {
                                (*this).show_share_modal = false;
                                (*this).share_link_copied = false;
                            }
                        }
                    });
                });
            
            // Close the modal if the open flag was changed
            if !modal_open {
                self.show_share_modal = false;
                self.share_link_copied = false;
            }
        }
    }
}

impl HackerNewsReaderApp {
    // Process comment collapse/expand buttons
    fn process_comment_collapse_buttons(&mut self, ctx: &egui::Context) {
        if self.comments.is_empty() {
            return; // No comments to process
        }
        
        // Clone the comments to avoid borrow conflicts
        let comments = self.comments.clone();
        
        // Recursively check for collapse button clicks on all comments
        self.check_comment_buttons_recursive(ctx, &comments);
    }
    
    // Load stories with option to force refresh (bypass cache)
    fn load_stories_with_refresh(&mut self, force_refresh: bool) {
        if self.loading {
            return; // Don't start another load if we're already loading
        }
        
        // Reset search state when loading fresh stories
        if self.show_search_ui {
            self.toggle_search_ui();
        } else {
            self.reset_all_filters();
        }
        
        self.loading = true;
        self.current_page = 1; // Reset to page 1 when loading fresh stories
        self.end_of_stories = false; // Reset end of stories flag
        self.selected_story_index = None; // Reset the selected story index
        
        // Create a new thread for loading
        let client = self.hn_client.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Convert the tab enum to a string
        let tab_str = match self.current_tab {
            Tab::Hot => "hot",
            Tab::New => "new",
            Tab::Show => "show",
            Tab::Ask => "ask",
            Tab::Jobs => "jobs",
            Tab::Best => "best",
        };
        
        let handle = thread::spawn(move || {
            let result: Box<dyn std::any::Any + Send> = if force_refresh {
                // If force refresh, bypass cache
                match client.fetch_fresh_stories_by_tab(tab_str) {
                    Ok(stories) => {
                        let _ = tx.send(Some(stories));
                        Box::new(())
                    }
                    Err(_) => {
                        let _ = tx.send(None::<Vec<HackerNewsItem>>);
                        Box::new(())
                    }
                }
            } else {
                // Otherwise use cached data if available
                match client.fetch_stories_by_tab(tab_str) {
                    Ok(stories) => {
                        let _ = tx.send(Some(stories));
                        Box::new(())
                    }
                    Err(_) => {
                        let _ = tx.send(None::<Vec<HackerNewsItem>>);
                        Box::new(())
                    }
                }
            };
            result
        });
        
        self.load_thread = Some(handle);
        self.stories_receiver = Some(rx);
        self.needs_repaint = true;
    }

    fn refresh_current_view(&mut self, force_refresh: bool) {
        if self.loading {
            return; // Don't start another load if we're already loading
        }
        
        if let Some(ref selected_story) = self.selected_story {
            // We're in comments view - refresh the comments for this story
            self.view_comments(selected_story.clone(), force_refresh);
        } else {
            // We're in stories view - refresh the current tab with force refresh
            self.load_stories_with_refresh(force_refresh);
        }
    }
    
    // Process keyboard shortcuts
    fn process_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Get keyboard input
        let input = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Space),        // Space - Scroll down / collapse comment
                i.key_pressed(egui::Key::C),            // C - Collapse/expand all comments
                i.modifiers.shift,                      // Modifier for various actions
                i.key_pressed(egui::Key::ArrowLeft),    // Left - Previous page / scroll left
                i.key_pressed(egui::Key::ArrowRight),   // Right - Next page / scroll right
                i.key_pressed(egui::Key::ArrowUp),      // Up - Scroll up
                i.key_pressed(egui::Key::ArrowDown),    // Down - Scroll down
                i.key_pressed(egui::Key::Home),         // Home - First page / top of content
                i.key_pressed(egui::Key::End),          // End - Last page / bottom of content
                i.key_pressed(egui::Key::PageUp),       // Page Up - Scroll up a page
                i.key_pressed(egui::Key::PageDown),     // Page Down - Scroll down a page
                i.key_pressed(egui::Key::Backspace),    // Backspace - Go back to stories view
                i.key_pressed(egui::Key::Escape),       // Escape - Close search UI
                i.key_pressed(egui::Key::F),            // F - Show/hide search UI (with Control)
                i.modifiers.ctrl,                       // Control modifier for various actions
                i.key_pressed(egui::Key::Num1),         // Number keys for tab switching
                i.key_pressed(egui::Key::Num2),
                i.key_pressed(egui::Key::Num3),
                i.key_pressed(egui::Key::Num4),
                i.key_pressed(egui::Key::Num5),
                i.key_pressed(egui::Key::Num6),
                i.key_pressed(egui::Key::Plus),         // Plus key - Increase font size
                i.key_pressed(egui::Key::Minus),        // Minus key - Decrease font size
                i.key_pressed(egui::Key::R),            // R key - For Ctrl+R refresh shortcut
                i.key_pressed(egui::Key::Enter),        // Enter key - Open selected story
                i.key_pressed(egui::Key::S),            // S key - For Ctrl+S side panel toggle
                i.key_pressed(egui::Key::T),            // T key - Mark selected story as Todo
                i.key_pressed(egui::Key::D),            // D key - Mark selected story as Done
                i.key_pressed(egui::Key::O),            // O key - For Ctrl+O to open article in browser
                i.key_pressed(egui::Key::L),            // L key - For Ctrl+L to copy article link
            )
        });
        
        // Handle Ctrl+R for refresh (highest priority) - this should work in any view
        if input.14 && input.23 && !self.loading {  // Ctrl + R and not already loading
            self.refresh_current_view(true);  // Force refresh (bypass cache)
            return;
        }
        
        // Handle Ctrl+S to toggle side panel (high priority) - this should work in any view
        if input.14 && input.25 {  // Ctrl + S
            self.toggle_favorites_panel();
            self.needs_repaint = true;
            return;
        }
        
        // Handle Ctrl+L to copy article link (high priority) - this should work in comments view
        if input.14 && input.29 {  // Ctrl + L
            if let Some(ref story) = self.selected_story {
                // Generate the HN link
                let hn_link = format!("https://news.ycombinator.com/item?id={}", story.id);
                
                // Copy to clipboard
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if clipboard.set_text(hn_link).is_ok() {
                        // Show confirmation message
                        self.set_status_message("Article link copied to clipboard".to_string());
                        self.share_link_copied = true;
                    } else {
                        self.set_status_message("Failed to copy link to clipboard".to_string());
                    }
                } else {
                    self.set_status_message("Clipboard access error".to_string());
                }
                self.needs_repaint = true;
                return;
            }
        }
        
        // Handle search UI keyboard shortcuts (high priority)
        // Ctrl+F to show search UI
        if input.14 && input.13 && !self.show_search_ui {  // Ctrl + F
            self.toggle_search_ui();
            self.needs_repaint = true;
            return;
        }
        
        // ESC to close search UI
        if input.12 && self.show_search_ui {
            self.toggle_search_ui();
            self.needs_repaint = true;
            return;
        }
        
        // Don't process number key shortcuts if we have input focus in search
        let has_text_focus = ctx.memory(|m| m.has_focus(egui::Id::new("search_input")));
        
        // Handle story navigation with arrow keys in the stories view
        if !has_text_focus && self.selected_story.is_none() {
            // Get the list of stories to navigate
            let stories_to_display = if (!self.search_query.is_empty() || self.show_todo_only || self.show_done_only) && !self.filtered_stories.is_empty() {
                &self.filtered_stories
            } else {
                &self.stories
            };
            
            // Only process if we have stories
            if !stories_to_display.is_empty() {
                // Constants for story card height approximation
                const APPROX_STORY_HEIGHT: f32 = 85.0; // Approximate height of a story card in pixels
                const APPROX_STORY_MARGIN: f32 = 7.0;  // Approximate margin between stories
                #[allow(dead_code)]
                const VERTICAL_OFFSET_BUFFER: f32 = 100.0; // Additional buffer to ensure visibility
                
                // Helper function to calculate the scroll position to center the story in the viewport
                let center_story_in_viewport = |idx: usize| {
                    let story_position = (idx as f32 - 1.0) * (APPROX_STORY_HEIGHT + APPROX_STORY_MARGIN);
                    let viewport_height = ctx.available_rect().height();
                    let center_position = story_position - (viewport_height / 2.0) + (APPROX_STORY_HEIGHT / 2.0);
                    center_position.max(0.0)
                };
                
                // Down arrow to select the next story
                if input.6 {  // ArrowDown
                    match self.selected_story_index {
                        Some(idx) if idx + 1 < stories_to_display.len() => {
                            // Move to next story
                            self.selected_story_index = Some(idx + 1);
                            
                            // Center the next story in the viewport
                            self.stories_scroll_offset = center_story_in_viewport(idx + 1);
                        }
                        None => {
                            // Select the first story if none is selected
                            self.selected_story_index = Some(0);
                            // Center the first story in the viewport
                            // self.stories_scroll_offset = center_story_in_viewport(0);
                        }
                        _ => {}  // At the last story, do nothing
                    }
                    self.needs_repaint = true;
                    return;
                }
                
                // Up arrow to select the previous story
                else if input.5 {  // ArrowUp
                    if let Some(idx) = self.selected_story_index {
                        if idx > 0 {
                            // Move to previous story
                            self.selected_story_index = Some(idx - 1);
                            
                            // Center the previous story in the viewport
                            self.stories_scroll_offset = center_story_in_viewport(idx - 1);
                        }
                    } else if !stories_to_display.is_empty() {
                        // Select the last story if none is selected
                        let last_idx = stories_to_display.len() - 1;
                        self.selected_story_index = Some(last_idx);
                        
                        // Center the last story in the viewport
                        // self.stories_scroll_offset = center_story_in_viewport(last_idx);
                    }
                    self.needs_repaint = true;
                    return;
                }
                
                // Enter to view the selected story
                else if input.24 {  // Enter key - now at index 24
                    if let Some(idx) = self.selected_story_index {
                        if idx < stories_to_display.len() {
                            // Open the comments for the selected story
                            let story = stories_to_display[idx].clone();
                            self.view_comments(story, false);
                            return;
                        }
                    }
                }
                
                // T key to mark selected story as Todo
                else if input.26 { // T key - now at index 26
                    if let Some(idx) = self.selected_story_index {
                        if idx < stories_to_display.len() {
                            let story = stories_to_display[idx].clone();
                            self.add_to_todo(&story);
                            self.set_status_message(format!("Added '{}' to your todo list", story.title));
                            self.needs_repaint = true;
                            return;
                        }
                    }
                }
                
                // D key to mark selected story as Done
                else if input.27 { // D key - now at index 27
                    if let Some(idx) = self.selected_story_index {
                        if idx < stories_to_display.len() {
                            let story = stories_to_display[idx].clone();
                            let was_done = self.is_done(&story.id);
                            self.toggle_done(&story);
                            
                            if was_done {
                                self.set_status_message(format!("Marked '{}' as not done", story.title));
                            } else {
                                self.set_status_message(format!("Marked '{}' as done", story.title));
                            }
                            
                            self.needs_repaint = true;
                            return;
                        }
                    }
                }
            }
            
            // Handle tab switching with number keys (1-6)
            if input.15 {
                self.switch_tab(Tab::Hot);
                return;
            } else if input.16 {
                self.switch_tab(Tab::New);
                return;
            } else if input.17 {
                self.switch_tab(Tab::Show);
                return;
            } else if input.18 {
                self.switch_tab(Tab::Ask);
                return;
            } else if input.19 {
                self.switch_tab(Tab::Jobs);
                return;
            } else if input.20 {
                self.switch_tab(Tab::Best);
                return;
            }
        }
        
        // Ctrl+O to open article in browser - works in both story list and comments view
        if input.14 && input.28 { // Ctrl + O
            if let Some(ref selected_story) = self.selected_story {
                // In comments view
                if !selected_story.url.is_empty() {
                    self.open_link(&selected_story.url);
                    self.set_status_message(format!("Opening article in browser: {}", selected_story.title));
                } else {
                    // If no URL is available (self-posts like Ask HN), show message
                    self.set_status_message("No external URL available for this story".to_string());
                }
                self.needs_repaint = true;
                return;
            } else if let Some(idx) = self.selected_story_index {
                // In story list view with story selected via keyboard
                let stories_to_use = if (!self.search_query.is_empty() || self.show_todo_only || self.show_done_only) && !self.filtered_stories.is_empty() {
                    &self.filtered_stories
                } else {
                    &self.stories
                };
                
                if idx < stories_to_use.len() {
                    let story = &stories_to_use[idx];
                    if !story.url.is_empty() {
                        self.open_link(&story.url);
                        self.set_status_message(format!("Opening article in browser: {}", story.title));
                    } else {
                        self.set_status_message("No external URL available for this story".to_string());
                    }
                    self.needs_repaint = true;
                    return;
                }
            }
        }
        
        // Handle font size adjustment in comments view
        if let Some(_) = self.selected_story {
            
            // Plus key to increase font size
            if input.21 {
                self.increase_comment_font_size();
                return;
            }
            
            // Minus key to decrease font size
            if input.22 {
                self.decrease_comment_font_size();
                return;
            }
            
            // Continue with other comment view shortcuts
            // Check for backspace key to return to story list (highest priority)
            if input.11 { // Backspace key
                self.selected_story = None;
                self.comments.clear();
                self.comments_scroll_offset = 0.0;
                self.needs_repaint = true;
                return; // Don't process other keys after navigation
            }
            
            // Comment view shortcuts
            if !self.comments.is_empty() {
                // C - Toggle all comments based on shift key
                if input.1 {
                    if input.2 { // Shift+C
                        // Expand all comments
                        self.collapsed_comments.clear();
                    } else {
                        // Collapse all top-level comments
                        self.collapse_all_top_level_comments();
                    }
                    self.needs_repaint = true;
                    return; // Don't process other keys after this action
                }
                
                // Page navigation with keyboard for comments pagination
                let (current_page, total_pages, _) = self.get_pagination_info();
                
                // Left arrow - Previous page
                if input.3 && current_page > 0 {
                    self.comments_page = current_page - 1;
                    self.comments_scroll_offset = 0.0; // Reset scroll position on page change
                    self.needs_repaint = true;
                    return;
                }
                
                // Right arrow - Next page
                if input.4 && current_page < total_pages - 1 {
                    self.comments_page = current_page + 1;
                    self.comments_scroll_offset = 0.0; // Reset scroll position on page change
                    self.needs_repaint = true;
                    return;
                }
                
                // Home key - First page
                if input.7 && current_page > 0 {
                    self.comments_page = 0;
                    self.comments_scroll_offset = 0.0; // Reset scroll position on page change
                    self.needs_repaint = true;
                    return;
                }
                
                // End key - Last page
                if input.8 && current_page < total_pages - 1 {
                    self.comments_page = total_pages - 1;
                    self.comments_scroll_offset = 0.0; // Reset scroll position on page change
                    self.needs_repaint = true;
                    return;
                }
            }
            
            // Scroll controls for comments
            const SCROLL_AMOUNT: f32 = 30.0;
            const SCROLL_PAGE_AMOUNT: f32 = 500.0; // Larger value for more of a "page" feel
            
            // Space or PageDown - Scroll down by a page
            if input.0 || input.10 {
                self.comments_scroll_offset += SCROLL_PAGE_AMOUNT; // Both space and PageDown scroll a full page
                self.needs_repaint = true;
            }
            
            // PageUp - Scroll up a page
            if input.9 {
                self.comments_scroll_offset -= SCROLL_PAGE_AMOUNT;
                if self.comments_scroll_offset < 0.0 {
                    self.comments_scroll_offset = 0.0;
                }
                self.needs_repaint = true;
            }
            
            // Arrow Up - Scroll up
            if input.5 {
                self.comments_scroll_offset -= SCROLL_AMOUNT;
                if self.comments_scroll_offset < 0.0 {
                    self.comments_scroll_offset = 0.0;
                }
                self.needs_repaint = true;
            }
            
            // Arrow Down - Scroll down
            if input.6 {
                self.comments_scroll_offset += SCROLL_AMOUNT;
                self.needs_repaint = true;
            }
            
            // Home - Scroll to top
            if input.7 && !input.2 { // Home without Shift (Shift+Home is for pagination)
                self.comments_scroll_offset = 0.0;
                self.needs_repaint = true;
            }
            
            // End - Scroll to bottom (approximated)
            if input.8 && !input.2 { // End without Shift (Shift+End is for pagination)
                self.comments_scroll_offset = 10000.0; // A large value to scroll to bottom
                self.needs_repaint = true;
            }
        } else {
            // Stories view shortcuts
            #[allow(dead_code)]
            const SCROLL_AMOUNT: f32 = 30.0;
            const SCROLL_PAGE_AMOUNT: f32 = 500.0; // Larger value for more of a "page" feel
            
            // Space or PageDown - Scroll down by a page
            if input.0 || input.10 {
                self.stories_scroll_offset += SCROLL_PAGE_AMOUNT; // Both space and PageDown scroll a full page
                self.needs_repaint = true;
            }
            
            // PageUp - Scroll up a page
            if input.9 {
                self.stories_scroll_offset -= SCROLL_PAGE_AMOUNT;
                if self.stories_scroll_offset < 0.0 {
                    self.stories_scroll_offset = 0.0;
                }
                self.needs_repaint = true;
            }
            
            // We're not using arrow keys for scrolling in the stories view anymore.
            // Arrow key navigation is implemented in the story selection code above.
            // This prevents arrow keys from causing both selection and scrolling.
            
            // Home - Scroll to top
            if input.7 {
                self.stories_scroll_offset = 0.0;
                self.needs_repaint = true;
            }
            
            // End - Scroll to bottom (approximated)
            if input.8 {
                self.stories_scroll_offset = 10000.0; // A large value to scroll to bottom
                self.needs_repaint = true;
            }
        }
    }
    
    // Helper function to collapse all top-level comments
    fn collapse_all_top_level_comments(&mut self) {
        for comment in &self.comments {
            self.collapsed_comments.insert(comment.id.clone());
        }
    }
    
    // Helper function to get pagination information
    fn get_pagination_info(&self) -> (usize, usize, usize) {
        let total_pages = if self.comments.is_empty() {
            1
        } else {
            (self.comments.len() + self.comments_per_page - 1) / self.comments_per_page
        };
        
        let current_page = self.comments_page.min(total_pages - 1);
        let total_comments = self.comments.len();
        
        (current_page, total_pages, total_comments)
    }
    
    // Helper function to get comments for the current page
    fn get_current_page_comments(&self) -> Vec<&HackerNewsComment> {
        let start_idx = self.comments_page * self.comments_per_page;
        let end_idx = (start_idx + self.comments_per_page).min(self.comments.len());
        
        self.comments[start_idx..end_idx].iter().collect()
    }
    
    fn check_comment_buttons_recursive(&mut self, ctx: &egui::Context, comments: &[HackerNewsComment]) {
        // Process all comments including those not on the current page
        // This ensures collapsing/expanding works even for non-visible comments
        
        for comment in comments {
            // Skip processing empty comments
            if comment.text.is_empty() || comment.text == "[deleted]" {
                continue;
            }
            
            // Check if the collapse button for this comment was clicked
            let collapse_btn_id = egui::Id::new("comment_collapse_btn").with(comment.id.clone());
            if let Some(btn_response) = ctx.data_mut(|d| d.get_temp::<egui::Response>(collapse_btn_id)) {
                if btn_response.clicked() {
                    // Toggle collapse state for this comment
                    if self.collapsed_comments.contains(&comment.id) {
                        self.collapsed_comments.remove(&comment.id);
                    } else {
                        self.collapsed_comments.insert(comment.id.clone());
                    }
                    self.needs_repaint = true;
                }
            }
            
            // Check child comments recursively (if not collapsed)
            if !comment.children.is_empty() && !self.collapsed_comments.contains(&comment.id) {
                self.check_comment_buttons_recursive(ctx, &comment.children);
            }
        }
    }
    
    fn render_stories_table(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone(); // Get context from UI
        let mut story_to_view = None;
        
        // Use filtered stories if there's a search query or active filters, otherwise use all stories
        let stories_to_display = if (!self.search_query.is_empty() || self.show_todo_only || self.show_done_only) && !self.filtered_stories.is_empty() {
            self.filtered_stories.clone()
        } else {
            self.stories.clone()
        };
        
        // If filters are active but no results found, show a message
        if (!self.search_query.is_empty() || self.show_todo_only || self.show_done_only) && self.filtered_stories.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Build appropriate message based on active filters
                let mut message = String::new();
                
                if !self.search_query.is_empty() {
                    message.push_str(&format!("No results found for search '{}' ", self.search_query));
                }
                
                if self.show_todo_only {
                    if !message.is_empty() {
                        message.push_str("with ");
                    } else {
                        message.push_str("No results found with ");
                    }
                    message.push_str("TODO filter ");
                }
                
                if self.show_done_only {
                    if !message.is_empty() {
                        message.push_str("with ");
                    } else {
                        message.push_str("No results found with ");
                    }
                    message.push_str("DONE filter ");
                }
                
                ui.label(
                    RichText::new(message)
                        .color(self.theme.secondary_text)
                        .size(18.0)
                        .italics()
                );
                
                ui.add_space(8.0);
                
                // Build appropriate suggestion
                let try_again_text = if !self.search_query.is_empty() {
                    "Try different keywords or disable filters."
                } else {
                    "Try disabling filters or add some favorites first."
                };
                
                ui.label(
                    RichText::new(try_again_text)
                        .color(self.theme.secondary_text)
                        .size(16.0)
                );
                ui.add_space(20.0);
            });
            return;
        }
        
        // Add keyboard shortcuts info at the top
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Keyboard shortcuts: T for Todo, D for Done, Ctrl+O to open article")
                        .color(self.theme.secondary_text)
                        .size(13.0)
                        .italics()
                );
            });
        });
        ui.add_space(8.0);
        
        // Calculate proper starting rank for display (always start from 1)
        let mut current_rank = 1;
        
        for (i, story) in stories_to_display.iter().enumerate() {
            // Check if this story is the selected one for keyboard navigation
            let is_selected = self.selected_story_index == Some(i);
            
            // Get card background based on score using our helper method
            let mut card_background = self.theme.get_card_background(story.score);
            
            // Get the appropriate border stroke based on score
            let mut card_stroke = self.theme.get_card_stroke(story.score);
            
            // Override with selection highlighting if this is the selected story
            if is_selected {
                // Use a more prominent background and border for the selected story
                if self.is_dark_mode {
                    // In dark mode, use a slightly brighter background
                    card_background = Color32::from_rgba_premultiplied(
                        card_background.r().saturating_add(15),
                        card_background.g().saturating_add(15),
                        card_background.b().saturating_add(15),
                        255
                    );
                } else {
                    // In light mode, use a slightly darker background
                    card_background = Color32::from_rgba_premultiplied(
                        card_background.r().saturating_sub(10),
                        card_background.g().saturating_sub(10),
                        card_background.b().saturating_sub(10),
                        255
                    );
                }
                
                // Use a thicker, more visible border for the selected item
                card_stroke = Stroke::new(2.0, self.theme.accent);
            }
            
            // Create a card for each story with background and border based on score
            let card_response = egui::Frame::new()
                .fill(card_background)
                .corner_radius(CornerRadius::same(8))
                .stroke(card_stroke)
                .inner_margin(12.0)
                .outer_margin(egui::vec2(8.0, 6.0))
                .show(ui, |ui| {
                    // Top row with rank, title, and score
                    ui.horizontal(|ui| {
                        // Use the current_rank which always increments correctly
                        let rank = current_rank;
                        
                        // Increment for next story
                        current_rank += 1;
                        
                        // Cap at maximum number of stories (150)
                        if current_rank > 150 {
                            current_rank = 150;
                        }
                        ui.label(
                            RichText::new(format!("{}", rank))
                                .color(self.theme.secondary_text)
                                .size(16.0)
                        );
                        ui.add_space(8.0);
                        
                        // Story title with clickable behavior and color highlighting based on score
                        let score_color = self.theme.get_title_color(story.score);
                        // Use a different color for viewed stories
                        let color = if self.is_story_viewed(&story.id) {
                            // Use grayish color for viewed stories
                            self.theme.get_viewed_story_color()
                        } else {
                            // Use normal color based on score
                            score_color
                        };
                        
                        let title_label = ui.add(
                            egui::Label::new(
                                RichText::new(&story.title)
                                    .color(color)
                                    .size(16.0)
                                    .strong()
                            ).sense(egui::Sense::click())
                        );
                        
                        if title_label.clicked() && !story.url.is_empty() {
                            self.open_link(&story.url);
                        }
                        
                        if title_label.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                        
                        // Add domain if available
                        if !story.domain.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!("({})", story.domain))
                                    .color(self.theme.secondary_text)
                                    .italics()
                            );
                        }
                        
                        // Score on the right side with color based on value
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let score_color = self.theme.score_color(story.score);
                            ui.label(
                                RichText::new(format!("{} pts", story.score))
                                    .color(score_color)
                                    .strong()
                            );
                        });
                    });
                    
                    // Bottom row with metadata and actions
                    ui.horizontal(|ui| {
                        // Metadata
                        ui.label(
                            RichText::new("by")
                                .color(self.theme.secondary_text)
                                .size(14.0)
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&story.by)
                                .color(self.theme.text)
                                .size(14.0)
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(&story.time_ago)
                                .color(self.theme.secondary_text)
                                .size(14.0)
                        );
                        
                        // Actions on the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Comments button
                            let comments_btn = ui.add_sized(
                                [110.0, 28.0],
                                egui::Button::new(
                                    RichText::new(format!("{} Comments", story.comments_count))
                                        .size(14.0)
                                        .color(self.theme.button_foreground)
                                )
                                .corner_radius(CornerRadius::same(6))
                                .fill(self.theme.button_background)
                            );
                            
                            if comments_btn.clicked() {
                                story_to_view = Some(story.clone());
                            }
                            
                            // Add tooltip for comment button with improved stability
                            if comments_btn.hovered() {
                                // We don't need this anymore since we always force refresh
                            // let force_refresh = ctx.input(|i| i.modifiers.shift);
                                
                                // Fixed position tooltip to avoid flickering
                                let tooltip_pos = comments_btn.rect.left_top() + egui::vec2(0.0, -40.0);
                                
                                // Use the story ID to make the tooltip unique per story
                                egui::Area::new(egui::Id::new("comments_tooltip_area").with(story.id.clone()))
                                    .order(egui::Order::Tooltip)
                                    .fixed_pos(tooltip_pos)
                                    .show(&ctx, |ui| {
                                        egui::Frame::popup(ui.style())
                                            .fill(self.theme.card_background)
                                            .stroke(Stroke::new(1.0, self.theme.separator))
                                            .corner_radius(CornerRadius::same(6))
                                            .show(ui, |ui| {
                                                ui.add(egui::Label::new(RichText::new("View Comments")));
                                                ui.add(egui::Label::new(
                                                    RichText::new("Click to view thread")
                                                        .size(12.0)
                                                        .color(self.theme.secondary_text)
                                                ));
                                            });
                                    });
                            }
                            
                            // Favorite button
                            ui.add_space(8.0);
                            
                            // Get favorite status
                            let is_favorite = self.is_favorite(&story.id);
                            let favorite_color = if is_favorite {
                                Color32::from_rgb(255, 204, 0) // Gold star color for favorited
                            } else {
                                self.theme.secondary_text // Gray star for not favorited
                            };
                            
                            let favorite_btn = ui.add_sized(
                                [40.0, 28.0],
                                egui::Button::new(
                                    RichText::new("★") // Star symbol
                                        .size(18.0)
                                        .color(favorite_color)
                                )
                                .corner_radius(CornerRadius::same(6))
                                .fill(self.theme.button_background)
                            );
                            
                            // Add tooltip for favorite button
                            if favorite_btn.hovered() {
                                let tooltip_pos = favorite_btn.rect.left_top() + egui::vec2(0.0, -30.0);
                                
                                // Use the story ID to make the tooltip unique per story
                                egui::Area::new(egui::Id::new("favorite_tooltip_area").with(story.id.clone()))
                                    .order(egui::Order::Tooltip)
                                    .fixed_pos(tooltip_pos)
                                    .show(&ctx, |ui| {
                                        egui::Frame::popup(ui.style())
                                            .fill(self.theme.card_background)
                                            .stroke(Stroke::new(1.0, self.theme.separator))
                                            .corner_radius(CornerRadius::same(6))
                                            .show(ui, |ui| {
                                                ui.add(egui::Label::new(
                                                    if is_favorite {
                                                        "Remove from Favorites"
                                                    } else {
                                                        "Add to Favorites"
                                                    }
                                                ));
                                            });
                                    });
                            }
                            
                            if favorite_btn.clicked() {
                                self.pending_favorites_toggle = Some(story.id.clone());
                            }
                            
                            
                            // Link button if URL exists
                            if !story.url.is_empty() {
                                ui.add_space(8.0);
                                let link_btn = ui.add_sized(
                                    [40.0, 28.0],
                                    egui::Button::new(
                                        RichText::new("↗")
                                            .size(18.0)
                                            .color(self.theme.button_foreground)
                                    )
                                    .corner_radius(CornerRadius::same(6))
                                    .fill(self.theme.button_background)
                                );
                                
                                // Add tooltip for the link button with improved stability
                                if link_btn.hovered() {
                                    let tooltip_pos = link_btn.rect.left_top() + egui::vec2(0.0, -30.0);
                                    
                                    // Use the story ID to make the tooltip unique per story
                                    egui::Area::new(egui::Id::new("link_tooltip_area").with(story.id.clone()))
                                        .order(egui::Order::Tooltip)
                                        .fixed_pos(tooltip_pos)
                                        .show(&ctx, |ui| {
                                            egui::Frame::popup(ui.style())
                                                .fill(self.theme.card_background)
                                                .stroke(Stroke::new(1.0, self.theme.separator))
                                                .corner_radius(CornerRadius::same(6))
                                                .show(ui, |ui| {
                                                    ui.add(egui::Label::new("Open Link"));
                                                });
                                        });
                                }
                                
                                if link_btn.clicked() {
                                    self.open_link(&story.url);
                                }
                            }
                        });
                    });
                });
                
                // Check if the card was clicked to select this story
                if card_response.response.clicked() {
                    // Set this story as the selected one
                    self.selected_story_index = Some(i);
                    
                    // Calculate the scroll position to center this story
                    const APPROX_STORY_HEIGHT: f32 = 150.0;
                    const APPROX_STORY_MARGIN: f32 = 20.0;
                    let viewport_height = ui.available_height();
                    let story_position = (i as f32) * (APPROX_STORY_HEIGHT + APPROX_STORY_MARGIN);
                    let center_position = story_position - (viewport_height / 2.0) + (APPROX_STORY_HEIGHT / 2.0);
                    self.stories_scroll_offset = center_position.max(0.0);
                    
                    // Mark that we need to repaint
                    self.needs_repaint = true;
                }
        }
        
        if let Some(story) = story_to_view {
            // Check if shift is held for forced refresh
            let force_refresh = ctx.input(|i| i.modifiers.shift);
            self.view_comments(story, force_refresh);
        }
        
        // No need to process favorite toggles here anymore - it's handled in update()
    }

    // Function to clean HTML content in comments
    fn clean_html(&self, html: &str) -> String {
        // Check if the result is already in the cache
        // We have to use a different approach since self.clean_html_cache is behind a Mutex
        // and we're in a method that takes &self
        let this = self as *const _ as *mut Self;
        
        // Get a hash of the HTML for cache lookup
        let html_hash = format!("{:x}", md5::compute(html));
        
        unsafe {
            // Check if the cleaned HTML is already in the cache
            if let Some(cached) = (*this).clean_html_cache.get(&html_hash) {
                return cached.clone();
            }
        }
        
        // If not in cache, process the HTML
        // First clean up simple cases without regexes for better performance
        let text = if html.len() < 100 && !html.contains('<') {
            // Very short text with no HTML - just return it directly
            html.to_string()
        } else {
            // Regular HTML cleaning 
            // Remove <a href="item?id=44025901">1 hour ago</a> style links but keep the text
            let item_link_regex = regex::Regex::new(r#"<a\s+href="item\?id=\d+"[^>]*>([^<]+)</a>"#).unwrap();
            let text = item_link_regex.replace_all(html, "$1");
            
            // Replace other HN-specific links with properly formatted ones
            let text = text.replace("<a href=\"https://news.ycombinator.com/", "<a href=\"");
            
            // Replace paragraph tags with newlines to maintain paragraph structure
            let text = text.replace("<p>", "\n").replace("</p>", "\n");
            
            // Replace <br> tags with newlines
            let text = text.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
            
            // Remove any remaining HTML tags while preserving text
            let regex = regex::Regex::new(r#"<[^>]+>"#).unwrap();
            let text = regex.replace_all(&text, "").to_string();
            
            // Normalize whitespace: replace multiple consecutive newlines with just two
            let whitespace_regex = regex::Regex::new(r"\n{3,}").unwrap();
            let text = whitespace_regex.replace_all(&text, "\n\n").to_string();
            
            // Decode HTML entities like &gt; to >
            html_escape::decode_html_entities(&text).to_string()
        };
        
        // Cache the result for future use
        unsafe {
            if (*this).clean_html_cache.len() > 5000 {
                // Prevent cache from growing too large - clear it if needed
                (*this).clean_html_cache.clear();
            }
            
            (*this).clean_html_cache.insert(html_hash, text.clone());
        }
        
        text
    }
    
    // Render pagination controls
    fn render_pagination_controls(&mut self, ui: &mut Ui) {
        let (current_page, total_pages, total_comments) = self.get_pagination_info();
        
        ui.horizontal(|ui| {
            // Font size controls
            ui.horizontal(|ui| {
                // Text size label
                ui.label(
                    RichText::new("Font Size:")
                        .color(self.theme.secondary_text)
                        .size(14.0)
                );
                
                // Add a slider for direct font size control
                if let Ok(mut font_size_guard) = GLOBAL_FONT_SIZE.lock() {
                    let mut font_size = *font_size_guard;
                    let slider = ui.add(egui::Slider::new(&mut font_size, 10.0..=24.0)
                        .step_by(1.0)
                        .text("pt"));
                    
                    if slider.changed() {
                        // Update the global font size
                        *font_size_guard = font_size;
                        
                        // Save the font size setting to the database
                        self.save_font_size_setting(font_size);
                    }
                }
                
                // Check if we need to repaint - done outside the closure
                self.needs_repaint = true;
                
                ui.add_space(10.0);
                
                // Decrease button
                let decrease_btn = ui.add(
                    egui::Button::new(
                        RichText::new("A-")
                            .color(self.theme.button_foreground)
                            .size(14.0)
                    )
                    .min_size(egui::Vec2::new(28.0, 28.0))
                    .corner_radius(CornerRadius::same(4))
                    .fill(self.theme.button_background)
                );
                
                if decrease_btn.clicked() {
                    // Call the decrease method which updates the global value
                    self.decrease_comment_font_size();
                    
                    // Force a repaint immediately
                    ui.ctx().request_repaint();
                }
                
                // Show current size
                if let Ok(font_size) = GLOBAL_FONT_SIZE.lock() {
                    ui.label(
                        RichText::new(format!("{:.0}pt", *font_size))
                            .color(self.theme.text)
                            .size(14.0)
                    );
                }
                
                // Increase button
                let increase_btn = ui.add(
                    egui::Button::new(
                        RichText::new("A+")
                            .color(self.theme.button_foreground)
                            .size(14.0)
                    )
                    .min_size(egui::Vec2::new(28.0, 28.0))
                    .corner_radius(CornerRadius::same(4))
                    .fill(self.theme.button_background)
                );
                
                if increase_btn.clicked() {
                    // Call the increase method which updates the global value
                    self.increase_comment_font_size();
                    
                    // Force a repaint immediately
                    ui.ctx().request_repaint();
                }
            });
            
            ui.add_space(12.0); // Add spacing before pagination info
            
            // Add a toggle for showing latest comments first
            let sort_button_text = if self.show_latest_comments_first {
                "⏱ Latest First"
            } else {
                "⌛ Default"
            };
            
            let sort_button = ui.add(
                egui::Button::new(
                    RichText::new(sort_button_text)
                        .color(self.theme.button_foreground)
                        .size(14.0)
                )
                .min_size(egui::Vec2::new(110.0, 28.0))
                .corner_radius(CornerRadius::same(4))
                .fill(if self.show_latest_comments_first {
                    self.theme.button_active_background
                } else {
                    self.theme.button_background
                })
            );
            
            if sort_button.clicked() {
                self.show_latest_comments_first = !self.show_latest_comments_first;
                
                // Reload comments with new order if a story is selected
                if let Some(story) = &self.selected_story {
                    let story_id = story.id.clone();
                    self.load_comments(&story_id);
                }
                
                self.needs_repaint = true;
            }
            
            if sort_button.hovered() {
                ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                
                // Show tooltip
                let tooltip_pos = egui::pos2(
                    sort_button.rect.left() + sort_button.rect.width() / 2.0,
                    sort_button.rect.bottom() + 4.0,
                );
                
                egui::Area::new(egui::Id::new("sort_tooltip_area"))
                    .order(egui::Order::Tooltip)
                    .fixed_pos(tooltip_pos)
                    .show(ui.ctx(), |ui| {
                        ui.label("Toggle between default and latest-first comment order");
                    });
            }
            
            ui.add_space(8.0);
            
            ui.label(
                RichText::new(format!("Showing page {} of {} ({} comments total)", 
                    current_page + 1, total_pages, total_comments))
                    .color(self.theme.secondary_text)
                    .size(14.0)
            );
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Next page button
                let next_enabled = current_page < total_pages - 1;
                let next_btn = ui.add_enabled(
                    next_enabled,
                    egui::Button::new(
                        RichText::new("➡") // Right arrow (U+27A1) instead of → (U+2192)
                            .color(if next_enabled { self.theme.button_foreground } else { self.theme.secondary_text })
                            .size(16.0)
                    )
                    .min_size(egui::Vec2::new(32.0, 28.0))
                    .corner_radius(CornerRadius::same(4))
                    .fill(self.theme.button_background)
                );
                
                if next_btn.clicked() && next_enabled {
                    // Use direct pointer manipulation instead of unsafe reference casting
                    let page = self.comments_page;
                    let this = self as *const _ as *mut Self;
                    // Safely update through a mutable pointer
                    unsafe { 
                        (*this).comments_page = page + 1;
                        (*this).needs_repaint = true;
                    }
                }
                
                // Page indicator
                ui.label(
                    RichText::new(format!("{} / {}", current_page + 1, total_pages))
                        .color(self.theme.text)
                        .size(14.0)
                );
                
                // Previous page button
                let prev_enabled = current_page > 0;
                let prev_btn = ui.add_enabled(
                    prev_enabled,
                    egui::Button::new(
                        RichText::new("⬅") // Left arrow (U+2B05) instead of ← (U+2190)
                            .color(if prev_enabled { self.theme.button_foreground } else { self.theme.secondary_text })
                            .size(16.0)
                    )
                    .min_size(egui::Vec2::new(32.0, 28.0))
                    .corner_radius(CornerRadius::same(4))
                    .fill(self.theme.button_background)
                );
                
                if prev_btn.clicked() && prev_enabled {
                    // Use direct pointer manipulation instead of unsafe reference casting
                    let page = self.comments_page;
                    let this = self as *const _ as *mut Self;
                    // Safely update through a mutable pointer
                    unsafe { 
                        (*this).comments_page = page.saturating_sub(1);
                        (*this).needs_repaint = true;
                    }
                }
            });
        });
    }

    // Render a single comment and its children (recursive)
    fn render_comment(&self, ui: &mut Ui, comment: &HackerNewsComment, depth: usize) {
        // Skip empty comments
        if comment.text.is_empty() || comment.text == "[deleted]" {
            return;
        }
        
        // Check if this comment is collapsed
        let is_collapsed = self.collapsed_comments.contains(&comment.id);
        
        // Constants for better performance with large comment threads
        const MAX_DEPTH: usize = 10;         // Maximum depth to render before showing "load more"
        const MAX_CHILDREN: usize = 50;      // Maximum number of children to render at once
        
        // Limit render depth to prevent performance issues with extremely nested comments
        if depth > MAX_DEPTH {
            // Just show a "load more" button for very deep nesting
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space((depth * 16) as f32);
                
                let load_more_btn = ui.add(
                    egui::Button::new(
                        RichText::new("⟨ Nested replies hidden - Click to expand ⟩")
                            .color(self.theme.secondary_text)
                            .italics()
                            .size(14.0)
                    )
                    .min_size(egui::Vec2::new(200.0, 30.0))
                    .fill(self.theme.card_background)
                );
                
                if load_more_btn.clicked() {
                    // When clicked, toggle the collapsed state of this comment
                    let comment_id = comment.id.clone();
                    let this = self as *const _ as *mut Self;
                    unsafe {
                        if (*this).collapsed_comments.contains(&comment_id) {
                            (*this).collapsed_comments.remove(&comment_id);
                        } else {
                            (*this).collapsed_comments.insert(comment_id);
                        }
                        (*this).needs_repaint = true;
                    }
                }
            });
            return;
        }
        
        // Card background based on depth and theme - simplified for better performance
        let card_bg = if depth % 2 == 0 {
            self.theme.card_background
        } else if self.is_dark_mode {
            // Dark theme alternating color (slightly transparent)
            Color32::from_rgba_premultiplied(
                self.theme.card_background.r(), 
                self.theme.card_background.g(), 
                self.theme.card_background.b(), 
                220
            )
        } else {
            // Light theme alternating color (slightly darker)
            Color32::from_rgb(
                self.theme.card_background.r().saturating_sub(10),
                self.theme.card_background.g().saturating_sub(10),
                self.theme.card_background.b().saturating_sub(10)
            )
        };
        
        // Comment card with indentation - simplified for performance
        egui::Frame::new()
            .fill(card_bg)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(1.0, self.theme.separator))
            .inner_margin(10.0)
            .outer_margin(egui::vec2(8.0, 4.0))
            .show(ui, |ui| {
                // Indent based on depth
                ui.horizontal(|ui| {
                    // Limit indentation to avoid excessive horizontal space
                    let scaled_depth = if depth > 5 { 5 + (depth - 5) / 2 } else { depth };
                    ui.add_space((scaled_depth * 16) as f32); 
                    
                    ui.vertical(|ui| {
                        // Comment metadata and collapse button in the same horizontal line
                        ui.horizontal(|ui| {
                            // Collapse/expand button - use simple ASCII characters for maximum compatibility
                            let collapse_btn_text = if is_collapsed { "[+]" } else { "[-]" }; // Simple brackets with plus/minus
                            let collapse_btn = ui.add(
                                egui::Button::new(
                                    RichText::new(collapse_btn_text)
                                        .color(self.theme.text)
                                        .monospace()
                                        .size(16.0) // Slightly larger
                                )
                                .small()
                                .frame(false)
                                .fill(Color32::TRANSPARENT)
                            );
                            
                            // Add hover effect
                            if collapse_btn.hovered() {
                                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                            }
                            
                            // Handle collapsing/expanding directly here for more reliable operation
                            if collapse_btn.clicked() {
                                // We need to access self mutably to update the collapsed_comments set
                                let comment_id = comment.id.clone();
                                let this = self as *const _ as *mut Self;
                                unsafe {
                                    // Toggle collapse state
                                    if (*this).collapsed_comments.contains(&comment_id) {
                                        (*this).collapsed_comments.remove(&comment_id);
                                    } else {
                                        (*this).collapsed_comments.insert(comment_id);
                                    }
                                    (*this).needs_repaint = true;
                                }
                            }
                            
                            ui.add_space(4.0);
                            
                            // User name
                            ui.label(
                                RichText::new(&comment.by)
                                    .color(self.theme.accent)
                                    .strong()
                                    .size(14.0)
                            );
                            ui.add_space(8.0);
                            
                            // Time ago
                            ui.label(
                                RichText::new(&comment.time_ago)
                                    .color(self.theme.secondary_text)
                                    .size(14.0)
                            );
                            
                            // Child comment count if collapsed
                            if is_collapsed && !comment.children.is_empty() {
                                let total_children = self.count_total_children(&comment.children);
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("{} replies", total_children))
                                        .color(self.theme.secondary_text)
                                        .italics()
                                        .size(14.0)
                                );
                            }
                            
                            // Store the response for collapsing/expanding logic
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("comment_collapse_btn").with(comment.id.clone()), collapse_btn);
                            });
                        });
                        
                        if !is_collapsed {
                            // Space between comment header and body
                            ui.add_space(6.0);
                            
                            // Comment text with cleaned HTML
                            let clean_text = self.clean_html(&comment.text);
                            
                            // Use the global font size
                            if let Ok(font_size) = GLOBAL_FONT_SIZE.lock() {
                                // Create a label with increased line spacing
                                let text_with_spacing = clean_text.replace("\n", "\n\n");
                                
                                // Apply the font size to the comment text and increase spacing
                                ui.label(
                                    RichText::new(&text_with_spacing)
                                        .color(self.theme.text)
                                        .size(*font_size) // Use the global font size
                                );
                            }
                            
                            // Recursively render child comments (only if not collapsed)
                            if !comment.children.is_empty() {
                                // Space between comment text and child comments
                                ui.add_space(8.0);
                                
                                // Limit the number of children rendered for very large threads
                                let children_count = comment.children.len();
                                let children_to_render = std::cmp::min(children_count, MAX_CHILDREN);
                                
                                // Render visible child comments
                                for child in comment.children.iter().take(children_to_render) {
                                    self.render_comment(ui, child, depth + 1);
                                }
                                
                                // Show "load more" button if there are more children
                                if children_count > MAX_CHILDREN {
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space((depth * 16) as f32);
                                        
                                        let remaining = children_count - MAX_CHILDREN;
                                        let load_more_btn = ui.add(
                                            egui::Button::new(
                                                RichText::new(format!("Show {} more replies...", remaining))
                                                    .color(self.theme.accent)
                                                    .size(14.0)
                                            )
                                            .min_size(egui::Vec2::new(160.0, 30.0))
                                            .fill(self.theme.card_background)
                                        );
                                        
                                        // Handle "load more" button - this would need state tracking
                                        // For now, we'll just collapse the comment on click as a placeholder
                                        if load_more_btn.clicked() {
                                            let comment_id = comment.id.clone();
                                            let this = self as *const _ as *mut Self;
                                            unsafe {
                                                // Collapse this comment to reset the view
                                                (*this).collapsed_comments.insert(comment_id);
                                                (*this).needs_repaint = true;
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    });
                });
            });
    }
    
    // Helper function to render all comments (used for recursive rendering)
    #[allow(dead_code)]
    fn render_comments(&self, ui: &mut Ui, comments: &[HackerNewsComment], depth: usize) {
        for comment in comments {
            self.render_comment(ui, comment, depth);
        }
    }
    
    fn count_total_children(&self, comments: &[HackerNewsComment]) -> usize {
        let mut count = comments.len();
        for comment in comments {
            count += self.count_total_children(&comment.children);
        }
        count
    }
    
    // Function to optimize very large comment threads for better performance
    fn optimize_large_comment_thread(&self, comments: Vec<HackerNewsComment>) -> Vec<HackerNewsComment> {
        // Placeholder values for maximum depth and children
        const MAX_OPTIMIZATION_DEPTH: usize = 12;  
        const MAX_CHILDREN_PER_COMMENT: usize = 100;
        
        // Process each top-level comment
        comments.into_iter()
            .map(|comment| self.optimize_comment_tree(comment, 0, MAX_OPTIMIZATION_DEPTH, MAX_CHILDREN_PER_COMMENT))
            .collect()
    }
    
    // Helper function to recursively optimize a comment tree
    fn optimize_comment_tree(&self, comment: HackerNewsComment, current_depth: usize, 
                            max_depth: usize, max_children: usize) -> HackerNewsComment {
        // Skip empty content
        if comment.text.is_empty() || comment.text == "[deleted]" {
            return comment;
        }
        
        // If we're at max depth, return with no children to save memory and CPU
        if current_depth >= max_depth {
            return HackerNewsComment {
                id: comment.id,
                by: comment.by,
                text: comment.text,
                time_ago: comment.time_ago,
                level: comment.level,
                children: Vec::new(), // No children beyond max depth
            };
        }
        
        // If too many children, limit them to save resources
        let children_count = comment.children.len();
        let reduced_children = if children_count > max_children {
            comment.children.into_iter()
                .take(max_children)
                .map(|child| self.optimize_comment_tree(child, current_depth + 1, max_depth, max_children))
                .collect()
        } else {
            comment.children.into_iter()
                .map(|child| self.optimize_comment_tree(child, current_depth + 1, max_depth, max_children))
                .collect()
        };
        
        // Return the optimized comment
        HackerNewsComment {
            id: comment.id,
            by: comment.by,
            text: comment.text,
            time_ago: comment.time_ago,
            level: comment.level,
            children: reduced_children,
        }
    }
    
    // Estimate the height of a comment for virtual scrolling optimization
    #[allow(dead_code)]
    fn estimate_comment_height(&self, comment: &HackerNewsComment, depth: usize) -> f32 {
        // Skip empty comments
        if comment.text.is_empty() || comment.text == "[deleted]" {
            return 0.0;
        }
        
        // Check if this comment is collapsed
        let is_collapsed = self.collapsed_comments.contains(&comment.id);
        
        // Base height for comment header
        let mut height = 40.0; // Header height
        
        // Add height for comment text if not collapsed
        if !is_collapsed {
            // Estimate text height based on length
            // Assuming average of 10 characters per line and 20 pixels per line
            let text_length = comment.text.len() as f32;
            let estimated_lines = (text_length / 80.0).max(1.0); // Assume 80 chars per line
            let text_height = estimated_lines * 20.0; // 20 pixels per line
            
            height += text_height;
            
            // Add spacing
            height += 20.0;
            
            // Add height for children recursively
            if !comment.children.is_empty() {
                let mut children_height = 0.0;
                
                for child in &comment.children {
                    children_height += self.estimate_comment_height(child, depth + 1);
                }
                
                height += children_height;
            }
        } else {
            // If collapsed, just add a small fixed height
            height += 10.0;
        }
        
        // Add margins
        height += 20.0;
        
        height
    }
    
    // Render the tab buttons
    fn render_tab_buttons(&mut self, ui: &mut Ui) {
        let button_size = [80.0, 32.0];
        
        // Hot tab
        let hot_btn = ui.add_sized(
            button_size,
            egui::Button::new(
                // Use a conditional to create different RichText objects based on the current tab
                if self.current_tab == Tab::Hot {
                    RichText::new("Hot")
                        .size(16.0)
                        .color(self.theme.highlight)
                        .strong()
                } else {
                    RichText::new("Hot")
                        .size(16.0)
                        .color(self.theme.secondary_text)
                }
            )
            .fill(if self.current_tab == Tab::Hot {
                self.theme.card_background
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if self.current_tab == Tab::Hot {
                egui::Stroke::new(2.0, self.theme.highlight)
            } else {
                egui::Stroke::NONE
            })
        );
        
        if hot_btn.clicked() {
            self.switch_tab(Tab::Hot);
        }
        
        // New tab
        let new_btn = ui.add_sized(
            button_size,
            egui::Button::new(
                // Use a conditional to create different RichText objects based on the current tab
                if self.current_tab == Tab::New {
                    RichText::new("New")
                        .size(16.0)
                        .color(self.theme.highlight)
                        .strong()
                } else {
                    RichText::new("New")
                        .size(16.0)
                        .color(self.theme.secondary_text)
                }
            )
            .fill(if self.current_tab == Tab::New {
                self.theme.card_background
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if self.current_tab == Tab::New {
                egui::Stroke::new(2.0, self.theme.highlight)
            } else {
                egui::Stroke::NONE
            })
        );
        
        if new_btn.clicked() {
            self.switch_tab(Tab::New);
        }
        
        // Show tab
        let show_btn = ui.add_sized(
            button_size,
            egui::Button::new(
                // Use a conditional to create different RichText objects based on the current tab
                if self.current_tab == Tab::Show {
                    RichText::new("Show")
                        .size(16.0)
                        .color(self.theme.highlight)
                        .strong()
                } else {
                    RichText::new("Show")
                        .size(16.0)
                        .color(self.theme.secondary_text)
                }
            )
            .fill(if self.current_tab == Tab::Show {
                self.theme.card_background
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if self.current_tab == Tab::Show {
                egui::Stroke::new(2.0, self.theme.highlight)
            } else {
                egui::Stroke::NONE
            })
        );
        
        if show_btn.clicked() {
            self.switch_tab(Tab::Show);
        }
        
        // Ask tab
        let ask_btn = ui.add_sized(
            button_size,
            egui::Button::new(
                if self.current_tab == Tab::Ask {
                    RichText::new("Ask")
                        .size(16.0)
                        .color(self.theme.highlight)
                        .strong()
                } else {
                    RichText::new("Ask")
                        .size(16.0)
                        .color(self.theme.secondary_text)
                }
            )
            .fill(if self.current_tab == Tab::Ask {
                self.theme.card_background
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if self.current_tab == Tab::Ask {
                egui::Stroke::new(2.0, self.theme.highlight)
            } else {
                egui::Stroke::NONE
            })
        );
        
        if ask_btn.clicked() {
            self.switch_tab(Tab::Ask);
        }
        
        // Jobs tab
        let jobs_btn = ui.add_sized(
            button_size,
            egui::Button::new(
                if self.current_tab == Tab::Jobs {
                    RichText::new("Jobs")
                        .size(16.0)
                        .color(self.theme.highlight)
                        .strong()
                } else {
                    RichText::new("Jobs")
                        .size(16.0)
                        .color(self.theme.secondary_text)
                }
            )
            .fill(if self.current_tab == Tab::Jobs {
                self.theme.card_background
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if self.current_tab == Tab::Jobs {
                egui::Stroke::new(2.0, self.theme.highlight)
            } else {
                egui::Stroke::NONE
            })
        );
        
        if jobs_btn.clicked() {
            self.switch_tab(Tab::Jobs);
        }
        
        // Best tab
        let best_btn = ui.add_sized(
            button_size,
            egui::Button::new(
                if self.current_tab == Tab::Best {
                    RichText::new("Best")
                        .size(16.0)
                        .color(self.theme.highlight)
                        .strong()
                } else {
                    RichText::new("Best")
                        .size(16.0)
                        .color(self.theme.secondary_text)
                }
            )
            .fill(if self.current_tab == Tab::Best {
                self.theme.card_background
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if self.current_tab == Tab::Best {
                egui::Stroke::new(2.0, self.theme.highlight)
            } else {
                egui::Stroke::NONE
            })
        );
        
        if best_btn.clicked() {
            self.switch_tab(Tab::Best);
        }
    }
}// Implement favorites management functionality
impl HackerNewsReaderApp {
    // Functions for favorites management
    #[allow(dead_code)]
    fn toggle_favorite(&mut self, story: &HackerNewsItem) {
        let is_favorite = match self.database.is_favorite(&story.id) {
            Ok(is_fav) => is_fav,
            Err(e) => {
                eprintln!("Error checking if story is favorited: {}", e);
                return;
            }
        };

        let result = if is_favorite {
            // Remove from favorites
            self.database.remove_favorite(&story.id)
        } else {
            // Add to favorites
            self.database.add_favorite(story)
        };
        
        if let Err(e) = result {
            eprintln!("Error toggling favorite status: {}", e);
            return;
        }
        
        // Set appropriate status message
        if is_favorite {
            self.set_status_message(format!("Removed '{}' from favorites", story.title));
        } else {
            self.set_status_message(format!("Added '{}' to favorites", story.title));
        }
        
        // Update our local favorites list
        self.reload_favorites();
    }
    
    fn add_to_todo(&mut self, story: &HackerNewsItem) {
        // Add to favorites if not already a favorite
        if !self.is_favorite(&story.id) {
            if let Err(e) = self.database.add_favorite(story) {
                eprintln!("Error adding story to favorites: {}", e);
                return;
            }
        }
        
        // Ensure it's marked as not done
        if self.is_done(&story.id) {
            if let Err(e) = self.database.toggle_favorite_done(&story.id) {
                eprintln!("Error marking story as todo: {}", e);
            }
        }
        
        // Reload favorites to reflect changes
        self.reload_favorites();
    }
    
    fn toggle_done(&mut self, story: &HackerNewsItem) {
        // If not a favorite yet, add it first
        if !self.is_favorite(&story.id) {
            if let Err(e) = self.database.add_favorite(story) {
                eprintln!("Error adding story to favorites: {}", e);
                return;
            }
        }
        
        // Toggle the done status
        if let Err(e) = self.database.toggle_favorite_done(&story.id) {
            eprintln!("Error toggling done status: {}", e);
            return;
        }
        
        // Reload favorites to reflect changes
        self.reload_favorites();
        self.needs_repaint = true;
    }

    fn reload_favorites(&mut self) {
        self.favorites_loading = true;

        match self.database.get_all_favorites() {
            Ok(favorites) => {
                self.favorites = favorites;
                self.favorites_loading = false;
                self.needs_repaint = true;
            }
            Err(e) => {
                eprintln!("Error loading favorites: {}", e);
                self.favorites_loading = false;
            }
        }
    }
    
    // Load history stories from the database
    fn load_history(&mut self) {
        self.history_loading = true;

        match self.database.get_viewed_stories() {
            Ok(history) => {
                self.history_stories = history;
                self.history_loading = false;
                self.needs_repaint = true;
            }
            Err(e) => {
                eprintln!("Error loading history: {}", e);
                self.history_loading = false;
            }
        }
    }

    fn fetch_and_view_story_by_id(&mut self, story_id: String) {
        let client = self.hn_client.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            match client.fetch_story_by_id(&story_id) {
                Ok(story) => {
                    let _ = tx.send(Some(story));
                }
                Err(e) => {
                    eprintln!("Error fetching story {}: {}", story_id, e);
                    let _ = tx.send(None::<HackerNewsItem>);
                }
            }
        });
        
        // Store the receiver to check for results in update loop
        self.story_fetch_receiver = Some(rx);
    }

    fn is_favorite(&self, id: &str) -> bool {
        match self.database.is_favorite(id) {
            Ok(is_fav) => is_fav,
            Err(_) => false,
        }
    }
    
    fn is_todo(&self, id: &str) -> bool {
        // A story is a "todo" if it's a favorite but not marked as done
        match self.database.is_favorite(id) {
            Ok(is_fav) => {
                if !is_fav {
                    return false;
                }
                
                // Check if it's not marked as done
                match self.get_favorite_done_status(id) {
                    Ok(is_done) => !is_done,
                    Err(_) => false,
                }
            },
            Err(_) => false,
        }
    }
    
    fn is_done(&self, id: &str) -> bool {
        self.get_favorite_done_status(id).unwrap_or(false)
    }
    
    fn get_favorite_done_status(&self, id: &str) -> Result<bool, anyhow::Error> {
        // Find the favorite in our cached list first for better performance
        for fav in &self.favorites {
            if fav.id == id {
                return Ok(fav.done);
            }
        }
        
        // If not found in cache, we don't know the done status
        // (we only know done status for favorites)
        Ok(false)
    }
    
    // Check if a story has been viewed by the user
    fn is_story_viewed(&self, story_id: &str) -> bool {
        // First check local set
        if self.viewed_story_ids.contains(story_id) {
            return true;
        }
        
        // Then check database (in case the set was not properly loaded)
        match self.database.is_story_viewed(story_id) {
            Ok(is_viewed) => is_viewed,
            Err(_) => false
        }
    }
    
    // Mark a story as viewed, updating both the local set and the database
    fn mark_story_as_viewed(&mut self, story_id: &str, story_title: Option<&str>) {
        // Update local set
        self.viewed_story_ids.insert(story_id.to_string());
        
        // Update database viewed status
        if let Err(e) = self.database.mark_story_as_viewed(story_id) {
            eprintln!("Error marking story as viewed: {}", e);
        }
        
        // If we have a title, save it as well
        if let Some(title) = story_title {
            if let Err(e) = self.database.save_story_details(story_id, title) {
                eprintln!("Error saving story details: {}", e);
            }
        }
    }
    
    fn toggle_favorites_panel(&mut self) {
        self.show_favorites_panel = !self.show_favorites_panel;
        
        // Reload favorites when panel is opened
        if self.show_favorites_panel {
            self.reload_favorites();
        }
        
        self.needs_repaint = true;
    }

    // Render the side panel with tabs for Favorites and History
    fn render_side_panel(&mut self, ctx: &egui::Context) {
        let open = self.show_favorites_panel;
        
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(300.0)
            .width_range(250.0..=400.0)
            .show_animated(ctx, open, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    
                    // Add tabs for Favorites and History
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            self.current_side_panel_tab == SidePanelTab::Favorites,
                            RichText::new("Favorites")
                                .size(16.0)
                                .color(self.theme.text)
                                .strong()
                        ).clicked() {
                            self.current_side_panel_tab = SidePanelTab::Favorites;
                            // Reload favorites when switching to this tab
                            self.reload_favorites();
                        }
                        
                        ui.add_space(20.0);
                        
                        if ui.selectable_label(
                            self.current_side_panel_tab == SidePanelTab::History,
                            RichText::new("History")
                                .size(16.0)
                                .color(self.theme.text)
                                .strong()
                        ).clicked() {
                            self.current_side_panel_tab = SidePanelTab::History;
                            // Load history when switching to this tab
                            self.load_history();
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.add(egui::Separator::default().spacing(8.0));
                    
                    // Render the content based on the selected tab
                    match self.current_side_panel_tab {
                        SidePanelTab::Favorites => self.render_favorites_content(ui),
                        SidePanelTab::History => self.render_history_content(ui),
                    }
                });
            });
            
        // Update the state variable if the panel was closed by clicking the X
        self.show_favorites_panel = open;
    }
    
    // Render the favorites tab content
    fn render_favorites_content(&mut self, ui: &mut egui::Ui) {
        if self.favorites_loading {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(8.0);
                ui.label("Loading favorites...");
            });
        } else if self.favorites.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("No favorites yet")
                        .color(self.theme.secondary_text)
                        .italics()
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Click the star icon on a story to add it to your favorites")
                        .color(self.theme.secondary_text)
                        .size(14.0)
                );
            });
        } else {
            // Render favorites list
            let favorites_clone = self.favorites.clone(); // Clone to avoid borrow issues
            let scroll_response = ScrollArea::vertical()
                .id_salt("favorites_scroll_area")
                .auto_shrink([false, false])
                .vertical_scroll_offset(self.favorites_scroll_offset)
                .show(ui, |ui| {
                    // Split favorites into "Todo" and "Done" lists
                    let (todo_favorites, done_favorites): (Vec<_>, Vec<_>) = 
                        favorites_clone.iter().partition(|f| !f.done);
                    
                    // Render "Todo" section
                    ui.vertical(|ui| {
                        ui.add_space(8.0);
                        ui.heading(
                            RichText::new("Todo")
                                .size(18.0)
                                .color(self.theme.text)
                        );
                        
                        if todo_favorites.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("No pending stories")
                                    .color(self.theme.secondary_text)
                                    .italics()
                            );
                        } else {
                            for favorite in &todo_favorites {
                                self.render_favorite_item_with_checkbox(ui, favorite);
                            }
                        }
                    });
                    
                    // Separator between Todo and Done
                    ui.add_space(16.0);
                    ui.add(egui::Separator::default().spacing(8.0));
                    
                    // Render "Done" section
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.heading(
                                RichText::new("Done")
                                    .size(18.0)
                                    .color(self.theme.text)
                            );
                            
                            // Only show clear button if there are done favorites
                            if !done_favorites.is_empty() {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let clear_btn = ui.add_sized(
                                        [60.0, 24.0],
                                        egui::Button::new(
                                            RichText::new("Clear All")
                                                .size(14.0)
                                                .color(self.theme.button_foreground)
                                        )
                                        .corner_radius(CornerRadius::same(4))
                                        .fill(self.theme.button_background)
                                    );
                                    
                                    if clear_btn.clicked() {
                                        // Clear all done favorites
                                        match self.database.clear_done_favorites() {
                                            Ok(count) => {
                                                println!("Cleared {} done favorites", count);
                                                // Reload favorites immediately
                                                self.reload_favorites();
                                            },
                                            Err(e) => {
                                                eprintln!("Error clearing done favorites: {}", e);
                                            }
                                        }
                                        self.needs_repaint = true;
                                    }
                                    
                                    if clear_btn.hovered() {
                                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                                    }
                                });
                            }
                        });
                        
                        if done_favorites.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("No completed stories")
                                    .color(self.theme.secondary_text)
                                    .italics()
                            );
                        } else {
                            for favorite in &done_favorites {
                                self.render_favorite_item_with_checkbox(ui, favorite);
                            }
                        }
                    });
                    
                    ui.add_space(20.0);
                });
                
            // Store the scroll position
            self.favorites_scroll_offset = scroll_response.state.offset.y;
        }
    }
    
    // Render the history tab content
    fn render_history_content(&mut self, ui: &mut egui::Ui) {
        // Add search bar at the top
        ui.horizontal(|ui| {
            ui.label(RichText::new("Search:").color(self.theme.text).size(14.0));
            ui.add_space(4.0);
            
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.history_search_query)
                    .hint_text("Search in history...")
                    .desired_width(220.0)
            );
            
            if response.changed() {
                // Search query changed, filter results
                self.needs_repaint = true;
            }
            
            if !self.history_search_query.is_empty() {
                // Add clear button for search
                if ui.button("✕").clicked() {
                    self.history_search_query.clear();
                    self.needs_repaint = true;
                }
            }
        });
        
        ui.add_space(8.0);
        
        if self.history_loading {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(8.0);
                ui.label("Loading history...");
            });
        } else if self.history_stories.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("No browsing history yet")
                        .color(self.theme.secondary_text)
                        .italics()
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("View stories to add them to your history")
                        .color(self.theme.secondary_text)
                        .size(14.0)
                );
            });
        } else {
            // Filter history stories based on search query
            // Clone stories to avoid borrow checker issues
            let stories_to_filter = self.history_stories.clone();
            let filtered_stories: Vec<db::ViewedStory> = if !self.history_search_query.is_empty() {
                let query = self.history_search_query.to_lowercase();
                stories_to_filter.into_iter()
                    .filter(|story| story.title.to_lowercase().contains(&query))
                    .collect()
            } else {
                stories_to_filter
            };
            
            if filtered_stories.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No matching stories found")
                            .color(self.theme.secondary_text)
                            .italics()
                    );
                });
            } else {
                // Render filtered history list
                let scroll_response = ScrollArea::vertical()
                    .id_salt("history_scroll_area")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(self.history_scroll_offset)
                    .show(ui, |ui| {
                        for story in &filtered_stories {
                            // Pass a reference to the story
                            self.render_history_item(ui, story);
                        }
                        
                        ui.add_space(20.0);
                    });
                    
                // Store the scroll position
                self.history_scroll_offset = scroll_response.state.offset.y;
            }
        }
    }
    
    fn render_favorite_item_with_checkbox(&mut self, ui: &mut egui::Ui, favorite: &FavoriteStory) {
        let mut view_story = false;
        
        // Favorite item card with checkbox
        ui.horizontal_wrapped(|ui| {
            // Checkbox for marking done
            let mut done = favorite.done;
            if ui.checkbox(&mut done, "").changed() {
                // Toggle done status in the database
                if let Err(e) = self.database.toggle_favorite_done(&favorite.id) {
                    eprintln!("Error toggling favorite done status: {}", e);
                } else {
                    // Reload favorites immediately
                    self.reload_favorites();
                }
                self.needs_repaint = true;
            }
            
            ui.vertical(|ui| {
                // Title with truncation if needed
                let title_text = if favorite.done {
                    // Strikethrough text for done items
                    RichText::new(&favorite.title)
                        .color(self.theme.secondary_text)
                        .strikethrough()
                } else {
                    RichText::new(&favorite.title)
                        .color(self.theme.text)
                        .strong()
                };
                
                let title_label = ui.add(
                    egui::Label::new(title_text)
                        .sense(egui::Sense::click())
                        .wrap()
                );
                
                // Handle click on title
                if title_label.clicked() {
                    view_story = true;
                }
                
                if title_label.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                
                // Meta row
                ui.horizontal(|ui| {
                    // Score
                    let score_color = self.theme.score_color(favorite.score);
                    ui.label(
                        RichText::new(format!("{} pts", favorite.score))
                            .color(score_color)
                            .size(13.0)
                    );
                    
                    ui.label(RichText::new("|").color(self.theme.separator).size(13.0));
                    
                    // Domain
                    if !favorite.domain.is_empty() {
                        ui.label(
                            RichText::new(&favorite.domain)
                                .color(self.theme.secondary_text)
                                .size(13.0)
                                .italics()
                        );
                        ui.label(RichText::new("|").color(self.theme.separator).size(13.0));
                    }
                    
                    // Author
                    ui.label(
                        RichText::new(&format!("by {}", favorite.by))
                            .color(self.theme.secondary_text)
                            .size(13.0)
                    );
                });
                
                // Action buttons
                ui.horizontal(|ui| {
                    // Info about when added
                    let added_local = favorite.added_at.with_timezone(&chrono::Local);
                    let date_str = added_local.format("%Y-%m-%d %H:%M").to_string();
                    
                    ui.label(
                        RichText::new(format!("Added: {}", date_str))
                            .color(self.theme.secondary_text)
                            .size(12.0)
                            .italics()
                    );
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // View comments button
                        let comments_btn = ui.add_sized(
                            [90.0, 24.0],
                            egui::Button::new(
                                RichText::new(format!("{} Comments", favorite.comments_count))
                                    .size(13.0)
                                    .color(self.theme.button_foreground)
                            )
                            .corner_radius(CornerRadius::same(4))
                            .fill(self.theme.button_background)
                        );
                        
                        if comments_btn.clicked() {
                            view_story = true;
                        }
                        
                        // Link button if URL exists
                        if !favorite.url.is_empty() {
                            ui.add_space(4.0);
                            let link_btn = ui.add_sized(
                                [30.0, 24.0],
                                egui::Button::new(
                                    RichText::new("↗")
                                        .size(16.0)
                                        .color(self.theme.button_foreground)
                                )
                                .corner_radius(CornerRadius::same(4))
                                .fill(self.theme.button_background)
                            );
                            
                            if link_btn.clicked() {
                                let url = favorite.url.clone();
                                self.open_link(&url);
                            }
                        }
                        
                        // Remove favorite button
                        ui.add_space(4.0);
                        let remove_btn = ui.add_sized(
                            [30.0, 24.0],
                            egui::Button::new(
                                RichText::new("✖")
                                    .size(16.0)
                                    .color(self.theme.highlight)
                            )
                            .corner_radius(CornerRadius::same(4))
                            .fill(self.theme.button_background)
                        );
                        
                        if remove_btn.clicked() {
                            // Store id for removal after ui rendering
                            if let Err(e) = self.database.remove_favorite(&favorite.id) {
                                eprintln!("Error removing favorite: {}", e);
                            } else {
                                // Reload favorites immediately
                                self.reload_favorites();
                            }
                            self.needs_repaint = true;
                        }
                    });
                });
            });
        });
        
        // Add separator between items
        ui.add(egui::Separator::default().spacing(8.0));
        
        // Handle navigation
        if view_story {
            let story = HackerNewsItem::from(favorite.clone());
            self.view_comments(story, false);
            self.show_favorites_panel = false;
            self.needs_repaint = true;
        }
    }
    
    #[allow(dead_code)]
    fn render_history_item(&mut self, ui: &mut egui::Ui, story: &db::ViewedStory) {
        ui.add_space(8.0);
        
        // Create a card for each history item
        egui::Frame::new()
            .fill(self.theme.card_background)
            .corner_radius(egui::CornerRadius::same(8))
            .stroke(egui::Stroke::new(1.0, self.theme.separator))
            .inner_margin(8.0)
            .outer_margin(4.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Title with wrapped text
                    let title_text = RichText::new(&story.title)
                        .color(self.theme.get_viewed_story_color())
                        .size(16.0);
                    
                    let title_label = ui.add(
                        egui::Label::new(title_text)
                            .wrap()
                            .sense(egui::Sense::click())
                    );
                    
                    // Handle click on history item
                    if title_label.clicked() {
                        // First check if we have this story in our current stories list
                        let mut found_in_current_stories = false;
                        for current_story in &self.stories {
                            if current_story.id == story.id {
                                let story_clone = current_story.clone();
                                self.view_comments(story_clone, false);
                                found_in_current_stories = true;
                                break;
                            }
                        }
                        
                        // If not found in current stories, fetch it from the API
                        if !found_in_current_stories {
                            self.fetch_and_view_story_by_id(story.id.clone());
                        }
                    }
                    
                    if title_label.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                    
                    // Metadata row
                    ui.horizontal(|ui| {
                        // Add when viewed timestamp
                        let viewed_local = story.viewed_at.with_timezone(&chrono::Local);
                        let date_str = viewed_local.format("%Y-%m-%d %H:%M").to_string();
                        
                        ui.label(
                            RichText::new(format!("Viewed: {}", date_str))
                                .color(self.theme.secondary_text)
                                .size(13.0)
                                .italics()
                        );
                        
                        // Add a star button to save to favorites
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let is_favorite = self.is_favorite(&story.id);
                            let star_icon = if is_favorite { "★" } else { "☆" };
                            let star_color = if is_favorite { self.theme.highlight } else { self.theme.secondary_text };
                            
                            let star_btn = ui.add(
                                egui::Button::new(
                                    RichText::new(star_icon)
                                        .size(16.0)
                                        .color(star_color)
                                )
                                .frame(false)
                            );
                            
                            if star_btn.clicked() {
                                if is_favorite {
                                    // Remove from favorites
                                    if let Err(e) = self.database.remove_favorite(&story.id) {
                                        eprintln!("Error removing favorite: {}", e);
                                    }
                                } else {
                                    // Find the story in our list to get all details
                                    for current_story in &self.stories {
                                        if current_story.id == story.id {
                                            if let Err(e) = self.database.add_favorite(current_story) {
                                                eprintln!("Error adding favorite: {}", e);
                                            }
                                            break;
                                        }
                                    }
                                }
                                self.reload_favorites();
                            }
                        });
                    });
                });
            });
    }
    
    #[allow(dead_code)]
    fn render_favorite_item(&mut self, ui: &mut egui::Ui, favorite: &FavoriteStory) {
        let story_clone = HackerNewsItem::from(favorite.clone());
        let mut view_story = false;
        
        // Favorite item card
        egui::Frame::default()
            .fill(self.theme.card_background)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(1.0, self.theme.separator))
            .inner_margin(egui::vec2(10.0, 10.0))
            .outer_margin(egui::vec2(6.0, 4.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Title with truncation if needed
                    let title_label = ui.add(
                        egui::Label::new(
                            RichText::new(&favorite.title)
                                .color(self.theme.text)
                                .strong()
                        )
                        .sense(egui::Sense::click())
                        .wrap()
                    );
                    
                    // Handle click on title
                    if title_label.clicked() {
                        view_story = true;
                    }
                    
                    if title_label.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                    
                    // Meta row
                    ui.horizontal(|ui| {
                        // Score
                        let score_color = self.theme.score_color(favorite.score);
                        ui.label(
                            RichText::new(format!("{} pts", favorite.score))
                                .color(score_color)
                                .size(13.0)
                        );
                        
                        ui.label(RichText::new("|").color(self.theme.separator).size(13.0));
                        
                        // Domain
                        if !favorite.domain.is_empty() {
                            ui.label(
                                RichText::new(&favorite.domain)
                                    .color(self.theme.secondary_text)
                                    .size(13.0)
                                    .italics()
                            );
                            ui.label(RichText::new("|").color(self.theme.separator).size(13.0));
                        }
                        
                        // Author
                        ui.label(
                            RichText::new(&format!("by {}", favorite.by))
                                .color(self.theme.secondary_text)
                                .size(13.0)
                        );
                    });
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        // Info about when added
                        let added_local = favorite.added_at.with_timezone(&chrono::Local);
                        let date_str = added_local.format("%Y-%m-%d %H:%M").to_string();
                        
                        ui.label(
                            RichText::new(format!("Added: {}", date_str))
                                .color(self.theme.secondary_text)
                                .size(12.0)
                                .italics()
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // View comments button
                            let comments_btn = ui.add_sized(
                                [90.0, 24.0],
                                egui::Button::new(
                                    RichText::new(format!("{} Comments", favorite.comments_count))
                                        .size(13.0)
                                        .color(self.theme.button_foreground)
                                )
                                .corner_radius(CornerRadius::same(4))
                                .fill(self.theme.button_background)
                            );
                            
                            if comments_btn.clicked() {
                                view_story = true;
                            }
                            
                            // Link button if URL exists
                            if !favorite.url.is_empty() {
                                ui.add_space(4.0);
                                let link_btn = ui.add_sized(
                                    [30.0, 24.0],
                                    egui::Button::new(
                                        RichText::new("↗")
                                            .size(16.0)
                                            .color(self.theme.button_foreground)
                                    )
                                    .corner_radius(CornerRadius::same(4))
                                    .fill(self.theme.button_background)
                                );
                                
                                if link_btn.clicked() {
                                    self.open_link(&favorite.url);
                                }
                            }
                            
                            // Remove favorite button
                            ui.add_space(4.0);
                            let remove_btn = ui.add_sized(
                                [30.0, 24.0],
                                egui::Button::new(
                                    RichText::new("✖")
                                        .size(16.0)
                                        .color(self.theme.highlight)
                                )
                                .corner_radius(CornerRadius::same(4))
                                .fill(self.theme.button_background)
                            );
                            
                            if remove_btn.clicked() {
                                if let Err(e) = self.database.remove_favorite(&favorite.id) {
                                    eprintln!("Error removing favorite: {}", e);
                                } else {
                                    self.reload_favorites();
                                }
                            }
                        });
                    });
                });
            });
            
        if view_story {
            self.view_comments(story_clone, false);
            // Close the favorites panel when selecting a story
            if self.show_favorites_panel {
                self.toggle_favorites_panel();
            }
        }
    }
}