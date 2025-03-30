// src/ui/dependency_matrix.rs
use eframe::egui;
use petgraph::graph::{NodeIndex, EdgeIndex};
use std::collections::HashMap;
use std::collections::HashSet;
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
    
    // Calculate cell sizes and header sizes
    // First, measure each component and feature name to determine optimal cell widths
    let mut column_widths: Vec<f32> = Vec::with_capacity(all_features.len());
    
    for (comp_name, feat_name) in &all_features {
        // Measure component name width
        let comp_width = ui.fonts(|f| {
            f.layout_no_wrap(
                comp_name.clone(), 
                egui::FontId::default(), 
                ui.style().visuals.text_color()
            ).size().x
        }) + 20.0; // Add padding
        
        // Measure feature name width
        let feat_width = ui.fonts(|f| {
            f.layout_no_wrap(
                feat_name.clone(), 
                egui::FontId::default(), 
                ui.style().visuals.text_color()
            ).size().x
        }) + 20.0; // Add padding
        
        // Use the maximum width needed for this column
        let width = comp_width.max(feat_width);
        column_widths.push(width);
    }
    
    // Ensure a minimum width for each column
    const MIN_COLUMN_WIDTH: f32 = 50.0;
    for width in &mut column_widths {
        *width = width.max(MIN_COLUMN_WIDTH);
    }
    
    let base_header_width = 200.0;  // Width for row headers
    let column_header_height = 120.0; // Height for column headers
    
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
    
    // Calculate matrix dimensions using dynamic column widths
    let matrix_width = header_width + column_widths.iter().sum::<f32>();
    let matrix_height = column_header_height + (all_features.len() as f32 * 40.0); // Fixed row height of 40px
    
    // Build or refresh the dependency map only when needed
    if state.dependency_map_cache.is_none() || state.dependency_map_cache_dirty {
        state.dependency_map_cache = Some(build_dependency_map(state));
        state.dependency_map_cache_dirty = false;
    }
    
    // Clone the dependency map to avoid borrowing issues
    let dependency_map = state.dependency_map_cache.as_ref().unwrap().clone();
    
    // Use persistent IDs for modal state
    let modal_open_id = egui::Id::new("dependency_modal_open");
    let modal_info_id = egui::Id::new("dependency_modal_info");
    
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
                        
                        // Draw grid lines with dynamic column widths
                        let grid_color = ui.style().visuals.widgets.noninteractive.bg_stroke.color;
                        
                        // Draw horizontal grid lines
                        let row_height = 40.0;
                        for i in 0..=all_features.len() {
                            painter.line_segment(
                                [
                                    rect.left_top() + egui::Vec2::new(0.0, column_header_height + i as f32 * row_height),
                                    rect.right_top() + egui::Vec2::new(0.0, column_header_height + i as f32 * row_height)
                                ],
                                ui.style().visuals.widgets.noninteractive.bg_stroke
                            );
                        }
                        
                        // Draw vertical grid lines
                        let mut current_x = header_width;
                        for i in 0..=all_features.len() {
                            painter.line_segment(
                                [
                                    rect.left_top() + egui::Vec2::new(current_x, 0.0),
                                    rect.left_bottom() + egui::Vec2::new(current_x, 0.0)
                                ],
                                ui.style().visuals.widgets.noninteractive.bg_stroke
                            );
                            
                            if i < all_features.len() {
                                current_x += column_widths[i];
                            }
                        }
                        
                        // Draw separator between headers and cells - use dynamic width
                        painter.line_segment(
                            [
                                rect.left_top() + egui::Vec2::new(0.0, column_header_height),
                                rect.right_top() + egui::Vec2::new(0.0, column_header_height)
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
                            let row_height = 40.0; // Fixed row height
                            let text_pos = rect.left_top() + 
                                egui::Vec2::new(10.0, column_header_height + i as f32 * row_height + row_height / 2.0);
                            
                            let header_text = format_feature_text(comp_name, feat_name);
                            let header_rect = egui::Rect::from_min_size(
                                rect.left_top() + egui::Vec2::new(0.0, column_header_height + i as f32 * row_height),
                                egui::Vec2::new(header_width, row_height)
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
                        
                        // Draw column headers (horizontal) with dynamic widths
                        let mut current_x = header_width;
                        for (i, (comp_name, feat_name)) in all_features.iter().enumerate() {
                            let column_width = column_widths[i];
                            
                            // Calculate header cell
                            let header_rect = egui::Rect::from_min_size(
                                rect.left_top() + egui::Vec2::new(current_x, 0.0),
                                egui::Vec2::new(column_width, column_header_height)
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
                            
                            // Draw component name at the top
                            let comp_pos = egui::Pos2::new(
                                header_rect.center().x, 
                                header_rect.min.y + 30.0
                            );
                            
                            // Show full component name without truncation
                            painter.text(
                                comp_pos,
                                egui::Align2::CENTER_CENTER,
                                comp_name.clone(),
                                egui::FontId::default(),
                                ui.style().visuals.text_color()
                            );
                            
                            // Calculate position and text for feature
                            let feat_pos = egui::Pos2::new(
                                header_rect.center().x,
                                header_rect.min.y + 80.0
                            );
                            
                            // Display full feature name without truncation
                            let display_feat = feat_name.clone();
                            
                            painter.text(
                                feat_pos,
                                egui::Align2::CENTER_CENTER,
                                display_feat,
                                egui::FontId::default(),
                                ui.style().visuals.text_color()
                            );
                            
                            // Draw separator line between component and feature
                            painter.line_segment(
                                [
                                    egui::Pos2::new(header_rect.min.x, header_rect.min.y + 55.0),
                                    egui::Pos2::new(header_rect.max.x, header_rect.min.y + 55.0)
                                ],
                                egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color)
                            );
                            
                            // Update current_x for next column
                            current_x += column_width;
                        }
                        
                        // Draw matrix cells with dependency counts - using dynamic column widths
                        let mut current_x = header_width;
                        for (col, (col_comp, col_feat)) in all_features.iter().enumerate() {
                            let column_width = column_widths[col];
                            
                            for (row, (row_comp, row_feat)) in all_features.iter().enumerate() {
                                let row_height = 40.0; // Fixed row height
                                let cell_rect = egui::Rect::from_min_size(
                                    rect.left_top() + egui::Vec2::new(
                                        current_x,
                                        column_header_height + row as f32 * row_height
                                    ),
                                    egui::Vec2::new(column_width, row_height)
                                );
                                
                                // Get dependency count - use consistent key ordering
                                let (first, second) = if row_comp < col_comp || 
                                                     (row_comp == col_comp && row_feat < col_feat) {
                                    (
                                        (row_comp.clone(), row_feat.clone()),
                                        (col_comp.clone(), col_feat.clone())
                                    )
                                } else {
                                    (
                                        (col_comp.clone(), col_feat.clone()),
                                        (row_comp.clone(), row_feat.clone())
                                    )
                                };
                                
                                let key = (first, second);
                                let count = dependency_map.get(&key).copied().unwrap_or(0);
                                
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
                                        // Store modal state in egui memory
                                        ui.ctx().memory_mut(|mem| {
                                            mem.data.insert_temp(modal_open_id, true);
                                            mem.data.insert_temp(
                                                modal_info_id, 
                                                (row_comp.clone(), row_feat.clone(), col_comp.clone(), col_feat.clone())
                                            );
                                        });
                                    }
                                }
                            }
                            
                            // Move to next column
                            current_x += column_width;
                        }
                    }
                });
        });
    
    // Show modal if needed - use memory to persist between frames
    let show_modal = ui.ctx().memory(|mem| mem.data.get_temp::<bool>(modal_open_id).unwrap_or(false));
    if show_modal {
        if let Some(modal_info) = ui.ctx().memory(|mem| 
            mem.data.get_temp::<(String, String, String, String)>(modal_info_id)
        ) {
            let (row_comp, row_feat, col_comp, col_feat) = modal_info;
            show_dependency_details_modal(ui.ctx(), state, &row_comp, &row_feat, &col_comp, &col_feat, modal_open_id);
        }
    }
}

