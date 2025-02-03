// src/ui/analysis.rs
use eframe::egui;
use crate::app::App;
use crate::state::ui_state::{DialogMode, AnalysisTab};
use crate::state::input_state::{InputMode, EditField};
use crate::analysis::stackup::{
    AnalysisMethod, MonteCarloResult, StackupAnalysis, AnalysisResults,
    StackupContribution, DistributionType, DistributionParams
};
use crate::ui::dialog::DialogState;
use crate::config::Feature;

pub fn draw_analysis_view(ui: &mut egui::Ui, app: &mut App, _dialog_state: &mut DialogState) {
    let available_height = ui.available_height();
    
    // Top panel for analysis list (40% height)
    egui::TopBottomPanel::top("analysis_list_panel")
        .exact_height(available_height * 0.4)
        .show_inside(ui, |ui| {
            draw_analysis_list(ui, app);
        });

    // Main panel for details/results/visualization
    egui::CentralPanel::default().show_inside(ui, |ui| {
        let selected_analysis = app.state.ui.analysis_list_state.selected()
            .and_then(|idx| app.state.analysis.analyses.get(idx).cloned());

        if let Some(analysis) = selected_analysis {
            // Top tabs for switching views
            egui::TopBottomPanel::top("analysis_tabs")
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(app.state.ui.analysis_tab == AnalysisTab::Details, "Details").clicked() {
                            app.state.ui.analysis_tab = AnalysisTab::Details;
                        }
                        if ui.selectable_label(app.state.ui.analysis_tab == AnalysisTab::Results, "Results").clicked() {
                            app.state.ui.analysis_tab = AnalysisTab::Results;
                        }
                        if ui.selectable_label(app.state.ui.analysis_tab == AnalysisTab::Visualization, "Visualization").clicked() {
                            app.state.ui.analysis_tab = AnalysisTab::Visualization;
                        }
                    });
                });

            // Content based on selected tab
            match app.state.ui.analysis_tab {
                AnalysisTab::Details => draw_analysis_details(ui, app, &analysis),
                AnalysisTab::Results => draw_analysis_results(ui, app, &analysis),
                AnalysisTab::Visualization => draw_analysis_visualization(ui, app, &analysis),
                _ => {}
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an analysis to view details");
            });
        }
    });

    // Handle dialogs
    match app.state.ui.dialog_mode {
        DialogMode::AddAnalysis | DialogMode::EditAnalysis => {
            draw_analysis_dialog(ui.ctx(), app);
        },
        DialogMode::AddContribution | DialogMode::EditContribution => {
            draw_contribution_dialog(ui.ctx(), app);
        },
        _ => {}
    }
}

