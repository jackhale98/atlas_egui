// src/ui/analysis.rs
use eframe::egui;
use egui_plot::{self, Plot, BarChart, Bar, Line};
use crate::state::{AppState, DialogState, AnalysisTab};
use crate::analysis::stackup::{AnalysisMethod, MonteCarloSettings, StackupAnalysis, AnalysisResults};
use crate::config::{Component, Feature};
use crate::utils::find_feature;

pub fn show_analysis_view(ui: &mut egui::Ui, state: &mut AppState) {
    let available_size = ui.available_size();

    // Use full vertical space
    ui.horizontal(|ui| {
        // Left panel - Analysis List (with explicit width and height)
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() * 0.3);
            ui.set_min_height(available_size.y);
            show_analysis_list(ui, state);
        });
        
        ui.separator();

        // Right panel with tabs
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(available_size.y);
            
            // Tab bar
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                
                let current_tab = state.analysis_tab;
                let tabs = [
                    (AnalysisTab::Details, "Details"),
                    (AnalysisTab::Results, "Results"),
                    (AnalysisTab::Visualization, "Visualization"),
                ];

                for (tab, label) in tabs {
                    if ui.selectable_label(current_tab == tab, label).clicked() {
                        state.analysis_tab = tab;
                    }
                }
            });

            ui.add_space(10.0);

            // Tab content
            if let Some(selected_idx) = state.selected_analysis {
                if let Some(analysis) = state.analyses.get(selected_idx).cloned() {
                    let results = state.latest_results.get(&analysis.id).cloned();
                    
                    match state.analysis_tab {
                        AnalysisTab::Details => {
                            show_analysis_details(ui, state, &analysis, selected_idx);
                        },
                        AnalysisTab::Results => {
                            if let Some(results) = results {
                                show_analysis_results(ui, state, &analysis);
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No results available - run analysis to see results");
                                });
                            }
                        },
                        AnalysisTab::Visualization => {
                            if let Some(results) = results {
                                show_analysis_visualization(ui, state, &analysis, &results);
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No results available - run analysis to see visualizations");
                                });
                            }
                        },
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an analysis to view details");
                });
            }
        });
    });
}

