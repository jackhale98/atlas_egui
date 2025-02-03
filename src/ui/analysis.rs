// src/ui/analysis.rs
use eframe::egui;
use crate::app::App;
use crate::state::ui_state::{DialogMode, AnalysisTab};
use crate::state::input_state::{InputMode, EditField};
use crate::analysis::stackup::{
    AnalysisMethod, MonteCarloResult, StackupAnalysis, AnalysisResults,
    StackupContribution, DistributionType, DistributionParams
};
use crate::analysis::MonteCarloSettings;
use crate::ui::dialog::{DialogState, AnalysisEditData, ContributionEditData};
use crate::config::Feature;
use uuid::Uuid;

pub fn draw_analysis_view(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    let available_size = ui.available_size();
    
    // Clone the selected analysis for immutable use
    let selected_analysis = app.state.ui.analysis_list_state.selected()
        .and_then(|idx| app.state.analysis.analyses.get(idx).cloned());

    egui::Grid::new("analysis_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            // Left panel - Analysis List
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.3);
                ui.set_min_height(available_size.y);
                
                ui.heading("Analyses");
                ui.add_space(4.0);

                if ui.button("➕ Add Analysis").clicked() {
                    *dialog_state = DialogState::AnalysisEdit(AnalysisEditData {
                        name: String::new(),
                        methods: vec![],
                        monte_carlo_settings: MonteCarloSettings::default(),
                        is_editing: false,
                        analysis_index: None,
                    });
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_source("analysis_list_scroll")
                    .show(ui, |ui| {
                        let mut delete_index = None;

                        for (idx, analysis) in app.state.analysis.analyses.iter().enumerate() {
                            let is_selected = app.state.ui.analysis_list_state.selected() == Some(idx);
                            
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                
                                let methods_str = analysis.methods.iter()
                                    .map(|m| format!("{:?}", m))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                
                                let response = ui.selectable_label(
                                    is_selected,
                                    format!(
                                        "{}\nMethods: {}\nContributions: {}",
                                        analysis.name,
                                        methods_str,
                                        analysis.contributions.len()
                                    )
                                );

                                if response.clicked() {
                                    app.state.ui.analysis_list_state.select(Some(idx));
                                }

                                response.context_menu(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        *dialog_state = DialogState::AnalysisEdit(AnalysisEditData {
                                            name: analysis.name.clone(),
                                            methods: analysis.methods.clone(),
                                            monte_carlo_settings: analysis.monte_carlo_settings.clone()
                                                .unwrap_or_default(),
                                            is_editing: true,
                                            analysis_index: Some(idx),
                                        });
                                        ui.close_menu();
                                    }

                                    if ui.button("▶ Run Analysis").clicked() {
                                        let results = analysis.run_analysis(&app.state.project.components);
                                        app.state.analysis.latest_results.insert(analysis.id.clone(), results.clone());
                                        // Save results to file system
                                        if let Err(e) = app.state.file_manager.analysis_handler.save_analysis(
                                            analysis,
                                            &results
                                        ) {
                                            println!("Error saving analysis results: {}", e);
                                        }
                                        ui.close_menu();
                                    }

                                    ui.separator();

                                    if ui.button(egui::RichText::new("🗑 Delete").color(egui::Color32::RED)).clicked() {
                                        delete_index = Some(idx);
                                        ui.close_menu();
                                    }
                                });

                                // Show additional details when selected
                                if is_selected {
                                    ui.add_space(4.0);
                                    
                                    if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
                                        ui.label(format!("Last Run: {}", results.timestamp));
                                        if let Some(mc) = &results.monte_carlo {
                                            ui.label(format!(
                                                "Mean: {:.3}, σ = {:.3}",
                                                mc.mean,
                                                mc.std_dev
                                            ));
                                        }
                                    }
                                }
                            });
                            ui.add_space(4.0);
                        }

                        if let Some(idx) = delete_index {
                            app.state.analysis.analyses.remove(idx);
                            if app.state.analysis.analyses.is_empty() {
                                app.state.ui.analysis_list_state.select(None);
                            } else if idx >= app.state.analysis.analyses.len() {
                                app.state.ui.analysis_list_state.select(Some(app.state.analysis.analyses.len() - 1));
                            }

                            if let Err(e) = app.state.file_manager.save_project(
                                &app.state.project.project_file,
                                &app.state.project.components,
                                &app.state.analysis.analyses,
                            ) {
                                println!("Error saving project after analysis deletion: {}", e);
                            }
                        }
                    });
            });

            // Right panel
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.7);
                ui.set_min_height(available_size.y);

                if let Some(analysis) = &selected_analysis {
                    // Analysis details header
                    ui.heading(&analysis.name);
                    ui.add_space(8.0);

                    // Tabs
                    ui.horizontal(|ui| {
                        for (tab, label) in [
                            (AnalysisTab::Details, "Details"),
                            (AnalysisTab::Results, "Results"),
                            (AnalysisTab::Visualization, "Visualization"),
                        ] {
                            if ui.selectable_label(app.state.ui.analysis_tab == tab, label).clicked() {
                                app.state.ui.analysis_tab = tab;
                            }
                        }
                    });
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Content based on selected tab
                    match app.state.ui.analysis_tab {
                        AnalysisTab::Details => draw_analysis_details(ui, app, analysis, dialog_state),
                        AnalysisTab::Results => {
                            if let Some(results) = app.state.analysis.latest_results.get(&analysis.id).cloned() {
                                draw_analysis_results(ui, app, analysis, Some(&results));
                            } else {
                                draw_analysis_results(ui, app, analysis, None);
                            }
                        },
                        AnalysisTab::Visualization => {
                            let results = app.state.analysis.latest_results.get(&analysis.id);
                            draw_analysis_visualization(ui, app, analysis, results);
                        },
                        _ => {}
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select an analysis to view details");
                    });
                }
            });
        });

    // Handle dialogs
    match dialog_state {
        DialogState::AnalysisEdit(_) => {
            draw_analysis_dialog(ui.ctx(), dialog_state, app);
        },
        DialogState::ContributionEdit(_) => {
            draw_contribution_dialog(ui.ctx(), dialog_state, app);
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

fn draw_analysis_details(ui: &mut egui::Ui, app: &mut App, analysis: &StackupAnalysis, dialog_state: &mut DialogState) {
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
                    *dialog_state = DialogState::ContributionEdit(ContributionEditData {
                        component_id: String::new(),
                        feature_id: String::new(),
                        direction: 1.0,
                        half_count: false,
                        analysis_index: Some(app.state.ui.analysis_list_state.selected().unwrap_or(0)),
                        contribution_index: None,
                        is_editing: false,
                    });
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

                                // Add delete button on the right
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑").clicked() {
                                        if let Some(analysis_idx) = app.state.ui.analysis_list_state.selected() {
                                            if let Some(analysis) = app.state.analysis.analyses.get_mut(analysis_idx) {
                                                analysis.contributions.remove(idx);
                                                // Save changes
                                                if let Err(e) = app.state.file_manager.save_project(
                                                    &app.state.project.project_file,
                                                    &app.state.project.components,
                                                    &app.state.analysis.analyses,
                                                ) {
                                                    println!("Error saving after contribution deletion: {}", e);
                                                }
                                            }
                                        }
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

fn draw_analysis_results(ui: &mut egui::Ui, app: &mut App, analysis: &StackupAnalysis, results: Option<&AnalysisResults>) {
    // Split screen into results and history
    egui::Grid::new("results_grid")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            // Left side - Latest results
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width() * 0.7);
                
                if let Some(results) = app.state.analysis.latest_results.get(&analysis.id) {
                    ui.group(|ui| {
                        ui.heading("Latest Results");
                        ui.add_space(8.0);
                        
                        // Nominal value
                        ui.group(|ui| {
                            ui.heading("Nominal Value");
                            ui.strong(format!("{:.6}", results.nominal));
                        });

                        // Show results in columns for better space utilization
                        egui::Grid::new("analysis_results")
                            .num_columns(2)
                            .spacing([40.0, 8.0])
                            .show(ui, |ui| {
                                if let Some(wc) = &results.worst_case {
                                    ui.vertical(|ui| {
                                        ui.group(|ui| {
                                            ui.heading("Worst Case");
                                            ui.label(format!("Min: {:.6}", wc.min));
                                            ui.label(format!("Max: {:.6}", wc.max));
                                            ui.label(format!("Range: {:.6}", wc.max - wc.min));
                                        });
                                    });

                                    if let Some(rss) = &results.rss {
                                        ui.vertical(|ui| {
                                            ui.group(|ui| {
                                                ui.heading("RSS Analysis");
                                                ui.label(format!("Mean: {:.6}", results.nominal));
                                                ui.label(format!("Std Dev: {:.6}", rss.std_dev));
                                                ui.label(format!("3σ Range: [{:.6}, {:.6}]", rss.min, rss.max));
                                            });
                                        });
                                        ui.end_row();
                                    }
                                }

                                if let Some(mc) = &results.monte_carlo {
                                    ui.vertical(|ui| {
                                        ui.group(|ui| {
                                            ui.heading("Monte Carlo");
                                            ui.label(format!("Mean: {:.6}", mc.mean));
                                            ui.label(format!("Std Dev: {:.6}", mc.std_dev));
                                            ui.label(format!("Range: [{:.6}, {:.6}]", mc.min, mc.max));
                                            
                                            // Add confidence intervals
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
                                        });
                                    });
                                }
                            });
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Run analysis to see results");
                    });
                }
            });

            // Right side - History viewer
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width() * 0.3);
                ui.group(|ui| {
                    ui.heading("Analysis History");
                    
                    // Get metadata for this analysis
                    if let Ok(metadata) = app.state.file_manager.analysis_handler.load_metadata(&analysis.id) {
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 40.0)
                            .show(ui, |ui| {
                                for results_file in metadata.results_files.iter().rev() {
                                    ui.group(|ui| {
                                        let timestamp = results_file.timestamp
                                            .format("%Y-%m-%d %H:%M:%S")
                                            .to_string();
                                        
                                        let methods = results_file.analysis_methods.iter()
                                            .map(|m| format!("{:?}", m))
                                            .collect::<Vec<_>>()
                                            .join(", ");

                                        ui.label(format!("Run: {}", timestamp));
                                        ui.label(format!("Methods: {}", methods));

                                        // Add button to load these results
                                        if ui.button("View Results").clicked() {
                                            // Load and display historical results
                                            if let Ok(content) = std::fs::read_to_string(
                                                app.state.file_manager.analysis_handler.get_results_file_path(&results_file.path)
                                            ) {
                                                if let Ok(historical_results) = ron::from_str(&content) {
                                                    app.state.analysis.latest_results
                                                        .insert(analysis.id.clone(), historical_results);
                                                }
                                            }
                                        }
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                    }
                });
            });
        });
}