fn draw_analysis_list(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.heading("Analyses");
        if ui.button("➕ Add Analysis").clicked() {
            // Create new analysis but don't add it yet - it will be added when dialog is saved
            app.state.ui.dialog_mode = DialogMode::AddAnalysis;
        }
    });

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let mut delete_index = None;
            
            for (idx, analysis) in app.state.analysis.analyses.iter().enumerate() {
                let selected = app.state.ui.analysis_list_state.selected() == Some(idx);
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Main analysis info and selection
                        if ui.selectable_label(selected, {
                            let method_list = analysis.methods.iter()
                                .map(|m| format!("{:?}", m))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!(
                                "{}\n{} contributions, Methods: {}", 
                                analysis.name,
                                analysis.contributions.len(),
                                method_list
                            )
                        }).clicked() {
                            app.state.ui.analysis_list_state.select(Some(idx));
                        }
                        
                        // Action buttons on the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.horizontal(|ui| {
                                // Delete button
                                if ui.button("🗑").clicked() {
                                    delete_index = Some(idx);
                                }
                                
                                // Edit button
                                if ui.button("✏").clicked() {
                                    app.state.ui.analysis_list_state.select(Some(idx));
                                    app.state.ui.dialog_mode = DialogMode::EditAnalysis;
                                }
                                
                                // Run analysis button
                                if ui.button("▶").clicked() {
                                    let results = analysis.run_analysis(&app.state.project.components);
                                    app.state.analysis.latest_results.insert(analysis.id.clone(), results);
                                    app.state.ui.analysis_tab = AnalysisTab::Results;
                                }
                            });
                        });
                    });

                    // Show additional details when selected
                    if selected {
                        ui.add_space(4.0);
                        
                        // Show Monte Carlo settings if enabled
                        if analysis.methods.contains(&AnalysisMethod::MonteCarlo) {
                            if let Some(mc_settings) = &analysis.monte_carlo_settings {
                                ui.label(format!(
                                    "Monte Carlo: {} iterations, {:.1}% confidence",
                                    mc_settings.iterations,
                                    mc_settings.confidence * 100.0
                                ));
                            }
                        }

                        // Add contribution button when analysis is selected
                        if ui.button("Add Contribution").clicked() {
                            app.state.ui.dialog_mode = DialogMode::AddContribution;
                        }

                        // Show latest results summary if available
                        if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
                            ui.add_space(2.0);
                            ui.label(format!("Nominal: {:.3}", results.nominal));
                            
                            if let Some(mc) = &results.monte_carlo {
                                ui.label(format!(
                                    "Latest Run: Mean = {:.3}, σ = {:.3}",
                                    mc.mean,
                                    mc.std_dev
                                ));
                            }
                        }
                    }
                });
                ui.add_space(4.0);
            }

            // Handle deletion
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

fn draw_analysis_details(ui: &mut egui::Ui, app: &mut App, analysis: &StackupAnalysis) {
    ui.group(|ui| {
        // Analysis header section with edit button
        ui.horizontal(|ui| {
            ui.heading(&analysis.name);
            if ui.button("✏").clicked() {
                // Handle through app.state.ui.dialog_mode
            }
        });
        ui.add_space(8.0);

        // Methods section
        ui.group(|ui| {
            ui.heading("Analysis Methods");
            for method in &analysis.methods {
                ui.label(format!("• {:?}", method));
            }
        });

        ui.add_space(8.0);

        // Monte Carlo settings if enabled
        if analysis.methods.contains(&AnalysisMethod::MonteCarlo) {
            ui.group(|ui| {
                ui.heading("Monte Carlo Settings");
                if let Some(settings) = &analysis.monte_carlo_settings {
                    ui.horizontal(|ui| {
                        ui.label("Iterations:");
                        ui.label(settings.iterations.to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Confidence Level:");
                        ui.label(format!("{:.2}%", settings.confidence * 100.0));
                    });
                    if let Some(seed) = settings.seed {
                        ui.horizontal(|ui| {
                            ui.label("Random Seed:");
                            ui.label(seed.to_string());
                        });
                    }
                }
            });

            ui.add_space(8.0);
        }

        // Contributions section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Contributions");
                if ui.button("➕ Add Contribution").clicked() {
                    app.state.ui.dialog_mode = DialogMode::AddContribution;
                }
            });

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 60.0)
                .show(ui, |ui| {
                    for (idx, contrib) in analysis.contributions.iter().enumerate() {
                        let selected = app.state.ui.contribution_list_state.selected() == Some(idx);
                        
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Component and feature info
                                ui.vertical(|ui| {
                                    ui.set_min_width(ui.available_width() - 50.0);
                                    
                                    // Find the actual feature to display its values
                                    if let Some(feature) = find_feature(app, &contrib.component_id, &contrib.feature_id) {
                                        let label = format!(
                                            "{}.{} {} {}",
                                            contrib.component_id,
                                            contrib.feature_id,
                                            if contrib.direction > 0.0 { "+" } else { "-" },
                                            if contrib.half_count { "(½)" } else { "" }
                                        );
                                        ui.strong(label);

                                        ui.label(format!(
                                            "Value: {:.3} [{:+.3}/{:+.3}]",
                                            feature.dimension.value,
                                            feature.dimension.plus_tolerance,
                                            feature.dimension.minus_tolerance
                                        ));

                                        if let Some(dist_type) = feature.distribution {
                                            ui.label(format!("Distribution: {:?}", dist_type));
                                        }
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Missing feature: {}.{}", contrib.component_id, contrib.feature_id)
                                        );
                                    }
                                });
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
        });
    });
}

