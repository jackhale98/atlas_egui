// src/ui/analysis.rs
use eframe::egui;
use crate::app::App;
use crate::state::ui_state::{DialogMode, AnalysisTab};
use crate::analysis::stackup::{AnalysisMethod, MonteCarloResult, StackupAnalysis, AnalysisResults, DistributionType, StackupContribution, DistributionParams};
use crate::config::Component;
use crate::ui::dialog::DialogState;
use chrono::DateTime;
use egui_plot::{Plot, Bar, BarChart};
use tui_input::Input;

pub fn draw_analysis_view(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    // Tab selection
    ui.horizontal(|ui| {
        let tabs = [
            (AnalysisTab::List, "List"),
            (AnalysisTab::Details, "Details"),
            (AnalysisTab::Results, "Results"),
            (AnalysisTab::Visualization, "Visualization"),
        ];

        for (tab, label) in tabs {
            if ui.selectable_label(app.state.ui.analysis_tab == tab, label).clicked() {
                app.state.ui.analysis_tab = tab;
            }
        }
    });

    ui.add_space(8.0);

    // Main content area
    match app.state.ui.analysis_tab {
        AnalysisTab::List => draw_analysis_list(ui, app),
        AnalysisTab::Details => draw_analysis_details(ui, app),
        AnalysisTab::Results => draw_analysis_results(ui, app),
        AnalysisTab::Visualization => draw_analysis_visualization(ui, app),
    }

    // Draw contribution modal if in contribution mode
    if matches!(app.state.ui.dialog_mode, DialogMode::AddContribution | DialogMode::EditContribution) {
        draw_contribution_modal(ui.ctx(), app, dialog_state);
    }
}

// Rest of the implementation remains the same as in the previous artifact
fn draw_contribution_dialog(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    let mut can_save = false;
    
    ui.heading(match app.state.ui.dialog_mode {
        DialogMode::AddContribution => "Add Contribution",
        DialogMode::EditContribution => "Edit Contribution",
        _ => "Contribution",
    });
    
    // Component selection
    ui.horizontal(|ui| {
        ui.label("Component:");
        egui::ComboBox::from_label("Select Component")
            .selected_text(app.state.input.contribution_inputs.selected_component.clone())
            .show_ui(ui, |ui| {
                for component in &app.state.project.components {
                    ui.selectable_value(
                        &mut app.state.input.contribution_inputs.selected_component,
                        component.name.clone(),
                        &component.name
                    );
                }
            });
    });

    // Feature selection (only show if a component is selected)
    if !app.state.input.contribution_inputs.selected_component.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Feature:");
            
            // Find the selected component
            if let Some(component) = app.state.project.components.iter()
                .find(|c| c.name == app.state.input.contribution_inputs.selected_component) {
                
                // Check if we have features
                if !component.features.is_empty() {
                    egui::ComboBox::from_label("Select Feature")
                        .selected_text(app.state.input.contribution_inputs.selected_feature.clone())
                        .show_ui(ui, |ui| {
                            for feature in &component.features {
                                ui.selectable_value(
                                    &mut app.state.input.contribution_inputs.selected_feature,
                                    feature.name.clone(),
                                    &feature.name
                                );
                            }
                        });
                } else {
                    ui.label("No features available");
                }
            }
        });
    }

    // Validate feature selection
    can_save = !app.state.input.contribution_inputs.selected_component.is_empty() 
        && !app.state.input.contribution_inputs.selected_feature.is_empty();

    // Direction selection
    ui.horizontal(|ui| {
        ui.label("Direction:");
        let mut positive = app.state.input.contribution_inputs.direction > 0.0;
        if ui.radio_value(&mut positive, true, "Positive").changed() ||
           ui.radio_value(&mut positive, false, "Negative").changed() {
            app.state.input.contribution_inputs.direction = if positive { 1.0 } else { -1.0 };
        }
    });

    // Half count checkbox
    ui.checkbox(
        &mut app.state.input.contribution_inputs.half_count, 
        "Half Count"
    );

    // Distribution type selection
    ui.horizontal(|ui| {
        ui.label("Distribution:");
        let mut dist_type = app.state.input.contribution_inputs.distribution_type;
        
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", dist_type))
            .show_ui(ui, |ui| {
                for dtype in [DistributionType::Normal, DistributionType::Uniform, 
                              DistributionType::Triangular, DistributionType::LogNormal] {
                    ui.selectable_value(&mut dist_type, dtype, format!("{:?}", dtype));
                }
                app.state.input.contribution_inputs.distribution_type = dist_type;
            });
    });

    // Action buttons
    ui.horizontal(|ui| {
        // Save button
        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
            save_contribution(app);
            app.state.ui.dialog_mode = DialogMode::None;
        }

        // Cancel button
        if ui.button("Cancel").clicked() {
            app.state.ui.dialog_mode = DialogMode::None;
        }
    });
}

