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
            ui.set_width(ui.available_width() * 0.25);
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
                                "{}\nLast Run: {}\n{} methods, {} contributions",
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
    // Main layout - vertical with Latest Results on top, History on bottom
    ui.vertical(|ui| {
        // Top section - Latest Results
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            
            // Clone the results if available (to avoid borrow issues)
            let results_clone = state.latest_results.get(&analysis.id).cloned();
            
            // Run Analysis button (outside of any closures to avoid borrow conflicts)
            ui.horizontal(|ui| {
                ui.heading("Analysis Results");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("▶ Run Analysis").clicked() {
                        let new_results = analysis.run_analysis(&state.components);
                        
                        if let Err(e) = state.file_manager.analysis_handler.save_analysis(
                            analysis,
                            &new_results
                        ) {
                            state.error_message = Some(format!("Error saving analysis results: {}", e));
                        }
                        
                        // Update results after saving
                        state.latest_results.insert(analysis.id.clone(), new_results);
                    }
                });
            });
            
            ui.add_space(8.0);
            
            if let Some(results) = results_clone {
                // Nominal value
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("Nominal Value");
                            ui.strong(format!("{:.6}", results.nominal));
                        });
                    });
                });
                
                ui.add_space(8.0);

                // Analysis results in a horizontal layout
                ui.horizontal(|ui| {
                    if let Some(wc) = &results.worst_case {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.heading("Worst Case");
                                ui.label(format!("Min: {:.6}", wc.min));
                                ui.label(format!("Max: {:.6}", wc.max));
                                ui.label(format!("Range: {:.6}", wc.max - wc.min));
                            });
                        });

                        if let Some(rss) = &results.rss {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.heading("RSS Analysis");
                                    ui.label(format!("Mean: {:.6}", results.nominal));
                                    ui.label(format!("Std Dev: {:.6}", rss.std_dev));
                                    ui.label(format!("3σ Range: [{:.6}, {:.6}]", rss.min, rss.max));
                                });
                            });
                        }

                        if let Some(mc) = &results.monte_carlo {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.heading("Monte Carlo");
                                    ui.label(format!("Mean: {:.6}", mc.mean));
                                    ui.label(format!("Std Dev: {:.6}", mc.std_dev));
                                    ui.label(format!("Range: [{:.6}, {:.6}]", mc.min, mc.max));
                                });
                            });
                        }
                    }
                });

                // Process Capability section
                if let Some(process_cap) = &results.process_capability {
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Process Capability");
                            
                            // Specification limits
                            ui.horizontal(|ui| {
                                ui.label("Specification Limits:");
                                ui.add_space(5.0);
                                
                                if let Some(lsl) = process_cap.lower_spec {
                                    ui.label(format!("LSL: {:.6}", lsl));
                                } else {
                                    ui.label("LSL: —");
                                }
                                
                                ui.add_space(20.0);
                                
                                if let Some(usl) = process_cap.upper_spec {
                                    ui.label(format!("USL: {:.6}", usl));
                                } else {
                                    ui.label("USL: —");
                                }
                            });
                            
                            // Capability indices
                            ui.horizontal(|ui| {
                                ui.label("Capability Indices:");
                                ui.add_space(5.0);
                                
                                if let Some(cp) = process_cap.cp {
                                    let color = if cp >= 1.33 {
                                        egui::Color32::GREEN
                                    } else if cp >= 1.0 {
                                        egui::Color32::YELLOW
                                    } else {
                                        egui::Color32::RED
                                    };
                                    ui.colored_label(color, format!("Cp: {:.3}", cp));
                                }
                                
                                ui.add_space(20.0);
                                
                                if let Some(cpk) = process_cap.cpk {
                                    let color = if cpk >= 1.33 {
                                        egui::Color32::GREEN
                                    } else if cpk >= 1.0 {
                                        egui::Color32::YELLOW
                                    } else {
                                        egui::Color32::RED
                                    };
                                    ui.colored_label(color, format!("Cpk: {:.3}", cpk));
                                }
                            });
                            
                            // PPM Values
                            if let (Some(ppm_below), Some(ppm_above)) = (process_cap.ppm_below, process_cap.ppm_above) {
                                ui.horizontal(|ui| {
                                    ui.label("Expected PPM:");
                                    ui.add_space(5.0);
                                    ui.label(format!("Below: {:.1}", ppm_below));
                                    ui.add_space(20.0);
                                    ui.label(format!("Above: {:.1}", ppm_above));
                                    ui.add_space(20.0);
                                    ui.label(format!("Total: {:.1}", ppm_below + ppm_above));
                                });
                            }
                            
                            // PPH Values
                            if let (Some(pph_below), Some(pph_above)) = (process_cap.pph_below, process_cap.pph_above) {
                                ui.horizontal(|ui| {
                                    ui.label("Expected PPH:");
                                    ui.add_space(5.0);
                                    ui.label(format!("Below: {:.1}", pph_below));
                                    ui.add_space(20.0);
                                    ui.label(format!("Above: {:.1}", pph_above));
                                    ui.add_space(20.0);
                                    ui.label(format!("Total: {:.1}", pph_below + pph_above));
                                });
                            }
                        });
                    });
                }

                // Confidence Intervals
                if let Some(mc) = &results.monte_carlo {
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.vertical(|ui| {
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
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No results available - run analysis to see results");
                });
            }
        });

        ui.add_space(8.0);

        // Bottom section - Results History
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.heading("Results History");
            
            // Create a separate scope for working with metadata to avoid borrow issues
            let analysis_id = analysis.id.clone();
            let latest_results_timestamp = state.latest_results.get(&analysis_id)
                .map(|r| r.timestamp.clone());
                
            let metadata_result = state.file_manager.analysis_handler.load_metadata(&analysis_id);
            
            match metadata_result {
                Ok(metadata) => {
                    if metadata.results_files.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No analysis history available");
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(150.0) // Fixed max height for history section
                            .show(ui, |ui| {
                                // Table-like layout for history items
                                ui.horizontal(|ui| {
                                    ui.strong("Date & Time");
                                    ui.add_space(20.0);
                                    ui.strong("Methods Used");
                                    ui.add_space(20.0);
                                    ui.strong("Actions");
                                });
                                
                                ui.separator();
                                
                                for result_file in metadata.results_files.iter().rev() {
                                    let timestamp = result_file.timestamp.format("%Y-%m-%d %H:%M").to_string();
                                    let methods = result_file.analysis_methods.iter()
                                        .map(|m| format!("{:?}", m))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    
                                    // Highlight current results
                                    let is_current = latest_results_timestamp.as_ref()
                                        .map(|t| t == &result_file.timestamp.to_rfc3339())
                                        .unwrap_or(false);
                                    
                                    let text_color = if is_current { 
                                        egui::Color32::from_rgb(100, 200, 100) 
                                    } else { 
                                        ui.style().visuals.text_color() 
                                    };
                                    
                                    ui.horizontal(|ui| {
                                        ui.colored_label(text_color, timestamp);
                                        ui.add_space(20.0);
                                        ui.colored_label(text_color, methods);
                                        
                                        // Use all remaining space
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if !is_current {
                                                if ui.button("Load").clicked() {
                                                    // Load the selected results
                                                    let results_path = state.file_manager.analysis_handler
                                                        .get_results_file_path(&result_file.path);
                                                        
                                                    if let Ok(content) = std::fs::read_to_string(&results_path) {
                                                        if let Ok(results) = ron::from_str(&content) {
                                                            state.latest_results.insert(analysis_id.clone(), results);
                                                        }
                                                    }
                                                }
                                            } else {
                                                ui.label("Current");
                                            }
                                        });
                                    });
                                    
                                    ui.separator();
                                }
                            });
                    }
                },
                Err(_) => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Run analysis to create history");
                    });
                }
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