fn draw_analysis_results(ui: &mut egui::Ui, app: &App, analysis: &StackupAnalysis) {
    if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
        // Nominal value at top
        ui.group(|ui| {
            ui.heading("Nominal Value");
            ui.label(format!("{:.6}", results.nominal));
        });

        ui.add_space(8.0);

        egui::Grid::new("results_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .min_col_width(ui.available_width() / 2.0)
            .show(ui, |ui| {
                // Worst Case Results
                if let Some(wc) = &results.worst_case {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("Worst Case Analysis");
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Minimum:");
                                    ui.label("Maximum:");
                                    ui.label("Range:");
                                });
                                ui.vertical(|ui| {
                                    ui.label(format!("{:.6}", wc.min));
                                    ui.label(format!("{:.6}", wc.max));
                                    ui.label(format!("{:.6}", wc.max - wc.min));
                                });
                            });

                            if !wc.sensitivity.is_empty() {
                                ui.add_space(8.0);
                                ui.label("Top Contributors:");
                                for sens in wc.sensitivity.iter().take(3) {
                                    ui.label(format!(
                                        "{}.{}: {:.1}%",
                                        sens.component_id,
                                        sens.feature_id,
                                        sens.contribution_percent
                                    ));
                                }
                            }
                        });
                    });

                    // RSS Results
                    if let Some(rss) = &results.rss {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.heading("RSS Analysis");
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("Mean:");
                                        ui.label("Std Dev:");
                                        ui.label("3σ Range:");
                                    });
                                    ui.vertical(|ui| {
                                        ui.label(format!("{:.6}", results.nominal));
                                        ui.label(format!("{:.6}", rss.std_dev));
                                        ui.label(format!("[{:.6}, {:.6}]", rss.min, rss.max));
                                    });
                                });

                                // Show sensitivities
                                if !rss.sensitivity.is_empty() {
                                    ui.add_space(8.0);
                                    ui.label("Top Contributors:");
                                    for sens in rss.sensitivity.iter().take(3) {
                                        ui.label(format!(
                                            "{}.{}: {:.1}%",
                                            sens.component_id,
                                            sens.feature_id,
                                            sens.contribution_percent
                                        ));
                                    }
                                }
                            });
                        });
                    }
                    ui.end_row();

                    // Monte Carlo Results
                    if let Some(mc) = &results.monte_carlo {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.heading("Monte Carlo Analysis");
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("Mean:");
                                        ui.label("Std Dev:");
                                        ui.label("Range:");
                                    });
                                    ui.vertical(|ui| {
                                        ui.label(format!("{:.6}", mc.mean));
                                        ui.label(format!("{:.6}", mc.std_dev));
                                        ui.label(format!("[{:.6}, {:.6}]", mc.min, mc.max));
                                    });
                                });

                                // Confidence Intervals
                                if !mc.confidence_intervals.is_empty() {
                                    ui.add_space(8.0);
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

                                // Show sensitivities
                                if !mc.sensitivity.is_empty() {
                                    ui.add_space(8.0);
                                    ui.label("Top Contributors:");
                                    for sens in mc.sensitivity.iter().take(3) {
                                        ui.label(format!(
                                            "{}.{}: {:.1}% (corr: {:.3})",
                                            sens.component_id,
                                            sens.feature_id,
                                            sens.contribution_percent,
                                            sens.correlation.unwrap_or(0.0)
                                        ));
                                    }
                                }
                            });
                        });
                    }
                }
            });
    } else {
        ui.centered_and_justified(|ui| {
            ui.add_space(20.0);
            ui.label("Run analysis to see results");
        });
    }
}

