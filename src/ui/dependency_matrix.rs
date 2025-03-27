// src/ui/dependency_matrix.rs
use eframe::egui;
use petgraph::graph::{NodeIndex, EdgeIndex};
use std::collections::{HashMap, HashSet};
use crate::state::{AppState, Screen};
use crate::config::{Component, Feature};

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
    let header_width = 150.0;
    let header_height = 80.0;
    
    let matrix_width = header_width + (all_features.len() as f32 * cell_size);
    let matrix_height = header_height + (all_features.len() as f32 * cell_size);
    
    // Helper to format component.feature
    let format_feature = |comp: &str, feat: &str| -> String {
        format!("{}.{}", comp, feat)
    };
    
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
                            
                            let header_text = format_feature(comp_name, feat_name);
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
                            painter.text(
                                text_pos,
                                egui::Align2::LEFT_CENTER,
                                header_text,
                                egui::FontId::default(),
                                ui.style().visuals.text_color()
                            );
                        }
                        
                        // Draw column headers (horizontal)
                        for (i, (comp_name, feat_name)) in all_features.iter().enumerate() {
                            let header_text = format_feature(comp_name, feat_name);
                            let header_rect = egui::Rect::from_min_size(
                                rect.left_top() + egui::Vec2::new(header_width + i as f32 * cell_size, 0.0),
                                egui::Vec2::new(cell_size, header_height)
                            );
                            
                            // Check for clicks on column headers
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
                            
                            // Draw text for column headers - rotated
                            let center = header_rect.center();
                            
                            // Split the text into parts
                            let parts: Vec<&str> = header_text.split('.').collect();
                            if parts.len() == 2 {
                                // Draw component name and feature name separately for better readability
                                painter.text(
                                    center + egui::Vec2::new(0.0, -15.0),
                                    egui::Align2::CENTER_CENTER,
                                    parts[0],
                                    egui::FontId::proportional(10.0),
                                    ui.style().visuals.text_color()
                                );
                                
                                painter.text(
                                    center + egui::Vec2::new(0.0, 5.0),
                                    egui::Align2::CENTER_CENTER,
                                    parts[1],
                                    egui::FontId::proportional(10.0),
                                    ui.style().visuals.text_color()
                                );
                            } else {
                                // Fallback for unexpected format
                                painter.text(
                                    center,
                                    egui::Align2::CENTER_CENTER,
                                    &header_text,
                                    egui::FontId::proportional(10.0),
                                    ui.style().visuals.text_color()
                                );
                            }
                        }
                        
                        // Draw matrix cells with dependency counts
                        let dependency_map = build_dependency_map(state);
                        
                        // Collect cells to potentially handle clicks
                        let mut clickable_cells = Vec::new();
                        
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
                                    
                                    // Store this cell for potential clicks
                                    clickable_cells.push((
                                        cell_rect,
                                        row_comp.clone(),
                                        row_feat.clone(),
                                        col_comp.clone(),
                                        col_feat.clone()
                                    ));
                                }
                            }
                        }
                        
                        // Now handle clicks - this is outside the loop so we don't have multiple mutable borrows
                        if response.clicked() {
                            if let Some(click_pos) = response.interact_pointer_pos() {
                                for (cell_rect, row_comp, row_feat, col_comp, col_feat) in clickable_cells {
                                    if cell_rect.contains(click_pos) {
                                        handle_dependency_click(ui.ctx(), state, &row_comp, &row_feat, &col_comp, &col_feat);
                                        break;
                                    }
                                }
                            }
                        }
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

// Helper function to handle clicks on dependency cells
fn handle_dependency_click(
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
    
    // Show context menu with options
    if !options.is_empty() {
        egui::Area::new("dependency_context_menu")
            .order(egui::Order::Foreground)
            .fixed_pos(ctx.input(|i| i.pointer.hover_pos().unwrap_or_default()))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .show(ui, |ui| {
                        for (label, action) in options {
                            if ui.button(label).clicked() {
                                match action {
                                    DependencyAction::GotoMate(idx) => {
                                        state.selected_mate = Some(idx);
                                        state.current_screen = Screen::Mates;
                                    },
                                    DependencyAction::GotoAnalysis(idx) => {
                                        state.selected_analysis = Some(idx);
                                        state.current_screen = Screen::Analysis;
                                    }
                                }
                                ui.close_menu();
                            }
                        }
                    });
            });
    }
}

// Action to take when a dependency cell is clicked
enum DependencyAction {
    GotoMate(usize),
    GotoAnalysis(usize),
}