fn show_analysis_list(ui: &mut egui::Ui, state: &mut AppState) {
    use chrono::DateTime;
    
    ui.vertical(|ui| {
        ui.heading("Analyses");
        ui.add_space(4.0);

        if ui.button("➕ Add Analysis").clicked() {
            state.current_dialog = DialogState::NewAnalysis {
                name: String::new(),
                methods: vec![AnalysisMethod::WorstCase],
                monte_carlo_settings: MonteCarloSettings::default(),
            };
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                let analyses = state.analyses.clone();
                for (index, analysis) in analyses.iter().enumerate() {
                    let is_selected = state.selected_analysis == Some(index);
                    
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        
                        // Format the timestamp if available
                        let timestamp = state.latest_results.get(&analysis.id)
                            .and_then(|r| DateTime::parse_from_rfc3339(&r.timestamp).ok())
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_default();
                        
                        // Create compact display string
                        let display_text = if timestamp.is_empty() {
                            format!(
                                "{}\n{} methods, {} contributions",
                                analysis.name,
                                analysis.methods.len(),
                                analysis.contributions.len()
                            )
                        } else {
                            format!(
                                "{} - Last Run: {}\n{} methods, {} contributions",
                                analysis.name,
                                timestamp,
                                analysis.methods.len(),
                                analysis.contributions.len()
                            )
                        };
                        
                        let response = ui.selectable_label(is_selected, display_text);

                        if response.clicked() {
                            state.selected_analysis = Some(index);
                        }

                        response.context_menu(|ui| {
                            if ui.button("✏ Edit").clicked() {
                                state.current_dialog = DialogState::EditAnalysis {
                                    index,
                                    name: analysis.name.clone(),
                                    methods: analysis.methods.clone(),
                                    monte_carlo_settings: analysis.monte_carlo_settings.clone()
                                        .unwrap_or_default(),
                                };
                                ui.close_menu();
                            }

                            if ui.button("▶ Run Analysis").clicked() {
                                let results = analysis.run_analysis(&state.components);
                                state.latest_results.insert(analysis.id.clone(), results.clone());
                                
                                if let Err(e) = state.file_manager.analysis_handler.save_analysis(
                                    analysis,
                                    &results
                                ) {
                                    state.error_message = Some(format!("Error saving analysis results: {}", e));
                                }
                                ui.close_menu();
                            }

                            ui.separator();

                            if ui.button(egui::RichText::new("🗑 Delete").color(egui::Color32::RED)).clicked() {
                                state.analyses.remove(index);
                                if state.analyses.is_empty() {
                                    state.selected_analysis = None;
                                } else if index >= state.analyses.len() {
                                    state.selected_analysis = Some(state.analyses.len() - 1);
                                }
                                
                                if let Err(e) = state.save_project() {
                                    state.error_message = Some(e.to_string());
                                }
                                ui.close_menu();
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            });
    });
}


fn show_analysis_details(
    ui: &mut egui::Ui, 
    state: &mut AppState, 
    analysis: &StackupAnalysis,
    analysis_index: usize,
) {
    ui.group(|ui| {
        // Analysis header section with edit button
        ui.horizontal(|ui| {
            ui.heading("Analysis Settings");
            if ui.small_button("✏").clicked() {
                state.current_dialog = DialogState::EditAnalysis {
                    index: analysis_index,
                    name: analysis.name.clone(),
                    methods: analysis.methods.clone(),
                    monte_carlo_settings: analysis.monte_carlo_settings.clone()
                        .unwrap_or_default(),
                };
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
                if ui.small_button("➕").clicked() {
                    state.current_dialog = DialogState::NewContribution {
                        analysis_index,
                        component_id: String::new(),
                        feature_id: String::new(),
                        direction: 1.0,
                        half_count: false,
                    };
                }
            });

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 60.0)
                .show(ui, |ui| {
                    for (idx, contrib) in analysis.contributions.iter().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Component and feature info
                                ui.vertical(|ui| {
                                    ui.set_min_width(ui.available_width() - 50.0);
                                    
                                    // Find the actual feature to display its values
                                    if let Some(feature) = find_feature(&state.components, &contrib.component_id, &contrib.feature_id) {
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

                                // Add edit/delete buttons on the right
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("🗑").clicked() {
                                        if let Some(analysis) = state.analyses.get_mut(analysis_index) {
                                            analysis.contributions.remove(idx);
                                            // Save changes
                                            if let Err(e) = state.save_project() {
                                                state.error_message = Some(e.to_string());
                                            }
                                        }
                                    }
                                    if ui.small_button("✏").clicked() {
                                        state.current_dialog = DialogState::EditContribution {
                                            analysis_index,
                                            contribution_index: Some(idx),
                                            component_id: contrib.component_id.clone(),
                                            feature_id: contrib.feature_id.clone(),
                                            direction: contrib.direction,
                                            half_count: contrib.half_count,
                                        };
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


fn show_analysis_results(ui: &mut egui::Ui, state: &mut AppState, analysis: &StackupAnalysis) {
    ui.horizontal(|ui| {
        // Left side - Results display
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() * 0.7);
            
            if let Some(results) = state.latest_results.get(&analysis.id) {
                ui.group(|ui| {
                    ui.heading("Latest Results");
                    ui.add_space(8.0);
                    
                    // Nominal value
                    ui.group(|ui| {
                        ui.heading("Nominal Value");
                        ui.strong(format!("{:.6}", results.nominal));
                    });
                    ui.add_space(8.0);

                    // Analysis results
                    ui.horizontal(|ui| {
                        ui.set_width(ui.available_width());
                        
                        if let Some(wc) = &results.worst_case {
                            ui.vertical(|ui| {
                                ui.group(|ui| {
                                    ui.heading("Worst Case");
                                    ui.label(format!("Min: {:.6}", wc.min));
                                    ui.label(format!("Max: {:.6}", wc.max));
                                    ui.label(format!("Range: {:.6}", wc.max - wc.min));
                                });
                            });

                            ui.add_space(8.0);

                            if let Some(rss) = &results.rss {
                                ui.vertical(|ui| {
                                    ui.group(|ui| {
                                        ui.heading("RSS Analysis");
                                        ui.label(format!("Mean: {:.6}", results.nominal));
                                        ui.label(format!("Std Dev: {:.6}", rss.std_dev));
                                        ui.label(format!("3σ Range: [{:.6}, {:.6}]", rss.min, rss.max));
                                    });
                                });
                            }

                            if let Some(mc) = &results.monte_carlo {
                                ui.vertical(|ui| {
                                    ui.group(|ui| {
                                        ui.heading("Monte Carlo");
                                        ui.label(format!("Mean: {:.6}", mc.mean));
                                        ui.label(format!("Std Dev: {:.6}", mc.std_dev));
                                        ui.label(format!("Range: [{:.6}, {:.6}]", mc.min, mc.max));
                                    });
                                });
                            }
                        }
                    });

                    // Confidence Intervals
                    if let Some(mc) = &results.monte_carlo {
                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.heading("Confidence Intervals");
                            for interval in &mc.confidence_intervals {
                                ui.label(format!(
                                    "{:.1}%: [{:.6}, {:.6}]",
                                    interval.confidence_level * 100.0,
                                    interval.lower_bound,
                                    interval.upper_bound
                                ));
                            }
                        });
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Run analysis to see results");
                });
            }
        });

        // Right side - Results History
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() * 0.3);
            
            if let Ok(metadata) = state.file_manager.analysis_handler.load_metadata(&analysis.id) {
                ui.group(|ui| {
                    ui.heading("Results History");
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            for result_file in metadata.results_files.iter().rev() {
                                let timestamp = result_file.timestamp.format("%Y-%m-%d %H:%M").to_string();
                                let methods = result_file.analysis_methods.iter()
                                    .map(|m| format!("{:?}", m))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                
                                // Highlight current results
                                let is_current = state.latest_results.get(&analysis.id)
                                    .map(|r| r.timestamp == result_file.timestamp.to_rfc3339())
                                    .unwrap_or(false);
                                
                                ui.group(|ui| {
                                    if ui.selectable_label(is_current, format!("{}\n{}", timestamp, methods)).clicked() {
                                        // Load the selected results
                                        let results_path = state.file_manager.analysis_handler
                                            .get_results_file_path(&result_file.path);
                                            
                                        if let Ok(content) = std::fs::read_to_string(&results_path) {
                                            if let Ok(results) = ron::from_str(&content) {
                                                state.latest_results.insert(analysis.id.clone(), results);
                                            }
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        });
                });
            }
        });
    });
}

fn show_analysis_visualization(
    ui: &mut egui::Ui, 
    state: &mut AppState,
    analysis: &StackupAnalysis,
    results: &AnalysisResults,
) {
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
                            .enumerate()
                            .map(|(i, (value, count))| {
                                let bin_start = *value;
                                let bin_end = if i < mc.histogram.len() - 1 {
                                    mc.histogram[i + 1].0
                                } else {
                                    mc.max
                                };
                                
                                    egui_plot::Bar::new(*value, *count as f64)
                                        .width(((mc.max - mc.min) / mc.histogram.len() as f64) * 0.9)
                                        .fill(egui::Color32::from_rgb(100, 150, 255))
                                        .name(format!("Range: {:.3} to {:.3}\nCount: {}", bin_start, bin_end, count))
                                })
                                .collect();
                        
                                plot_ui.bar_chart(
                                    egui_plot::BarChart::new(bars)
                                        .element_formatter(Box::new(|bar, _| {
                                            format!("{}", bar.name)
                                        }))
                                );

                            // Add mean line
                            let mean_line = egui_plot::Line::new(vec![
                                [mc.mean, 0.0],
                                [mc.mean, mc.histogram.iter()
                                    .map(|(_, count)| *count as f64)
                                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                                    .unwrap_or(0.0)],
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
                                if let Some(feature) = find_feature(&state.components, &contrib.component_id, &contrib.feature_id) {
                                    let value = contrib.direction * feature.dimension.value 
                                        * if contrib.half_count { 0.5 } else { 1.0 };
                                    
                                    running_total += value;
                                    
                                    bars.push(egui_plot::Bar::new((i + 1) as f64, value)
                                        .name(&format!("{}.{}", contrib.component_id, contrib.feature_id))
                                        .width(0.5)
                                        .fill(if value >= 0.0 {
                                            egui::Color32::from_rgb(100, 200, 100)
                                        } else {
                                            egui::Color32::from_rgb(200, 100, 100)
                                        }));
                                }
                            }

                            // Final total
                            bars.push(egui_plot::Bar::new(
                                (analysis.contributions.len() + 1) as f64,
                                running_total
                            )
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
}