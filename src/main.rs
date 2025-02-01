// src/main.rs
use eframe::egui;
use anyhow::Result;

mod analysis;
mod app;
mod config;
mod file;
mod input;
mod state;
mod ui;

use app::{App, AtlasApp};

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Atlas"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Atlas",
        options,
        Box::new(|cc| {
            // Customize egui here with cc.egui_ctx if needed
            Box::new(AtlasApp::new())
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))
}