// Modified to accept the modal_id param for state management
fn show_dependency_details_modal(
    ctx: &egui::Context,
    state: &mut AppState,
    row_comp: &str,
    row_feat: &str,
    col_comp: &str,
    col_feat: &str,
    modal_open_id: egui::Id,
) {
    // Find all mates and analyses that involve these two features
    let mut options = Vec::new();
    
    // Check for direct mates - Fix duplicate entries issue
    let mut seen_mate_ids = HashSet::new();
    for (idx, mate) in state.mates.iter().enumerate() {
        // Check if this mate relates the two selected features
        if (mate.component_a == row_comp && mate.feature_a == row_feat &&
            mate.component_b == col_comp && mate.feature_b == col_feat) ||
           (mate.component_a == col_comp && mate.feature_a == col_feat &&
            mate.component_b == row_comp && mate.feature_b == row_feat) {
            
            // Only add if we haven't seen this mate before
            if seen_mate_ids.insert(mate.id.clone()) {
                options.push((format!("Mate: {}.{} ↔ {}.{}", 
                            mate.component_a, mate.feature_a, 
                            mate.component_b, mate.feature_b),
                           DependencyAction::GotoMate(idx)));
            }
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
    
    // Generate a consistent modal ID
    let modal_id = egui::Id::new("dependency_relations_modal");
    
    // Use modal dialog without a header
    egui::Window::new("") // Empty title to remove header
        .id(modal_id)
        .collapsible(false)
        .title_bar(false) // Remove title bar completely
        .fixed_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("Relations: {}.{} ↔ {}.{}", 
                    row_comp, row_feat, col_comp, col_feat));
                ui.separator();
                
                if options.is_empty() {
                    ui.label("No direct relations found.");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (label, action) in options {
                            if ui.button(label).clicked() {
                                match action {
                                    DependencyAction::GotoMate(idx) => {
                                        state.selected_mate = Some(idx);
                                        state.current_screen = Screen::Mates;
                                        // Close the modal by clearing memory
                                        ctx.memory_mut(|mem| mem.data.remove::<bool>(modal_open_id));
                                    },
                                    DependencyAction::GotoAnalysis(idx) => {
                                        state.selected_analysis = Some(idx);
                                        state.current_screen = Screen::Analysis;
                                        // Close the modal by clearing memory
                                        ctx.memory_mut(|mem| mem.data.remove::<bool>(modal_open_id));
                                    }
                                }
                            }
                        }
                    });
                }
                
                ui.add_space(10.0);
                if ui.button("Close").clicked() {
                    // Close the modal by clearing memory
                    ctx.memory_mut(|mem| mem.data.remove::<bool>(modal_open_id));
                }
            });
        });
}