/// This function draws the modal dialog for adding or editing contributions
fn draw_contribution_modal(ctx: &egui::Context, app: &mut App, dialog_state: &mut DialogState) {
    egui::Window::new("Contribution")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            draw_contribution_dialog(ui, app, dialog_state);
        });
}

fn save_contribution(app: &mut App) {
    if let Some(selected) = app.state.ui.analysis_list_state.selected() {
        if let Some(analysis) = app.state.analysis.analyses.get_mut(selected) {
            // Find the selected component and feature
            let component_id = app.state.input.contribution_inputs.selected_component.clone();
            let feature_id = app.state.input.contribution_inputs.selected_feature.clone();

            // Fetch the feature to use as default for distribution params
            let component = app.state.project.components.iter()
                .find(|c| c.name == component_id);

            let feature = component.and_then(|comp| 
                comp.features.iter().find(|f| f.name == feature_id)
            );

            // Create the contribution
            let contribution = StackupContribution {
                component_id,
                feature_id,
                direction: app.state.input.contribution_inputs.direction,
                half_count: app.state.input.contribution_inputs.half_count,
                distribution: match app.state.input.contribution_inputs.distribution_type {
                    DistributionType::Normal => {
                        let mean = feature.map(|f| f.dimension.value).unwrap_or(0.0);
                        let std_dev = feature.map(|f| 
                            (f.dimension.plus_tolerance + f.dimension.minus_tolerance) / 6.0
                        ).unwrap_or(0.01);
                        Some(DistributionParams::new_normal(mean, std_dev))
                    },
                    DistributionType::Uniform => {
                        let feature = feature.unwrap_or_else(|| &feature.as_ref().unwrap());
                        Some(DistributionParams::new_uniform(
                            feature.dimension.value - feature.dimension.minus_tolerance,
                            feature.dimension.value + feature.dimension.plus_tolerance
                        ))
                    },
                    DistributionType::Triangular => {
                        let feature = feature.unwrap_or_else(|| &feature.as_ref().unwrap());
                        Some(DistributionParams::new_triangular(
                            feature.dimension.value - feature.dimension.minus_tolerance,
                            feature.dimension.value + feature.dimension.plus_tolerance,
                            feature.dimension.value
                        ))
                    },
                    DistributionType::LogNormal => {
                        let mean = feature.map(|f| f.dimension.value).unwrap_or(0.0);
                        let std_dev = feature.map(|f| 
                            (f.dimension.plus_tolerance + f.dimension.minus_tolerance) / 6.0
                        ).unwrap_or(0.01);
                        Some(DistributionParams::new_lognormal(mean, std_dev))
                    }
                },
            };

            // Determine if we're adding or editing
            match app.state.ui.dialog_mode {
                DialogMode::AddContribution => {
                    // Add new contribution
                    analysis.contributions.push(contribution);
                },
                DialogMode::EditContribution => {
                    // Replace an existing contribution
                    let edit_idx = analysis.contributions.iter()
                        .position(|c| 
                            c.component_id == app.state.input.contribution_inputs.selected_component &&
                            c.feature_id == app.state.input.contribution_inputs.selected_feature
                        );
                    
                    if let Some(idx) = edit_idx {
                        analysis.contributions[idx] = contribution;
                    }
                },
                _ => {}
            }
        }
    }

    // Reset dialog mode
    app.state.ui.dialog_mode = DialogMode::None;
}

fn draw_analysis_list(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.heading("Analyses");
        if ui.button("➕ Add Analysis").clicked() {
            let new_analysis = StackupAnalysis::new("New Analysis".to_string());
            app.state.analysis.analyses.push(new_analysis);
            let idx = app.state.analysis.analyses.len() - 1;
            app.state.ui.analysis_list_state.select(Some(idx));
            app.state.ui.dialog_mode = DialogMode::EditAnalysis;
        }
    });

    ui.add_space(8.0);
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        let mut delete_index = None;

        for (idx, analysis) in app.state.analysis.analyses.iter().enumerate() {
            let methods_str = analysis.methods.iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ");

            let selected = app.state.ui.analysis_list_state.selected() == Some(idx);
            
            ui.horizontal(|ui| {
                if ui.selectable_label(selected, 
                    format!("{} ({} contributions)\n{}", 
                        analysis.name, 
                        analysis.contributions.len(),
                        methods_str
                    )
                ).clicked() {
                    app.state.ui.analysis_list_state.select(Some(idx));
                }
                
                ui.menu_button("⋯", |ui| {
                    if ui.button("Edit").clicked() {
                        app.state.ui.dialog_mode = DialogMode::EditAnalysis;
                        ui.close_menu();
                    }
                    if ui.button("Run").clicked() {
                        if let Some(analysis) = app.state.analysis.analyses.get(idx) {
                            let results = analysis.run_analysis(&app.state.project.components);
                            app.state.analysis.latest_results.insert(analysis.id.clone(), results);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Delete").clicked() {
                        delete_index = Some(idx);
                        ui.close_menu();
                    }
                });
            });
        }

        if let Some(idx) = delete_index {
            app.state.analysis.analyses.remove(idx);
            if app.state.analysis.analyses.is_empty() {
                app.state.ui.analysis_list_state.select(None);
            } else if idx >= app.state.analysis.analyses.len() {
                app.state.ui.analysis_list_state.select(Some(app.state.analysis.analyses.len() - 1));
            }
        }
    });
}