fn draw_analysis_visualization(ui: &mut egui::Ui, app: &App, analysis: &StackupAnalysis, results: Option<&AnalysisResults>) {
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

fn draw_analysis_dialog(ctx: &egui::Context, dialog_state: &mut DialogState, app: &mut App) {
    let mut should_close = false;
    let mut should_save = false;
    let mut updated_analysis = None;

    if let DialogState::AnalysisEdit(data) = dialog_state {
        let title = if data.is_editing { "Edit Analysis" } else { "New Analysis" };
        
        egui::Window::new(title)
            .fixed_size([400.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Name input
                    ui.group(|ui| {
                        ui.heading("Analysis Name");
                        ui.text_edit_singleline(&mut data.name);
                    });

                    ui.add_space(8.0);

                    // Methods selection
                    ui.group(|ui| {
                        ui.heading("Analysis Methods");
                        for method in &[AnalysisMethod::WorstCase, AnalysisMethod::Rss, AnalysisMethod::MonteCarlo] {
                            let mut enabled = data.methods.contains(method);
                            if ui.checkbox(&mut enabled, format!("{:?}", method)).changed() {
                                if enabled {
                                    data.methods.push(*method);
                                } else {
                                    data.methods.retain(|m| m != method);
                                }
                            }
                        }
                    });

                    // Monte Carlo settings if enabled
                    if data.methods.contains(&AnalysisMethod::MonteCarlo) {
                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.heading("Monte Carlo Settings");
                            
                            ui.horizontal(|ui| {
                                ui.label("Iterations:");
                                let mut iter_str = data.monte_carlo_settings.iterations.to_string();
                                if ui.text_edit_singleline(&mut iter_str).changed() {
                                    if let Ok(value) = iter_str.parse() {
                                        data.monte_carlo_settings.iterations = value;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Confidence (%):");
                                let mut conf_str = (data.monte_carlo_settings.confidence * 100.0).to_string();
                                if ui.text_edit_singleline(&mut conf_str).changed() {
                                    if let Ok(value) = conf_str.parse::<f64>() {
                                        data.monte_carlo_settings.confidence = (value / 100.0).clamp(0.0, 0.9999);
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

                        let can_save = !data.name.trim().is_empty() && !data.methods.is_empty();
                        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                            should_save = true;
                            should_close = true;
                            updated_analysis = Some((StackupAnalysis {
                                id: Uuid::new_v4().to_string(),
                                name: data.name.clone(),
                                methods: data.methods.clone(),
                                monte_carlo_settings: if data.methods.contains(&AnalysisMethod::MonteCarlo) {
                                    Some(data.monte_carlo_settings.clone())
                                } else {
                                    None
                                },
                                contributions: vec![],
                            }, data.is_editing, data.analysis_index));
                        }
                    });
                });
            });
    }

    // Handle state changes outside the closure
    if should_save {
        if let Some((new_analysis, is_editing, idx)) = updated_analysis {
            if is_editing {
                if let Some(idx) = idx {
                    if let Some(analysis) = app.state.analysis.analyses.get_mut(idx) {
                        // Preserve existing contributions
                        let contributions = analysis.contributions.clone();
                        *analysis = new_analysis;
                        analysis.contributions = contributions;
                    }
                }
            } else {
                app.state.analysis.analyses.push(new_analysis);
                app.state.ui.analysis_list_state.select(Some(app.state.analysis.analyses.len() - 1));
            }

            // Save to filesystem
            if let Err(e) = app.state.file_manager.save_project(
                &app.state.project.project_file,
                &app.state.project.components,
                &app.state.analysis.analyses,
            ) {
                println!("Error saving analysis: {}", e);
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
}

#[derive(Default)]
struct ContributionDialogState {
    component: String,
    feature: String,
    direction: f64,
    half_count: bool,
}

fn draw_contribution_dialog(ctx: &egui::Context, dialog_state: &mut DialogState, app: &mut App) {
    let mut should_close = false;
    let mut should_save = false;
    let mut updated_contribution = None;

    if let DialogState::ContributionEdit(data) = dialog_state {
        let title = if data.is_editing { "Edit Contribution" } else { "Add Contribution" };

        egui::Window::new(title)
            .fixed_size([400.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Component selection
                    ui.group(|ui| {
                        ui.heading("Component");
                        egui::ComboBox::from_label("Select Component")
                            .selected_text(&data.component_id)
                            .show_ui(ui, |ui| {
                                for component in &app.state.project.components {
                                    if ui.selectable_value(
                                        &mut data.component_id,
                                        component.name.clone(),
                                        &component.name
                                    ).clicked() {
                                        data.feature_id.clear();
                                    }
                                }
                            });
                    });

                    // Feature selection
                    if !data.component_id.is_empty() {
                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.heading("Feature");
                            if let Some(component) = app.state.project.components.iter()
                                .find(|c| c.name == data.component_id) 
                            {
                                egui::ComboBox::from_label("Select Feature")
                                    .selected_text(&data.feature_id)
                                    .show_ui(ui, |ui| {
                                        for feature in &component.features {
                                            ui.selectable_value(
                                                &mut data.feature_id,
                                                feature.name.clone(),
                                                format!("{} ({:?})", feature.name, feature.feature_type)
                                            );
                                        }
                                    });

                                // Show feature details if selected
                                if let Some(feature) = component.features.iter()
                                    .find(|f| f.name == data.feature_id)
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
                            if ui.radio_value(&mut data.direction, 1.0, "Positive").clicked() ||
                               ui.radio_value(&mut data.direction, -1.0, "Negative").clicked() {
                                // Direction updated via radio buttons
                            }
                        });

                        ui.checkbox(&mut data.half_count, "Half Count");
                    });

                    // Action buttons
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }

                        let can_save = !data.component_id.is_empty() 
                            && !data.feature_id.is_empty();
                        
                        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                            should_save = true;
                            should_close = true;
                            if let Some(feature) = find_feature(app, &data.component_id, &data.feature_id) {
                                updated_contribution = Some((
                                    StackupContribution {
                                        component_id: data.component_id.clone(),
                                        feature_id: data.feature_id.clone(),
                                        direction: data.direction,
                                        half_count: data.half_count,
                                        distribution: Some(StackupAnalysis::calculate_distribution_params(feature)),
                                    },
                                    data.analysis_index,
                                    data.contribution_index,
                                    data.is_editing
                                ));
                            }
                        }
                    });
                });
            });
    }

    // Handle state changes outside the closure
    if should_save {
        if let Some((contribution, analysis_idx, contrib_idx, is_editing)) = updated_contribution {
            if let Some(idx) = analysis_idx {
                if let Some(analysis) = app.state.analysis.analyses.get_mut(idx) {
                    if is_editing {
                        if let Some(contrib_idx) = contrib_idx {
                            if let Some(existing) = analysis.contributions.get_mut(contrib_idx) {
                                *existing = contribution;
                            }
                        }
                    } else {
                        analysis.contributions.push(contribution);
                    }

                    // Save to filesystem
                    if let Err(e) = app.state.file_manager.save_project(
                        &app.state.project.project_file,
                        &app.state.project.components,
                        &app.state.analysis.analyses,
                    ) {
                        println!("Error saving contribution: {}", e);
                    }
                }
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
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