fn draw_analysis_visualization(ui: &mut egui::Ui, app: &App, analysis: &StackupAnalysis) {
    if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
        if let Some(mc) = &results.monte_carlo {
            // Split screen into histogram and waterfall
            egui::Grid::new("visualization_grid")
                .num_columns(1)
                .spacing([0.0, 16.0])
                .show(ui, |ui| {
                    // Histogram
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Distribution Histogram");
                            let plot = egui_plot::Plot::new("mc_histogram")
                                .height(200.0)
                                .allow_zoom(false)
                                .allow_drag(false)
                                .show_background(false)
                                .show_axes([false, true])
                                .include_y(0.0);

                            plot.show(ui, |plot_ui| {
                                // Create histogram bars
                                let bars: Vec<egui_plot::Bar> = mc.histogram.iter()
                                    .map(|(value, count)| {
                                        egui_plot::Bar::new(*value, *count as f64)
                                            .width(((mc.max - mc.min) / mc.histogram.len() as f64) * 0.9)
                                            .fill(egui::Color32::from_rgb(100, 150, 255))
                                    })
                                    .collect();

                                plot_ui.bar_chart(egui_plot::BarChart::new(bars));

                                // Add mean line
                                let mean_line = egui_plot::Line::new(vec![
                                    [mc.mean, 0.0],
                                    [mc.mean, mc.histogram.iter().map(|(_, count)| *count as f64).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)],
                                ])
                                .color(egui::Color32::RED)
                                .width(2.0);

                                plot_ui.line(mean_line);
                            });

                            // Add statistics below the histogram
                            ui.horizontal(|ui| {
                                ui.label(format!("Mean: {:.3}", mc.mean));
                                ui.label(format!("Std Dev: {:.3}", mc.std_dev));
                                ui.label(format!("Range: [{:.3}, {:.3}]", mc.min, mc.max));
                            });

                            // Show confidence intervals
                            if !mc.confidence_intervals.is_empty() {
                                ui.group(|ui| {
                                    ui.heading("Confidence Intervals");
                                    for interval in &mc.confidence_intervals {
                                        ui.label(format!(
                                            "{:.1}%: [{:.3}, {:.3}]",
                                            interval.confidence_level * 100.0,
                                            interval.lower_bound,
                                            interval.upper_bound
                                        ));
                                    }
                                });
                            }
                        });
                    });
                    ui.end_row();

                    // Waterfall chart
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Contribution Waterfall");
                            let plot = egui_plot::Plot::new("contribution_waterfall")
                                .height(200.0)
                                .allow_zoom(false)
                                .allow_drag(false)
                                .show_background(false);

                            plot.show(ui, |plot_ui| {
                                let mut running_total = 0.0;
                                let mut bars = Vec::new();
                                
                                // Starting point
                                bars.push(egui_plot::Bar::new(0.0, 0.0)
                                    .name("Start")
                                    .width(0.5)
                                    .fill(egui::Color32::GRAY));

                                // Add bars for each contribution
                                for (i, contrib) in analysis.contributions.iter().enumerate() {
                                    if let Some(feature) = find_feature(app, &contrib.component_id, &contrib.feature_id) {
                                        let value = contrib.direction * feature.dimension.value 
                                            * if contrib.half_count { 0.5 } else { 1.0 };
                                        
                                        running_total += value;
                                        
                                        let bar = egui_plot::Bar::new((i + 1) as f64, value)
                                            .name(&format!("{}.{}", contrib.component_id, contrib.feature_id))
                                            .width(0.5)
                                            .fill(if value >= 0.0 {
                                                egui::Color32::from_rgb(100, 200, 100)
                                            } else {
                                                egui::Color32::from_rgb(200, 100, 100)
                                            });
                                        
                                        bars.push(bar);
                                    }
                                }

                                // Final total
                                bars.push(egui_plot::Bar::new((analysis.contributions.len() + 1) as f64, running_total)
                                    .name("Total")
                                    .width(0.5)
                                    .fill(egui::Color32::BLUE));

                                plot_ui.bar_chart(egui_plot::BarChart::new(bars));
                            });

                            // Add contribution statistics
                            ui.group(|ui| {
                                ui.heading("Sensitivities");
                                for sens in &mc.sensitivity {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{}.{}: {:.1}% (correlation: {:.3})",
                                            sens.component_id,
                                            sens.feature_id,
                                            sens.contribution_percent,
                                            sens.correlation.unwrap_or(0.0)
                                        ));
                                    });
                                }
                            });
                        });
                    });
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Run Monte Carlo analysis to see visualizations");
            });
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Run analysis to see visualizations");
        });
    }
}

