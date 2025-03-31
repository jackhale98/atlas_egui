// src/utils.rs
use crate::config::{Component, Feature};
use eframe::egui;
use std::path::Path;
use std::sync::Arc;

pub fn find_feature<'a>(components: &'a [Component], component_name: &str, feature_name: &str) -> Option<&'a Feature> {
    components.iter()
        .find(|c| c.name == component_name)?
        .features.iter()
        .find(|f| f.name == feature_name)
}

/// Load the application icon from the specified SVG file
pub fn load_app_icon() -> Arc<egui::IconData> {
    let svg_path = Path::new("assets/atlas_logo.svg");
    
    // Attempt to load the SVG file
    match std::fs::read(svg_path) {
        Ok(svg_data) => {
            // Try to load the icon from SVG
            if let Some(icon) = load_svg_icon(&svg_data) {
                Arc::new(icon)
            } else {
                // Return a fallback icon on failure
                Arc::new(create_fallback_icon())
            }
        },
        Err(err) => {
            eprintln!("Failed to load icon: {}", err);
            // Return a fallback icon on failure
            Arc::new(create_fallback_icon())
        }
    }
}

/// Convert SVG data to an egui::IconData
fn load_svg_icon(svg_data: &[u8]) -> Option<egui::IconData> {
    let icon_size: u32 = 64; // Size to render the icon (pixels)
    
    // Parse SVG with default options
    let opt = usvg::Options::default();
    match usvg::Tree::from_data(svg_data, &opt) {
        Ok(tree) => {
            // Get the size of the SVG
            let svg_size = tree.size();
            
            // Create a pixmap to render to
            match tiny_skia::Pixmap::new(icon_size, icon_size) {
                Some(mut pixmap) => {
                    // Create a transform that will scale and center the SVG
                    let xform = {
                        // Calculate scaling to fit the SVG in the icon
                        let scale_x = icon_size as f32 / svg_size.width();
                        let scale_y = icon_size as f32 / svg_size.height();
                        let scale = scale_x.min(scale_y);
                        
                        // Calculate translation to center the SVG
                        let tx = (icon_size as f32 - svg_size.width() * scale) / 2.0;
                        let ty = (icon_size as f32 - svg_size.height() * scale) / 2.0;
                        
                        // Create the transform
                        usvg::Transform::from_scale(scale, scale).pre_translate(tx, ty)
                    };
                    
                    // Clear the pixmap with transparent background
                    pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0));
                    
                    // Render the SVG
                    resvg::render(&tree, xform, &mut pixmap.as_mut());
                    
                    // Convert the pixmap to RGBA data for egui
                    Some(egui::IconData {
                        rgba: pixmap.data().to_vec(),
                        width: icon_size,
                        height: icon_size,
                    })
                },
                None => None
            }
        },
        Err(_) => None
    }
}

/// Create a simple fallback icon if the SVG loading fails
fn create_fallback_icon() -> egui::IconData {
    let icon_size: u32 = 64;
    let icon_pixels = (icon_size * icon_size) as usize;
    let mut rgba = vec![0; icon_pixels * 4];
    
    // Fill with a blue background
    for i in 0..icon_pixels {
        let idx = i * 4;
        rgba[idx] = 30;     // R - dark blue
        rgba[idx + 1] = 58;  // G
        rgba[idx + 2] = 138; // B
        rgba[idx + 3] = 255; // A - fully opaque
    }
    
    egui::IconData {
        rgba,
        width: icon_size,
        height: icon_size,
    }
}