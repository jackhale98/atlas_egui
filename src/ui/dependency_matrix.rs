// src/ui/dependency_matrix.rs
use eframe::egui;
use petgraph::graph::{NodeIndex, EdgeIndex};
use std::collections::{HashMap, HashSet};
use crate::state::{AppState, Screen};
use crate::config::{Component, Feature};

// Helper function to format component.feature - defined at module level
fn format_feature_text(comp: &str, feat: &str) -> String {
    format!("{}.{}", comp, feat)
}

pub fn show_dependency_matrix(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Component Feature Dependencies");
    
    // Update mate state to ensure the dependency graph is current
    state.update_mate_state();
    
    // Build feature list from all components
    let mut all_features: Vec<(String, String)> = Vec::new(); // (component_name, feature_name)
    for component in &state.components {
        for feature in &component.features {
            all_features.push((component.name.clone(), feature.name.clone()));
        }
    }
    
    // Sort features for consistent display
    all_features.sort_by(|a, b| {
        let cmp = a.0.cmp(&b.0);
        if cmp == std::cmp::Ordering::Equal {
            a.1.cmp(&b.1)
        } else {
            cmp
        }
    });
    
    if all_features.is_empty() {
        ui.label("No features found. Create components with features to see dependencies.");
        return;
    }
    
    // Create a scrollable matrix with frozen headers
    let table_size = egui::Vec2::new(ui.available_width(), ui.available_height() - 40.0);
    
    // Calculate cell size and header sizes
    let cell_size = 32.0;
    let base_header_width = 200.0;  // Increased for better text display
    let header_height = 120.0;      // Increased for header height
    
    // Get the longest text to calculate header width
    let longest_text_width = all_features.iter()
        .map(|(c, f)| format_feature_text(c, f))
        .fold(0.0_f32, |max_width, text| {
            let galley = ui.fonts(|f| f.layout_no_wrap(
                text, 
                egui::FontId::default(), 
                ui.style().visuals.text_color()
            ));
            max_width.max(galley.size().x)
        });
    
    // Add padding and set minimum width for better display
    let header_width = (longest_text_width + 30.0_f32).max(base_header_width);
    
    let matrix_width = header_width + (all_features.len() as f32 * cell_size);
    let matrix_height = header_height + (all_features.len() as f32 * cell_size);
    
    // Build or refresh the dependency map only when needed
    if state.dependency_map_cache.is_none() || state.dependency_map_cache_dirty {
        state.dependency_map_cache = Some(build_dependency_map(state));
        state.dependency_map_cache_dirty = false;
    }
    
    // Clone the dependency map to avoid borrowing issues
    let dependency_map = state.dependency_map_cache.as_ref().unwrap().clone();
    
    // Create a vector of clickable cells that we'll populate while drawing
    let mut clickable_cells: Vec<(egui::Rect, String, String, String, String)> = Vec::new();
    
    // Track if we need to show the modal
    let mut show_dependency_modal = false;
    let mut modal_info: Option<(String, String, String, String)> = None;
    
    // Outer frame with scrolling
    egui::Frame::none()
        .fill(ui.style().visuals.panel_fill)
        .show(ui, |ui| {
            // Add ScrollArea for both horizontal and vertical scrolling
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_size(egui::Vec2::new(matrix_width, matrix_height));
                    
                    // Draw the dependency matrix
                    let (rect, response) = ui.allocate_exact_size(
                        egui::Vec2::new(matrix_width, matrix_height),
                        egui::Sense::click_and_drag()
                    );
                    
                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();
                        
                        // Draw background
                        painter.rect_filled(
                            rect,
                            0.0,
                            ui.style().visuals.window_fill
                        );
                        
                        // Draw grid lines
                        let grid_color = ui.style().visuals.widgets.noninteractive.bg_stroke.color;
                        for i in 0..=all_features.len() {
                            // Horizontal lines
                            painter.line_segment(
                                [
                                    rect.left_top() + egui::Vec2::new(0.0, header_height + i as f32 * cell_size),
                                    rect.right_top() + egui::Vec2::new(0.0, header_height + i as f32 * cell_size)
                                ],
                                ui.style().visuals.widgets.noninteractive.bg_stroke
                            );
                            
                            // Vertical lines
                            painter.line_segment(
                                [
                                    rect.left_top() + egui::Vec2::new(header_width + i as f32 * cell_size, 0.0),
                                    rect.left_bottom() + egui::Vec2::new(header_width + i as f32 * cell_size, 0.0)
                                ],
                                ui.style().visuals.widgets.noninteractive.bg_stroke
                            );
                        }
                        
                        // Draw separator between headers and cells
                        painter.line_segment(
                            [
                                rect.left_top() + egui::Vec2::new(0.0, header_height),
                                rect.right_top() + egui::Vec2::new(0.0, header_height)
                            ],
                            egui::Stroke::new(2.0, ui.style().visuals.widgets.active.bg_stroke.color)
                        );
                        
                        painter.line_segment(
                            [
                                rect.left_top() + egui::Vec2::new(header_width, 0.0),
                                rect.left_bottom() + egui::Vec2::new(header_width, 0.0)
                            ],
                            egui::Stroke::new(2.0, ui.style().visuals.widgets.active.bg_stroke.color)
                        );
                        
                        // Draw row headers (vertical)
                        for (i, (comp_name, feat_name)) in all_features.iter().enumerate() {
                            let text_pos = rect.left_top() + 
                                egui::Vec2::new(10.0, header_height + i as f32 * cell_size + cell_size / 2.0);
                            
                            let header_text = format_feature_text(comp_name, feat_name);
                            let header_rect = egui::Rect::from_min_size(
                                rect.left_top() + egui::Vec2::new(0.0, header_height + i as f32 * cell_size),
                                egui::Vec2::new(header_width, cell_size)
                            );
                            
                            // Check for clicks on row headers
                            if response.clicked() && header_rect.contains(response.interact_pointer_pos().unwrap_or_default()) {
                                // Find the component and feature indices to navigate to
                                if let Some(comp_idx) = state.components.iter().position(|c| c.name == *comp_name) {
                                    state.selected_component = Some(comp_idx);
                                    if let Some(component) = state.components.get(comp_idx) {
                                        if let Some(feat_idx) = component.features.iter().position(|f| f.name == *feat_name) {
                                            state.selected_feature = Some(feat_idx);
                                        }
                                    }
                                    state.current_screen = Screen::Components;
                                }
                            }
                            
                            // Draw header text with hover effect
                            if header_rect.contains(ui.ctx().input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                                painter.rect_filled(
                                    header_rect,
                                    0.0,
                                    ui.style().visuals.widgets.hovered.bg_fill
                                );
                            }
                            
                            // Draw the full row text
                            let row_text = format_feature_text(comp_name, feat_name);
                            
                            painter.text(
                                text_pos,
                                egui::Align2::LEFT_CENTER,
                                row_text,
                                egui::FontId::default(),
                                ui.style().visuals.text_color()
                            );
                        }
                        
                        // Draw column headers (horizontal) with properly rotated text
                        for (i, (comp_name, feat_name)) in all_features.iter().enumerate() {
                            let header_rect = egui::Rect::from_min_size(
                                rect.left_top() + egui::Vec2::new(header_width + i as f32 * cell_size, 0.0),
                                egui::Vec2::new(cell_size, header_height)
                            );
                            
                            // Check for clicks on column headers
                            if response.clicked() && header_rect.contains(response.interact_pointer_pos().unwrap_or_default()) {
                                if let Some(comp_idx) = state.components.iter().position(|c| c.name == *comp_name) {
                                    state.selected_component = Some(comp_idx);
                                    if let Some(component) = state.components.get(comp_idx) {
                                        if let Some(feat_idx) = component.features.iter().position(|f| f.name == *feat_name) {
                                            state.selected_feature = Some(feat_idx);
                                        }
                                    }
                                    state.current_screen = Screen::Components;
                                }
                            }
                            
                            // Draw header background with hover effect
                            if header_rect.contains(ui.ctx().input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                                painter.rect_filled(
                                    header_rect,
                                    0.0,
                                    ui.style().visuals.widgets.hovered.bg_fill
                                );
                            }
                            
                            // Simulate rotated text by drawing it in two parts
                            let center_x = header_rect.center().x;
                            
                            // 1. Draw component name at top
                            painter.text(
                                egui::Pos2::new(center_x, header_rect.min.y + 20.0),
                                egui::Align2::CENTER_CENTER,
                                comp_name,
                                egui::FontId::proportional(10.0),
                                ui.style().visuals.text_color()
                            );
                            
                            // 2. Calculate dimensions for rotated feature name
                            let feature_text_width = ui.fonts(|f| f.layout_no_wrap(
                                feat_name.clone(), 
                                egui::FontId::proportional(10.0), 
                                ui.style().visuals.text_color()
                            )).size().x;
                            
                            // Draw a line to represent the text path
                            let start_y = header_rect.min.y + 40.0;
                            let end_y = start_y + feature_text_width.min(header_height - 50.0);
                            
                            // Draw feature name label at approximate angle
                            for (j, ch) in feat_name.chars().enumerate() {
                                let num_chars = feat_name.chars().count();
                                if j < 10 { // Limit to avoid overflow
                                    // Calculate position along the line
                                    let t = j as f32 / (num_chars - 1).max(1) as f32;
                                    let x = center_x;
                                    let y = start_y + t * (end_y - start_y);
                                    
                                    painter.text(
                                        egui::Pos2::new(x, y),
                                        egui::Align2::CENTER_CENTER,
                                        ch.to_string(),
                                        egui::FontId::proportional(10.0),
                                        ui.style().visuals.text_color()
                                    );
                                }
                            }
                        }
                        
                        // Draw matrix cells with dependency counts
                        for (row, (row_comp, row_feat)) in all_features.iter().enumerate() {
                            for (col, (col_comp, col_feat)) in all_features.iter().enumerate() {
                                let cell_rect = egui::Rect::from_min_size(
                                    rect.left_top() + egui::Vec2::new(
                                        header_width + col as f32 * cell_size,
                                        header_height + row as f32 * cell_size
                                    ),
                                    egui::Vec2::new(cell_size, cell_size)
                                );
                                
                                // Get dependency count
                                let key1 = ((row_comp.clone(), row_feat.clone()), (col_comp.clone(), col_feat.clone()));
                                let key2 = ((col_comp.clone(), col_feat.clone()), (row_comp.clone(), row_feat.clone()));
                                
                                let count = dependency_map.get(&key1).or_else(|| dependency_map.get(&key2)).copied().unwrap_or(0);
                                
                                // Draw cell content if there are dependencies
                                if count > 0 {
                                    // Color intensity based on count
                                    let intensity = (count.min(5) as f32 / 5.0 * 0.8 + 0.2).min(1.0);
                                    let cell_color = egui::Color32::from_rgba_premultiplied(
                                        (100.0 * intensity) as u8,
                                        (150.0 * intensity) as u8,
                                        (255.0 * intensity) as u8,
                                        200
                                    );
                                    
                                    painter.rect_filled(
                                        cell_rect,
                                        2.0,
                                        cell_color
                                    );
                                    
                                    // Draw count in cell
                                    painter.text(
                                        cell_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        count.to_string(),
                                        egui::FontId::default(),
                                        egui::Color32::WHITE
                                    );
                                    
                                    // Check for click on this cell
                                    if response.clicked() && cell_rect.contains(response.interact_pointer_pos().unwrap_or_default()) {
                                        show_dependency_modal = true;
                                        modal_info = Some((
                                            row_comp.clone(), 
                                            row_feat.clone(), 
                                            col_comp.clone(), 
                                            col_feat.clone()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                });
        });
    
    // Show modal if needed - outside of painter context to avoid borrowing issues
    if show_dependency_modal {
        if let Some((row_comp, row_feat, col_comp, col_feat)) = modal_info {
            show_dependency_details_modal(
                ui.ctx(),
                state,
                &row_comp,
                &row_feat,
                &col_comp,
                &col_feat
            );
        }
    }
}

// Moved to a separate function for cleaner organization
fn show_dependency_details_modal(
    ctx: &egui::Context,
    state: &mut AppState,
    row_comp: &str,
    row_feat: &str,
    col_comp: &str,
    col_feat: &str
) {
    // Find all mates and analyses that involve these two features
    let mut options = Vec::new();
    
    // Check for direct mates
    for (idx, mate) in state.mates.iter().enumerate() {
        if (mate.component_a == row_comp && mate.feature_a == row_feat &&
            mate.component_b == col_comp && mate.feature_b == col_feat) ||
           (mate.component_a == col_comp && mate.feature_a == col_feat &&
            mate.component_b == row_comp && mate.feature_b == row_feat) {
            options.push((format!("Mate: {}.{} ↔ {}.{}", 
                          mate.component_a, mate.feature_a, 
                          mate.component_b, mate.feature_b),
                         DependencyAction::GotoMate(idx)));
        }
    }
    
    // Check for analyses that include both features
    for (idx, analysis) in state.analyses.iter().enumerate() {
        let row_found = analysis.contributions.iter().any(|c| 
            c.component_id == row_comp && c.feature_id == row_feat);
        let col_found = analysis.contributions.iter().any(|c| 
            c.component_id == col_comp && c.feature_id == col_feat);
        
        if row_found && col_found {
            options.push((format!("Analysis: {}", analysis.name),
                         DependencyAction::GotoAnalysis(idx)));
        }
    }
    
    // Use modal dialog to ensure it stays visible
    let modal_id = egui::Id::new("dependency_relations_modal");
    
    egui::Window::new(format!("Relations: {}.{} ↔ {}.{}", row_comp, row_feat, col_comp, col_feat))
        .id(modal_id)
        .collapsible(false)
        .default_pos([200.0, 200.0])
        .default_size([300.0, 400.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Related Items");
                ui.separator();
                
                if options.is_empty() {
                    ui.label("No direct relations found.");
                } else {
                    for (label, action) in options {
                        if ui.button(label).clicked() {
                            match action {
                                DependencyAction::GotoMate(idx) => {
                                    state.selected_mate = Some(idx);
                                    state.current_screen = Screen::Mates;
                                    ui.ctx().memory_mut(|mem| mem.data.remove::<bool>(modal_id));
                                },
                                DependencyAction::GotoAnalysis(idx) => {
                                    state.selected_analysis = Some(idx);
                                    state.current_screen = Screen::Analysis;
                                    ui.ctx().memory_mut(|mem| mem.data.remove::<bool>(modal_id));
                                }
                            }
                        }
                    }
                }
                
                ui.add_space(10.0);
                if ui.button("Close").clicked() {
                    ui.ctx().memory_mut(|mem| mem.data.remove::<bool>(modal_id));
                }
            });
        });
}

// Helper function to build a map of dependencies and their counts
fn build_dependency_map(state: &AppState) -> HashMap<((String, String), (String, String)), usize> {
    // Build a new map each time
    let mut dependency_map: HashMap<((String, String), (String, String)), usize> = HashMap::new();
    
    // Add mate relationships
    for mate in &state.mates {
        let key = (
            (mate.component_a.clone(), mate.feature_a.clone()),
            (mate.component_b.clone(), mate.feature_b.clone())
        );
        *dependency_map.entry(key).or_insert(0) += 1;
    }
    
    // Add analysis relationships
    for analysis in &state.analyses {
        // Create a set of features in this analysis
        let mut analysis_features = HashSet::new();
        for contrib in &analysis.contributions {
            analysis_features.insert((contrib.component_id.clone(), contrib.feature_id.clone()));
        }
        
        // For each pair of features in the analysis, increment their relationship count
        let features: Vec<_> = analysis_features.iter().collect();
        for i in 0..features.len() {
            for j in (i+1)..features.len() {
                let key = (
                    (features[i].0.clone(), features[i].1.clone()),
                    (features[j].0.clone(), features[j].1.clone())
                );
                *dependency_map.entry(key).or_insert(0) += 1;
            }
        }
    }
    
    dependency_map
}

// Action to take when a dependency cell is clicked
enum DependencyAction {
    GotoMate(usize),
    GotoAnalysis(usize),
}