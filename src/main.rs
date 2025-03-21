mod modes;
mod operations;
mod editor;
mod app;

use app::NotesApp;
use std::path::PathBuf;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Set up logging
    env_logger::init();
    
    // Application options
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        min_window_size: Some(egui::vec2(600.0, 400.0)),
        hardware_acceleration: eframe::HardwareAcceleration::Preferred, // For blazing fast performance!
        ..Default::default()
    };
    
    // Create application with notes directory in user's documents folder
    let notes_dir = dirs::document_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("vimnote");
    
    eframe::run_native(
        "VimNote",
        options,
        Box::new(|cc| {
            // Set custom fonts if available
            let fonts = egui::FontDefinitions::default();
            
            // Use default monospace font for code - no custom font loading
            cc.egui_ctx.set_fonts(fonts);
            
            // Enable global dark mode by default
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            Box::new(NotesApp::new(notes_dir))
        }),
    )
}