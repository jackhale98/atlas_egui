// src/ui/mates.rs
use eframe::egui;
use crate::state::{AppState, DialogState, Screen};
use crate::utils::find_feature;

pub fn show_mates_view(ui: &mut egui::Ui, state: &mut AppState) {
    let available_size = ui.available_size();

    egui::Grid::new("mates_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            // Left panel - Mates List
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.4);
                ui.set_min_height(available_size.y);
                
                // Header with potential filter info
                ui.heading("Mates");
                if !state.components.is_empty() {
                    if ui.button("➕ Add Mate").clicked() {
                        state.current_dialog = DialogState::NewMate {
                            component_a: String::new(),
                            feature_a: String::new(),
                            component_b: String::new(),
                            feature_b: String::new(),
                        };
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_source("mates_list_scroll")
                    .show(ui, |ui| {
                        let mates = state.mates.clone(); // Clone to avoid borrow checker issues
                        for (index, mate) in mates.iter().enumerate() {
                            let is_selected = state.selected_mate == Some(index);
                            let feature_a = find_feature(&state.components, &mate.component_a, &mate.feature_a);
                            let feature_b = find_feature(&state.components, &mate.component_b, &mate.feature_b);
                            
                            let validation = if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                                mate.validate(feat_a, feat_b)
                            } else {
                                crate::config::mate::FitValidation {
                                    is_valid: false,
                                    nominal_fit: 0.0,
                                    min_fit: 0.0,
                                    max_fit: 0.0,
                                    error_message: Some("Missing features".to_string()),
                                }
                            };
                            
                            ui.group(|ui| {
                                if !validation.is_valid {
                                    ui.style_mut().visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(64, 0, 0);
                                }

                                let response = ui.selectable_label(
                                    is_selected,
                                    format!(
                                        "{}.{} ↔ {}.{}\n{:?} Fit",
                                        mate.component_a, mate.feature_a,
                                        mate.component_b, mate.feature_b,
                                        mate.fit_type
                                    )
                                );

                                if response.clicked() {
                                    state.selected_mate = Some(index);
                                }

                                response.context_menu(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        state.current_dialog = DialogState::EditMate {
                                            index,
                                            component_a: mate.component_a.clone(),
                                            feature_a: mate.feature_a.clone(),
                                            component_b: mate.component_b.clone(),
                                            feature_b: mate.feature_b.clone(),
                                        };
                                        ui.close_menu();
                                    }
                                    
                                    if ui.button("🔍 Show Component A").clicked() {
                                        if let Some(comp_idx) = state.components
                                            .iter()
                                            .position(|c| c.name == mate.component_a) 
                                        {
                                            state.selected_component = Some(comp_idx);
                                            state.current_screen = Screen::Components;
                                        }
                                        ui.close_menu();
                                    }

                                    if ui.button("🔍 Show Component B").clicked() {
                                        if let Some(comp_idx) = state.components
                                            .iter()
                                            .position(|c| c.name == mate.component_b) 
                                        {
                                            state.selected_component = Some(comp_idx);
                                            state.current_screen = Screen::Components;
                                        }
                                        ui.close_menu();
                                    }
                                    
                                    ui.separator();
                                    
                                    let delete_clicked = ui.button(
                                        egui::RichText::new("🗑 Delete").color(egui::Color32::RED)
                                    ).clicked();
                                    
                                    if delete_clicked {
                                        let state_ptr = state as *mut AppState;
                                        unsafe {
                                            (*state_ptr).mates.remove(index);
                                            (*state_ptr).update_mate_graph();
                                            
                                            if (*state_ptr).mates.is_empty() {
                                                (*state_ptr).selected_mate = None;
                                            } else if index >= (*state_ptr).mates.len() {
                                                (*state_ptr).selected_mate = Some((*state_ptr).mates.len() - 1);
                                            }

                                            if let Err(e) = (*state_ptr).save_project() {
                                                (*state_ptr).error_message = Some(e.to_string());
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                });

                                if !validation.is_valid {
                                    if let Some(error) = &validation.error_message {
                                        ui.colored_label(egui::Color32::RED, error);
                                    }
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });
            });

            // Right panel - Mate Details
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.6);
                ui.set_min_height(available_size.y);

                if let Some(selected_idx) = state.selected_mate {
                    if let Some(mate) = state.mates.get(selected_idx) {
                        let feature_a = find_feature(&state.components, &mate.component_a, &mate.feature_a);
                        let feature_b = find_feature(&state.components, &mate.component_b, &mate.feature_b);

                        ui.heading("Mate Details");
                        ui.add_space(8.0);

                        if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                            // Feature A details
                            ui.group(|ui| {
                                ui.heading(&format!("Component A: {}", mate.component_a));
                                ui.label(&format!("Feature: {} ({:?})", 
                                    feat_a.name, feat_a.feature_type));
                                ui.horizontal(|ui| {
                                    ui.label("Nominal:");
                                    ui.strong(&format!("{:.3}", feat_a.dimension.value));
                                    ui.label("Tolerances:");
                                    ui.strong(&format!("[{:+.3}/{:+.3}]",
                                        feat_a.dimension.plus_tolerance,
                                        feat_a.dimension.minus_tolerance));
                                });
                            });

                            ui.add_space(8.0);

                            // Feature B details
                            ui.group(|ui| {
                                ui.heading(&format!("Component B: {}", mate.component_b));
                                ui.label(&format!("Feature: {} ({:?})", 
                                    feat_b.name, feat_b.feature_type));
                                ui.horizontal(|ui| {
                                    ui.label("Nominal:");
                                    ui.strong(&format!("{:.3}", feat_b.dimension.value));
                                    ui.label("Tolerances:");
                                    ui.strong(&format!("[{:+.3}/{:+.3}]",
                                        feat_b.dimension.plus_tolerance,
                                        feat_b.dimension.minus_tolerance));
                                });
                            });

                            ui.add_space(16.0);

                            // Fit Analysis
                            ui.group(|ui| {
                                ui.heading(&format!("Fit Analysis ({:?})", mate.fit_type));
                                
                                let nominal_fit = mate.calculate_nominal_fit(feat_a, feat_b);
                                let min_fit = mate.calculate_min_fit(feat_a, feat_b);
                                let max_fit = mate.calculate_max_fit(feat_a, feat_b);
                                let validation = mate.validate(feat_a, feat_b);

                                ui.horizontal(|ui| {
                                    ui.label("Nominal Fit:");
                                    ui.strong(&format!("{:.3}", nominal_fit));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Minimum Fit:");
                                    ui.strong(&format!("{:.3}", min_fit));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Maximum Fit:");
                                    ui.strong(&format!("{:.3}", max_fit));
                                });

                                ui.add_space(8.0);
                                
                                // Validation status
                                if validation.is_valid {
                                    ui.colored_label(egui::Color32::GREEN, "✓ Valid fit");
                                } else if let Some(error) = validation.error_message {
                                    ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
                                }
                            });
                        } else {
                            ui.colored_label(egui::Color32::RED, "One or more features not found");
                            if feature_a.is_none() {
                                ui.label(format!("Missing feature: {}.{}", mate.component_a, mate.feature_a));
                            }
                            if feature_b.is_none() {
                                ui.label(format!("Missing feature: {}.{}", mate.component_b, mate.feature_b));
                            }
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a mate to view details");
                    });
                }
            });
        });
}