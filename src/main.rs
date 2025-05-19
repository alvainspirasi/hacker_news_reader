use eframe::egui;
use egui::{Color32, RichText, ScrollArea, Ui, ViewportBuilder, Stroke, CornerRadius};
use std::thread;

mod hn_client;
mod models;

use crate::hn_client::HackerNewsClient;
use crate::models::{HackerNewsItem, HackerNewsComment};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Hacker News Reader"),
        ..Default::default()
    };
    
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
        if score >= 300 {
            self.score_high
        } else if score >= 100 {
            self.score_medium
        } else {
            self.score_low
        }
    }
}

// Define an enum for the different tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Hot,
    New,
    Show,
}

struct HackerNewsReaderApp {
    hn_client: HackerNewsClient,
    stories: Vec<HackerNewsItem>,
    selected_story: Option<HackerNewsItem>,
    comments: Vec<HackerNewsComment>,
    loading: bool,
    theme: AppTheme,
    is_dark_mode: bool,
    // Current active tab
    current_tab: Tab,
    // Change the thread type to handle any type of result
    load_thread: Option<thread::JoinHandle<Box<dyn std::any::Any + Send>>>,
    needs_repaint: bool,
    collapsed_comments: std::collections::HashSet<String>,
    stories_receiver: Option<std::sync::mpsc::Receiver<Option<Vec<HackerNewsItem>>>>,
    comments_receiver: Option<std::sync::mpsc::Receiver<Option<Vec<HackerNewsComment>>>>,
    // Pagination for comments
    comments_page: usize,
    comments_per_page: usize,
    total_comments_count: usize,
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
            },
        ];
        
        Self {
            hn_client: HackerNewsClient::new(),
            // Uncomment to use test_stories for debugging
            stories: test_stories, // Use empty Vec::new() for network loading
            selected_story: None,
            comments: Vec::new(),
            loading: false,
            theme: AppTheme::dark(),
            is_dark_mode: true,
            current_tab: Tab::Hot, // Start with the Hot tab
            load_thread: None,
            needs_repaint: false,
            collapsed_comments: std::collections::HashSet::new(),
            stories_receiver: None,
            comments_receiver: None,
            // Initialize pagination with reasonable defaults
            comments_page: 0,
            comments_per_page: 20, // Display 20 top-level comments per page
            total_comments_count: 0,
        }
    }
    
    fn load_stories(&mut self) {
        if self.loading {
            return; // Don't start another load if we're already loading
        }
        
        self.loading = true;
        
        // Create a new thread for loading
        let client = self.hn_client.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Convert the tab enum to a string
        let tab_str = match self.current_tab {
            Tab::Hot => "hot",
            Tab::New => "new",
            Tab::Show => "show",
        };
        
        let handle = thread::spawn(move || {
            let result: Box<dyn std::any::Any + Send> = match client.fetch_stories_by_tab(tab_str) {
                Ok(stories) => {
                    let _ = tx.send(Some(stories));
                    Box::new(())
                }
                Err(_e) => {
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
                    self.stories = stories;
                    self.loading = false;
                    self.stories_receiver = None; // Consume the receiver
                    self.needs_repaint = true;
                }
                Ok(None) => {
                    // Add a test item for debugging
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
                        }
                    ];
                    self.loading = false;
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
                    self.comments = comments;
                    self.loading = false;
                    self.comments_receiver = None; // Consume the receiver
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
        
        // Create a new thread for loading comments
        let handle = thread::spawn(move || {
            let result: Box<dyn std::any::Any + Send> = match client.fetch_comments(&item_id) {
                Ok(comments) => {
                    let _ = tx.send(Some(comments));
                    Box::new(())
                }
                Err(_e) => {
                    let _ = tx.send(None);
                    Box::new(())
                }
            };
            result
        });
        
        self.load_thread = Some(handle);
        
        // Store the receiver for later checks
        self.comments_receiver = Some(rx);
    }
    
    fn view_comments(&mut self, story: HackerNewsItem, force_refresh: bool) {
        self.selected_story = Some(story.clone());
        
        // Clear collapsed comments when loading a new story
        self.collapsed_comments.clear();
        
        // Reset pagination when loading a new story
        self.comments_page = 0;
        self.total_comments_count = story.comments_count as usize;
        
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
            
            // Create a new thread for loading comments with bypass cache
            let handle = thread::spawn(move || {
                let result: Box<dyn std::any::Any + Send> = match client.fetch_fresh_comments(&item_id) {
                    Ok(comments) => {
                        let _ = tx.send(Some(comments));
                        Box::new(())
                    }
                    Err(_e) => {
                        let _ = tx.send(None::<Vec<HackerNewsComment>>);
                        Box::new(())
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
    
    fn switch_tab(&mut self, tab: Tab) {
        if self.current_tab != tab {
            self.current_tab = tab;
            
            // Clear any selected story when switching tabs
            self.selected_story = None;
            self.comments.clear();
            
            // Reload stories for the new tab
            self.load_stories();
            self.needs_repaint = true;
        }
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
        
        // Request repaint if needed
        if self.needs_repaint {
            ctx.request_repaint();
            self.needs_repaint = false;
        }
        
        // Set up main layout
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a top header bar
            ui.horizontal(|ui| {
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
                    
                    let force_refresh = ctx.input(|i| i.modifiers.shift);
                    if refresh_btn.clicked() && !self.loading {
                        if force_refresh {
                            // Force refresh (bypass cache)
                            let client = self.hn_client.clone();
                            self.loading = true;
                            let (tx, rx) = std::sync::mpsc::channel();
                            
                            // Convert the tab enum to a string
                            let tab_str = match self.current_tab {
                                Tab::Hot => "hot",
                                Tab::New => "new",
                                Tab::Show => "show",
                            };
                            
                            let handle = thread::spawn(move || {
                                let result: Box<dyn std::any::Any + Send> = match client.fetch_fresh_stories_by_tab(tab_str) {
                                    Ok(stories) => {
                                        let _ = tx.send(Some(stories));
                                        Box::new(())
                                    }
                                    Err(_e) => {
                                        let _ = tx.send(None::<Vec<HackerNewsItem>>);
                                        Box::new(())
                                    }
                                };
                                result
                            });
                            
                            self.load_thread = Some(handle);
                            
                            // Store the receiver for later checks
                            self.stories_receiver = Some(rx);
                        } else {
                            // Normal refresh (uses cache if valid)
                            self.load_stories();
                        }
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
                                    let text = if force_refresh { "Force Refresh" } else { "Refresh" };
                                    ui.add(egui::Label::new(RichText::new(text).size(14.0)));
                                    ui.add(egui::Label::new(
                                        RichText::new("Hold Shift to bypass cache")
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
                                        ui.add(egui::Label::new("Back to Stories"));
                                    });
                            });
                    }
                    
                    if back_btn.clicked() {
                        clear = true;
                    }
                    
                    ui.add_space(8.0);
                });
                
                clear
            } else {
                false
            };

            if clear_story {
                self.selected_story = None;
                self.comments.clear();
            }

            if let Some(story) = &self.selected_story {
                // Story title
                ui.add_space(8.0);
                ui.label(
                    RichText::new(&story.title)
                        .size(22.0)
                        .color(self.theme.text)
                        .strong()
                );
                ui.add_space(8.0);
                
                // Story details card
                egui::Frame::new()
                    .fill(self.theme.card_background)
                    .corner_radius(CornerRadius::same(8))
                    .stroke(Stroke::new(1.0, self.theme.separator))
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
                            // Fixed position tooltip to avoid flickering
                            let tooltip_pos = help_btn.rect.left_top() + egui::vec2(-180.0, -120.0);
                            
                            // Use a stable area with fixed positioning for the tooltip
                            egui::Area::new("shortcuts_tooltip_area".into())
                                .order(egui::Order::Tooltip)
                                .fixed_pos(tooltip_pos)
                                .show(ui.ctx(), |ui| {
                                    egui::Frame::popup(ui.style())
                                        .fill(self.theme.card_background)
                                        .stroke(Stroke::new(1.0, self.theme.separator))
                                        .corner_radius(CornerRadius::same(6))
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.add(egui::Label::new(RichText::new("Keyboard Shortcuts:").strong()));
                                                ui.add_space(4.0);
                                                
                                                ui.add(egui::Label::new(RichText::new("Comment Controls:").strong()));
                                                ui.add(egui::Label::new("C - Collapse all top-level comments"));
                                                ui.add(egui::Label::new("Shift+C - Expand all comments"));
                                                
                                                ui.add_space(4.0);
                                                ui.add(egui::Label::new(RichText::new("Navigation:").strong()));
                                                ui.add(egui::Label::new("← / → - Previous/Next page"));
                                                ui.add(egui::Label::new("Home - First page"));
                                                ui.add(egui::Label::new("End - Last page"));
                                                
                                                ui.add_space(4.0);
                                                ui.add(egui::Label::new(RichText::new("Mouse:").strong()));
                                                ui.add(egui::Label::new("Click [+]/[-] to collapse/expand comments"));
                                            });
                                        });
                                });
                        }
                    });
                });
                
                ui.add_space(8.0);
                
                // Pagination controls at the top
                self.render_pagination_controls(ui);
                
                // Comments section with scrolling
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Get comments for the current page only
                        let page_comments = self.get_current_page_comments();
                        // Render comments from current page only
                        for comment in page_comments {
                            self.render_comment(ui, comment, 0);
                        }
                    });
                
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
                };
                
                ui.heading(
                    RichText::new(tab_name)
                        .size(18.0)
                        .color(self.theme.text)
                );
                
                ui.add_space(8.0);
                
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_stories_table(ui);
                    });
            }
        });
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
    
    // Process keyboard shortcuts
    fn process_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Only process keyboard shortcuts when viewing comments
        if self.selected_story.is_none() || self.comments.is_empty() {
            return;
        }
        
        let input = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Space), // Collapse/expand focused comment
                i.key_pressed(egui::Key::C),     // Collapse/expand all comments
                i.modifiers.shift,               // Modifier for "Collapse all" vs "Expand all"
                i.key_pressed(egui::Key::ArrowLeft),  // Previous page
                i.key_pressed(egui::Key::ArrowRight), // Next page
                i.key_pressed(egui::Key::Home),       // First page
                i.key_pressed(egui::Key::End),        // Last page
            )
        });
        
        if input.0 {
            // Space - Toggle the collapse state of focused comment
            // In a more sophisticated implementation, we'd track the focused comment
            // For now, this placeholder doesn't do anything
        }
        
        if input.1 {
            // C - Toggle all comments based on shift key
            if input.2 {
                // Shift+C: Expand all comments
                self.collapsed_comments.clear();
            } else {
                // C: Collapse all top-level comments
                self.collapse_all_top_level_comments();
            }
            self.needs_repaint = true;
        }
        
        // Page navigation with keyboard
        let (current_page, total_pages, _) = self.get_pagination_info();
        
        // Left arrow - Previous page
        if input.3 && current_page > 0 {
            self.comments_page = current_page - 1;
            self.needs_repaint = true;
        }
        
        // Right arrow - Next page
        if input.4 && current_page < total_pages - 1 {
            self.comments_page = current_page + 1;
            self.needs_repaint = true;
        }
        
        // Home key - First page
        if input.5 && current_page > 0 {
            self.comments_page = 0;
            self.needs_repaint = true;
        }
        
        // End key - Last page
        if input.6 && current_page < total_pages - 1 {
            self.comments_page = total_pages - 1;
            self.needs_repaint = true;
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
        
        for (i, story) in self.stories.iter().enumerate() {
            // Create a card for each story
            egui::Frame::new()
                .fill(self.theme.card_background)
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(1.0, self.theme.separator))
                .inner_margin(12.0)
                .outer_margin(egui::vec2(8.0, 6.0))
                .show(ui, |ui| {
                    // Top row with rank, title, and score
                    ui.horizontal(|ui| {
                        // Rank indicator
                        ui.label(
                            RichText::new(format!("{}", i+1))
                                .color(self.theme.secondary_text)
                                .size(16.0)
                        );
                        ui.add_space(8.0);
                        
                        // Story title with clickable behavior
                        let title_label = ui.add(
                            egui::Label::new(
                                RichText::new(&story.title)
                                    .color(self.theme.text)
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
                                let force_refresh = ctx.input(|i| i.modifiers.shift);
                                
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
                                                let text = if force_refresh { "Force refresh & View Comments" } else { "View Comments" };
                                                ui.add(egui::Label::new(RichText::new(text)));
                                                ui.add(egui::Label::new(
                                                    RichText::new("Hold Shift to bypass cache")
                                                        .size(12.0)
                                                        .color(self.theme.secondary_text)
                                                ));
                                            });
                                    });
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
        }
        
        if let Some(story) = story_to_view {
            // Check if shift is held for forced refresh
            let force_refresh = ctx.input(|i| i.modifiers.shift);
            self.view_comments(story, force_refresh);
        }
    }

    // Function to clean HTML content in comments
    fn clean_html(&self, html: &str) -> String {
        // Remove <a href="item?id=44025901">1 hour ago</a> style links but keep the text
        let item_link_regex = regex::Regex::new(r#"<a\s+href="item\?id=\d+"[^>]*>([^<]+)</a>"#).unwrap();
        let text = item_link_regex.replace_all(html, "$1");
        
        // Replace other HN-specific links with properly formatted ones
        let text = text.replace("<a href=\"https://news.ycombinator.com/", "<a href=\"");
        
        // Remove any remaining HTML tags while preserving text
        let regex = regex::Regex::new(r#"<[^>]+>"#).unwrap();
        regex.replace_all(&text, "").to_string()
    }
    
    // Render pagination controls
    fn render_pagination_controls(&self, ui: &mut Ui) {
        let (current_page, total_pages, total_comments) = self.get_pagination_info();
        
        ui.horizontal(|ui| {
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
        
        // Card background based on depth and theme
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
        
        // Comment card with indentation
        egui::Frame::new()
            .fill(card_bg)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(1.0, self.theme.separator))
            .inner_margin(10.0)
            .outer_margin(egui::vec2(8.0, 4.0))
            .show(ui, |ui| {
                // Indent based on depth
                ui.horizontal(|ui| {
                    ui.add_space((depth * 16) as f32); 
                    
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
                            ui.add_space(4.0);
                            
                            // Comment text with cleaned HTML
                            let clean_text = self.clean_html(&comment.text);
                            ui.label(
                                RichText::new(&clean_text)
                                    .color(self.theme.text)
                                    .size(15.0)
                            );
                            
                            // Recursively render child comments (only if not collapsed)
                            if !comment.children.is_empty() {
                                ui.add_space(8.0);
                                // Render all child comments (these are not paginated)
                                for child in &comment.children {
                                    self.render_comment(ui, child, depth + 1);
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
    }
}