fn draw_analysis_dialog(ctx: &egui::Context, app: &mut App) {
    let title = match app.state.ui.dialog_mode {
        DialogMode::AddAnalysis => "Add Analysis",
        DialogMode::EditAnalysis => "Edit Analysis",
        _ => return,
    };

    let mut temp_analysis = if let DialogMode::EditAnalysis = app.state.ui.dialog_mode {
        // If editing, get the existing analysis
        if let Some(idx) = app.state.ui.analysis_list_state.selected() {
            app.state.analysis.analyses.get(idx).cloned()
                .unwrap_or_else(|| StackupAnalysis::new("New Analysis".to_string()))
        } else {
            StackupAnalysis::new("New Analysis".to_string())
        }
    } else {
        // If adding new, create fresh analysis
        StackupAnalysis::new("New Analysis".to_string())
    };

    egui::Window::new(title)
        .fixed_size([400.0, 500.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .resizable(false)
        .show(ctx, |ui| {
            let mut should_close = false;
            let mut should_save = false;

            ui.vertical(|ui| {
                // Name input
                ui.group(|ui| {
                    ui.heading("Analysis Name");
                    ui.text_edit_singleline(&mut temp_analysis.name);
                });

                ui.add_space(8.0);

                // Methods selection
                ui.group(|ui| {
                    ui.heading("Analysis Methods");
                    for method in &[AnalysisMethod::WorstCase, AnalysisMethod::Rss, AnalysisMethod::MonteCarlo] {
                        let mut enabled = temp_analysis.methods.contains(method);
                        if ui.checkbox(&mut enabled, format!("{:?}", method)).changed() {
                            if enabled {
                                temp_analysis.methods.push(*method);
                            } else {
                                temp_analysis.methods.retain(|m| m != method);
                            }
                        }
                    }
                });

                ui.add_space(8.0);

                // Monte Carlo settings
                if temp_analysis.methods.contains(&AnalysisMethod::MonteCarlo) {
                    ui.group(|ui| {
                        ui.heading("Monte Carlo Settings");
                        let settings = temp_analysis.monte_carlo_settings.get_or_insert_with(Default::default);
                        
                        ui.horizontal(|ui| {
                            ui.label("Iterations:");
                            let mut iter_str = settings.iterations.to_string();
                            if ui.text_edit_singleline(&mut iter_str).changed() {
                                if let Ok(value) = iter_str.parse() {
                                    settings.iterations = value;
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Confidence (%):");
                            let mut conf_str = (settings.confidence * 100.0).to_string();
                            if ui.text_edit_singleline(&mut conf_str).changed() {
                                if let Ok(value) = conf_str.parse::<f64>() {
                                    settings.confidence = (value / 100.0).clamp(0.0, 0.9999);
                                }
                            }
                        });
                    });
                }

                // Action buttons
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }

                    let name_valid = !temp_analysis.name.trim().is_empty();
                    let methods_valid = !temp_analysis.methods.is_empty();
                    
                    if ui.add_enabled(
                        name_valid && methods_valid,
                        egui::Button::new("Save")
                    ).clicked() {
                        should_save = true;
                        should_close = true;
                    }
                });
            });

            if should_save {
                match app.state.ui.dialog_mode {
                    DialogMode::AddAnalysis => {
                        app.state.analysis.analyses.push(temp_analysis);
                        let idx = app.state.analysis.analyses.len() - 1;
                        app.state.ui.analysis_list_state.select(Some(idx));
                    },
                    DialogMode::EditAnalysis => {
                        if let Some(idx) = app.state.ui.analysis_list_state.selected() {
                            if let Some(existing) = app.state.analysis.analyses.get_mut(idx) {
                                *existing = temp_analysis;
                            }
                        }
                    },
                    _ => {}
                }
            }
            if should_close {
                app.state.ui.dialog_mode = DialogMode::None;
            }
        });
}

#[derive(Default)]
struct ContributionDialogState {
    component: String,
    feature: String,
    direction: f64,
    half_count: bool,
}

fn draw_contribution_dialog(ctx: &egui::Context, app: &mut App) {
    if !matches!(app.state.ui.dialog_mode, DialogMode::AddContribution | DialogMode::EditContribution) {
        return;
    }

    let title = match app.state.ui.dialog_mode {
        DialogMode::AddContribution => "Add Contribution",
        DialogMode::EditContribution => "Edit Contribution",
        _ => return,
    };

    let mut dialog_state = ContributionDialogState {
        direction: 1.0,
        ..Default::default()
    };

    // If editing, populate with existing data
    if let DialogMode::EditContribution = app.state.ui.dialog_mode {
        if let Some(selected_analysis) = app.state.ui.analysis_list_state.selected()
            .and_then(|idx| app.state.analysis.analyses.get(idx)) 
        {
            if let Some(selected_contribution) = app.state.ui.contribution_list_state.selected()
                .and_then(|idx| selected_analysis.contributions.get(idx)) 
            {
                dialog_state = ContributionDialogState {
                    component: selected_contribution.component_id.clone(),
                    feature: selected_contribution.feature_id.clone(),
                    direction: selected_contribution.direction,
                    half_count: selected_contribution.half_count,
                };
            }
        }
    }

    egui::Window::new(title)
        .fixed_size([400.0, 500.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .resizable(false)
        .show(ctx, |ui| {
            let mut should_close = false;
            let mut should_save = false;

            ui.vertical(|ui| {
                // Component selection
                ui.group(|ui| {
                    ui.heading("Component");
                    egui::ComboBox::from_label("Select Component")
                        .selected_text(&dialog_state.component)
                        .show_ui(ui, |ui| {
                            for component in &app.state.project.components {
                                ui.selectable_value(
                                    &mut dialog_state.component,
                                    component.name.clone(),
                                    &component.name
                                );
                            }
                        });
                });

                // Feature selection (only if component is selected)
                if !dialog_state.component.is_empty() {
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.heading("Feature");
                        if let Some(component) = app.state.project.components.iter()
                            .find(|c| c.name == dialog_state.component) 
                        {
                            egui::ComboBox::from_label("Select Feature")
                                .selected_text(&dialog_state.feature)
                                .show_ui(ui, |ui| {
                                    for feature in &component.features {
                                        ui.selectable_value(
                                            &mut dialog_state.feature,
                                            feature.name.clone(),
                                            format!("{} ({:?})", feature.name, feature.feature_type)
                                        );
                                    }
                                });

                            // Show feature details if selected
                            if let Some(feature) = component.features.iter()
                                .find(|f| f.name == dialog_state.feature)
                            {
                                ui.add_space(4.0);
                                ui.label(format!(
                                    "Value: {:.3} [{:+.3}/{:+.3}]",
                                    feature.dimension.value,
                                    feature.dimension.plus_tolerance,
                                    feature.dimension.minus_tolerance
                                ));
                                if let Some(dist) = &feature.distribution {
                                    ui.label(format!("Distribution: {:?}", dist));
                                }
                            }
                        }
                    });
                }

                // Direction and half count
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.heading("Properties");
                    
                    ui.horizontal(|ui| {
                        ui.label("Direction:");
                        let mut is_positive = dialog_state.direction > 0.0;
                        if ui.radio_value(&mut is_positive, true, "Positive").clicked() ||
                           ui.radio_value(&mut is_positive, false, "Negative").clicked() {
                            dialog_state.direction = if is_positive { 1.0 } else { -1.0 };
                        }
                    });

                    ui.checkbox(&mut dialog_state.half_count, "Half Count");
                });

                // Action buttons
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }

                    let can_save = !dialog_state.component.is_empty() 
                        && !dialog_state.feature.is_empty();
                    
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        should_save = true;
                        should_close = true;
                    }
                });
            });

            if should_save {
                // Get the feature for distribution parameters
                if let Some(feature) = find_feature(app, &dialog_state.component, &dialog_state.feature) {
                    let contribution = StackupContribution {
                        component_id: dialog_state.component.clone(),
                        feature_id: dialog_state.feature.clone(),
                        direction: dialog_state.direction,
                        half_count: dialog_state.half_count,
                        distribution: feature.distribution.map(|_| 
                            StackupAnalysis::calculate_distribution_params(feature)
                        ),
                    };

                    if let Some(selected) = app.state.ui.analysis_list_state.selected() {
                        if let Some(analysis) = app.state.analysis.analyses.get_mut(selected) {
                            match app.state.ui.dialog_mode {
                                DialogMode::AddContribution => {
                                    analysis.contributions.push(contribution);
                                },
                                DialogMode::EditContribution => {
                                    if let Some(contribution_idx) = app.state.ui.contribution_list_state.selected() {
                                        if contribution_idx < analysis.contributions.len() {
                                            analysis.contributions[contribution_idx] = contribution;
                                        }
                                    }
                                },
                                _ => {},
                            }
                        }
                    }
                }
            }

            if should_close {
                app.state.ui.dialog_mode = DialogMode::None;
            }
        });
}