fn draw_analysis_details(ui: &mut egui::Ui, app: &mut App) {
    if let Some(selected) = app.state.ui.analysis_list_state.selected() {
        if let Some(analysis) = app.state.analysis.analyses.get_mut(selected) {
            // Analysis name
            ui.horizontal(|ui| {
                ui.heading(&analysis.name);
                if ui.button("✏").clicked() {
                    app.state.ui.dialog_mode = DialogMode::EditAnalysis;
                }
            });

            ui.add_space(8.0);

            // Methods
            ui.group(|ui| {
                ui.heading("Methods");
                for method in &[AnalysisMethod::WorstCase, AnalysisMethod::Rss, AnalysisMethod::MonteCarlo] {
                    let mut enabled = analysis.methods.contains(method);
                    if ui.checkbox(&mut enabled, format!("{:?}", method)).changed() {
                        if enabled {
                            analysis.methods.push(*method);
                        } else {
                            analysis.methods.retain(|m| m != method);
                        }
                    }
                }
            });

            // Monte Carlo settings
            if analysis.methods.contains(&AnalysisMethod::MonteCarlo) {
                ui.group(|ui| {
                    ui.heading("Monte Carlo Settings");
                    let settings = analysis.monte_carlo_settings.get_or_insert_with(Default::default);
                    
                    ui.horizontal(|ui| {
                        ui.label("Iterations:");
                        let mut iterations = settings.iterations.to_string();
                        if ui.text_edit_singleline(&mut iterations).changed() {
                            if let Ok(value) = iterations.parse() {
                                settings.iterations = value;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Confidence (%):");
                        let mut confidence = (settings.confidence * 100.0).to_string();
                        if ui.text_edit_singleline(&mut confidence).changed() {
                            if let Ok(value) = confidence.parse::<f64>() {
                                settings.confidence = value / 100.0;
                            }
                        }
                    });
                });
            }

            // Contributions
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Contributions");
                    if ui.button("Add").clicked() {
                        // Reset contribution inputs
                        app.state.input.contribution_inputs = Default::default();
                        
                        // Clear any previous selections
                        app.state.input.contribution_inputs.selected_component = String::new();
                        app.state.input.contribution_inputs.selected_feature = String::new();
                        
                        // Set dialog mode
                        app.state.ui.dialog_mode = DialogMode::AddContribution;
                    }
                });

                                    egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut remove_idx = None;
                    let mut edit_idx = None;
                    for (idx, contrib) in analysis.contributions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Edit button
                            if ui.button("✏").clicked() {
                                // Prepare inputs for editing
                                app.state.input.contribution_inputs.selected_component = contrib.component_id.clone();
                                app.state.input.contribution_inputs.selected_feature = contrib.feature_id.clone();
                                app.state.input.contribution_inputs.direction = contrib.direction;
                                app.state.input.contribution_inputs.half_count = contrib.half_count;
                                
                                // Set distribution type if distribution exists
                                if let Some(dist) = &contrib.distribution {
                                    app.state.input.contribution_inputs.distribution_type = dist.dist_type;
                                }

                                // Set dialog mode for editing
                                app.state.ui.dialog_mode = DialogMode::EditContribution;
                                edit_idx = Some(idx);
                            }

                            // Delete button
                            if ui.button("🗑").clicked() {
                                remove_idx = Some(idx);
                            }

                            // Contribution details
                            ui.label(format!(
                                "{}.{} ({}{}) {}",
                                contrib.component_id,
                                contrib.feature_id,
                                if contrib.direction > 0.0 { "+" } else { "-" },
                                if contrib.half_count { "½" } else { "1" },
                                contrib.distribution.as_ref().map_or(
                                    "Normal".to_string(), 
                                    |d| format!("{:?}", d.dist_type)
                                )
                            ));
                        });
                    }

                    // Handle removal of contribution
                    if let Some(idx) = remove_idx {
                        analysis.contributions.remove(idx);
                    }

                    // Note: Editing is handled by the dialog mode now
                });
            });
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select an analysis to view details");
        });
    }
}

