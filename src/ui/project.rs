// src/ui/project.rs
use eframe::egui;
use crate::app::App;
use crate::config::Units;
use chrono::prelude::*;

pub fn draw_project_view(ui: &mut egui::Ui, app: &mut App) {
    let state = &mut app.state;
    
    // Project Details Section
    ui.group(|ui| {
        ui.set_min_height(120.0);
        ui.heading("Project Details");
        ui.add_space(8.0);
        
        // Project name with edit
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.add_sized(
                [ui.available_width(), 20.0],
                egui::TextEdit::singleline(&mut state.project.project_file.name)
                    .hint_text("Enter project name")
            );
        });
        
        // Description with edit
        ui.horizontal(|ui| {
            ui.label("Description:");
            let desc = state.project.project_file.description.get_or_insert_with(String::new);
            ui.add_sized(
                [ui.available_width(), 60.0],
                egui::TextEdit::multiline(desc)
                    .hint_text("Enter project description")
            );
        });
        
        // Units selection
        ui.horizontal(|ui| {
            ui.label("Units:");
            ui.radio_value(
                &mut state.project.project_file.units,
                Units::Metric,
                "Metric (mm)"
            );
            ui.radio_value(
                &mut state.project.project_file.units,
                Units::Imperial,
                "Imperial (in)"
            );
        });
    });
    
    ui.add_space(16.0);

    // Project location (read-only)
    if let Some(dir) = &state.project.project_dir {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Project Directory:");
                ui.label(dir.to_string_lossy().as_ref());
            });
        });
    }
    
    ui.add_space(16.0);

    // Statistics Overview - using horizontal layout for main categories
    ui.horizontal(|ui| {
        // Components Card
        ui.group(|ui| {
            ui.set_min_width(ui.available_width() / 3.0);
            ui.vertical(|ui| {
                ui.heading("Components");
                ui.add_space(8.0);
                
                let total_components = state.project.components.len();
                let total_features: usize = state.project.components
                    .iter()
                    .map(|c| c.features.len())
                    .sum();
                
                ui.strong(format!("Total Components: {}", total_components));
                ui.strong(format!("Total Features: {}", total_features));
                
                if total_components > 0 {
                    ui.label(format!("Average Features per Component: {:.1}", 
                        total_features as f64 / total_components as f64));
                }
                
                // Add components list preview if space allows
                if total_components > 0 {
                    ui.add_space(8.0);
                    ui.label("Recent Components:");
                    for component in state.project.components.iter().take(3) {
                        ui.label(format!("• {} ({} features)", 
                            component.name, 
                            component.features.len()));
                    }
                }
            });
        });

        // Mates Card
        ui.group(|ui| {
            ui.set_min_width(ui.available_width() / 2.0);
            ui.vertical(|ui| {
                ui.heading("Mates");
                ui.add_space(8.0);
                
                let total_mates = state.mates.mates.len();
                let valid_mates = state.mates.mates.iter()
                    .filter(|mate| {
                        if let (Some(feat_a), Some(feat_b)) = (
                            find_feature(state, &mate.component_a, &mate.feature_a),
                            find_feature(state, &mate.component_b, &mate.feature_b)
                        ) {
                            mate.validate(feat_a, feat_b).is_valid
                        } else {
                            false
                        }
                    })
                    .count();
                
                ui.strong(format!("Total Mates: {}", total_mates));
                ui.strong(format!("Valid Mates: {}", valid_mates));
                
                if total_mates > 0 {
                    let validity_percentage = (valid_mates as f64 / total_mates as f64 * 100.0).round();
                    // Show validity percentage with color based on health
                    let color = if validity_percentage > 90.0 {
                        egui::Color32::GREEN
                    } else if validity_percentage > 70.0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };
                    
                    ui.colored_label(color, 
                        format!("Mate Validity: {}%", validity_percentage));
                }
            });
        });
    });
    
    ui.add_space(16.0);

    // Analysis Results Section
    ui.group(|ui| {
        ui.heading("Analysis Overview");
        ui.add_space(8.0);
        
        ui.horizontal(|ui| {
            // Analysis Statistics
            ui.vertical(|ui| {
                let total_analyses = state.analysis.analyses.len();
                let total_monte_carlo: usize = state.analysis.analyses.iter()
                    .filter_map(|analysis| analysis.monte_carlo_settings.as_ref())
                    .map(|settings| settings.iterations)
                    .sum();
                
                ui.strong(format!("Total Analyses: {}", total_analyses));
                
                if total_monte_carlo > 0 {
                    ui.strong(format!("Total Monte Carlo Iterations: {}", total_monte_carlo));
                }
            });

            ui.separator();

            // Latest Results
            ui.vertical(|ui| {
                let latest_result = state.analysis.latest_results.values()
                    .max_by_key(|r| DateTime::parse_from_rfc3339(&r.timestamp).ok());
                
                if let Some(result) = latest_result {
                    if let Ok(timestamp) = DateTime::parse_from_rfc3339(&result.timestamp) {
                        ui.label(format!("Last Analysis: {}", 
                            timestamp.format("%Y-%m-%d %H:%M:%S")));
                    }
                    
                    if let Some(mc) = &result.monte_carlo {
                        ui.strong(format!("Latest Mean: {:.6}", mc.mean));
                        ui.strong(format!("Latest Std Dev: {:.6}", mc.std_dev));
                        
                        // Show confidence intervals if available
                        if !mc.confidence_intervals.is_empty() {
                            ui.add_space(4.0);
                            ui.label("Confidence Intervals:");
                            for interval in &mc.confidence_intervals {
                                ui.label(format!("{:.1}%: [{:.6}, {:.6}]",
                                    interval.confidence_level * 100.0,
                                    interval.lower_bound,
                                    interval.upper_bound));
                            }
                        }
                    }
                } else {
                    ui.label("No analyses run yet");
                }
            });
        });
    });
}

fn find_feature<'a>(
    state: &'a crate::state::AppState,
    component_name: &str,
    feature_name: &str,
) -> Option<&'a crate::config::Feature> {
    state.project.components
        .iter()
        .find(|c| c.name == component_name)?
        .features
        .iter()
        .find(|f| f.name == feature_name)
}