fn save_analysis(app: &mut App, analysis: StackupAnalysis) {
    match app.state.ui.dialog_mode {
        DialogMode::AddAnalysis => {
            app.state.analysis.analyses.push(analysis);
            let idx = app.state.analysis.analyses.len() - 1;
            app.state.ui.analysis_list_state.select(Some(idx));
        },
        DialogMode::EditAnalysis => {
            if let Some(idx) = app.state.ui.analysis_list_state.selected() {
                if let Some(existing) = app.state.analysis.analyses.get_mut(idx) {
                    *existing = analysis;
                }
            }
        },
        _ => {}
    }

    // Save changes to file system if needed
    if let Err(e) = app.state.file_manager.save_project(
        &app.state.project.project_file,
        &app.state.project.components,
        &app.state.analysis.analyses,
    ) {
        // In a real app, you'd want to handle this error appropriately
        println!("Error saving analysis: {}", e);
    }

    // Close the dialog
    app.state.ui.dialog_mode = DialogMode::None;
}

fn save_contribution(app: &mut App) {
    if let Some(selected) = app.state.ui.analysis_list_state.selected() {
        let inputs = &app.state.input.contribution_inputs;
        
        // First get the feature information
        let feature_info = find_feature(app, &inputs.selected_component, &inputs.selected_feature)
            .map(|feature| (feature.distribution, feature.clone()));
            
        if let Some((dist_type, feature)) = feature_info {
            // Create the contribution
            let contribution = StackupContribution {
                component_id: inputs.selected_component.clone(),
                feature_id: inputs.selected_feature.clone(),
                direction: inputs.direction,
                half_count: inputs.half_count,
                distribution: dist_type.map(|_| StackupAnalysis::calculate_distribution_params(&feature)),
            };

            // Now update the analysis
            if let Some(analysis) = app.state.analysis.analyses.get_mut(selected) {
                match app.state.ui.dialog_mode {
                    DialogMode::AddContribution => {
                        analysis.contributions.push(contribution);
                    },
                    DialogMode::EditContribution => {
                        if let Some(idx) = analysis.contributions.iter().position(|c| 
                            c.component_id == inputs.selected_component && 
                            c.feature_id == inputs.selected_feature
                        ) {
                            analysis.contributions[idx] = contribution;
                        }
                    },
                    _ => {},
                }
            }
        }
    }
}

fn find_feature<'a>(app: &'a App, component_name: &str, feature_name: &str) -> Option<&'a Feature> {
    app.state.project.components.iter()
        .find(|c| c.name == component_name)?
        .features.iter()
        .find(|f| f.name == feature_name)
}