fn draw_analysis_results(ui: &mut egui::Ui, app: &mut App) {
    if let Some(selected) = app.state.ui.analysis_list_state.selected() {
        if let Some(analysis) = app.state.analysis.analyses.get(selected) {
            if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
                // Results header
                ui.heading(&analysis.name);
                ui.label(format!("Nominal: {:.6}", results.nominal));

                ui.add_space(8.0);

                // Results by method
                if let Some(wc) = &results.worst_case {
                    ui.group(|ui| {
                        ui.heading("Worst Case Analysis");
                        ui.label(format!("Min: {:.6}", wc.min));
                        ui.label(format!("Max: {:.6}", wc.max));
                        ui.label(format!("Range: {:.6}", wc.max - wc.min));
                    });
                }

                if let Some(rss) = &results.rss {
                    ui.group(|ui| {
                        ui.heading("RSS Analysis");
                        ui.label(format!("Mean: {:.6}", results.nominal));
                        ui.label(format!("Min (3σ): {:.6}", rss.min));
                        ui.label(format!("Max (3σ): {:.6}", rss.max));
                        ui.label(format!("Std Dev: {:.6}", rss.std_dev));
                    });
                }

                if let Some(mc) = &results.monte_carlo {
                    ui.group(|ui| {
                        ui.heading("Monte Carlo Analysis");
                        ui.label(format!("Mean: {:.6}", mc.mean));
                        ui.label(format!("Std Dev: {:.6}", mc.std_dev));
                        ui.label(format!("Min: {:.6}", mc.min));
                        ui.label(format!("Max: {:.6}", mc.max));

                        if !mc.confidence_intervals.is_empty() {
                            ui.separator();
                            ui.label("Confidence Intervals:");
                            for interval in &mc.confidence_intervals {
                                ui.label(format!(
                                    "{:.1}%: [{:.6}, {:.6}]",
                                    interval.confidence_level * 100.0,
                                    interval.lower_bound,
                                    interval.upper_bound
                                ));
                            }
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Run analysis to see results");
                });
            }
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select an analysis to view results");
        });
    }
}

fn draw_analysis_visualization(ui: &mut egui::Ui, app: &mut App) {
    if let Some(selected) = app.state.ui.analysis_list_state.selected() {
        if let Some(analysis) = app.state.analysis.analyses.get(selected) {
            if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
                let mut running_total = 0.0;
                let mut bars = Vec::new();

                // Create bars for each contribution
                for (i, contrib) in analysis.contributions.iter().enumerate() {
                    let value = contrib.direction * 
                        (if let Some(feat) = analysis.get_feature(&app.state.project.components, contrib) {
                            feat.dimension.value * if contrib.half_count { 0.5 } else { 1.0 }
                        } else {
                            0.0
                        });

                    running_total += value;
                    
                    bars.push(Bar::new(i as f64, value)
                        .fill(if value >= 0.0 { 
                            egui::Color32::from_rgb(100, 200, 100)
                        } else {
                            egui::Color32::from_rgb(200, 100, 100)
                        }));
                }

                Plot::new("contribution_waterfall")
                    .height(200.0)
                    .show(ui, |plot_ui| {
                        let chart = BarChart::new(bars)
                            .name("Contributions");
                        
                        plot_ui.bar_chart(chart);
                    });

                ui.label(format!("Total: {:.6}", running_total));

                // If Monte Carlo results are available, show confidence intervals
                if let Some(mc) = &results.monte_carlo {
                    ui.group(|ui| {
                        ui.heading("Monte Carlo Statistics");
                        ui.label(format!("Mean: {:.6}", mc.mean));
                        ui.label(format!("Std Dev: {:.6}", mc.std_dev));
                        ui.label(format!("Min: {:.6}", mc.min));
                        ui.label(format!("Max: {:.6}", mc.max));

                        // Display confidence intervals
                        if !mc.confidence_intervals.is_empty() {
                            ui.separator();
                            ui.label("Confidence Intervals:");
                            for interval in &mc.confidence_intervals {
                                ui.label(format!(
                                    "{:.1}%: [{:.6}, {:.6}]",
                                    interval.confidence_level * 100.0,
                                    interval.lower_bound,
                                    interval.upper_bound
                                ));
                            }
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Run analysis to see visualization");
                });
            }
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select an analysis to view visualization");
        });
    }
}