// Helper function to build a map of dependencies and their counts
fn build_dependency_map(state: &AppState) -> HashMap<((String, String), (String, String)), usize> {
    // Build a new map each time
    let mut dependency_map: HashMap<((String, String), (String, String)), usize> = HashMap::new();
    
    // Add mate relationships - ensure consistent key ordering
    for mate in &state.mates {
        // Always order keys consistently to avoid duplicate entries
        let (first, second) = if mate.component_a < mate.component_b || 
                              (mate.component_a == mate.component_b && mate.feature_a < mate.feature_b) {
            (
                (mate.component_a.clone(), mate.feature_a.clone()),
                (mate.component_b.clone(), mate.feature_b.clone())
            )
        } else {
            (
                (mate.component_b.clone(), mate.feature_b.clone()),
                (mate.component_a.clone(), mate.feature_a.clone())
            )
        };
        
        let key = (first, second);
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
                // Order keys consistently
                let (first, second) = if features[i].0 < features[j].0 || 
                                     (features[i].0 == features[j].0 && features[i].1 < features[j].1) {
                    (
                        (features[i].0.clone(), features[i].1.clone()),
                        (features[j].0.clone(), features[j].1.clone())
                    )
                } else {
                    (
                        (features[j].0.clone(), features[j].1.clone()),
                        (features[i].0.clone(), features[i].1.clone())
                    )
                };
                
                let key = (first, second);
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