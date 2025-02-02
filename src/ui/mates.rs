use eframe::egui;
use crate::app::App;
use crate::config::{Component, Feature, mate::FitType};
use crate::state::mate_state::{get_component_by_name, MateFilter};
use crate::ui::dialog::{DialogState, MateEditData};

fn validate_mate(app: &App, mate: &crate::config::mate::Mate) -> Option<crate::config::mate::FitValidation> {
    let feature_a = find_feature(app, &mate.component_a, &mate.feature_a)?;
    let feature_b = find_feature(app, &mate.component_b, &mate.feature_b)?;
    Some(mate.validate(feature_a, feature_b))
}

pub fn draw_mates_view(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    let available_size = ui.available_size();

    egui::Grid::new("mates_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            // Left panel - Mates List
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.4);
                ui.set_min_height(available_size.y);
                
                // Add this section for filter status and controls
                match &app.state.mates.filter {
                    Some(MateFilter::Component(comp)) => {
                        ui.heading(format!("Mates for component {}", comp));
                        if ui.button("🔄 Clear Filter").clicked() {
                            app.state.mates.filter = None;
                            app.state.ui.mate_list_state.select(None);
                        }
                    },
                    Some(MateFilter::Feature(comp, feat)) => {
                        ui.heading(format!("Mates for {}.{}", comp, feat));
                        if ui.button("🔄 Clear Filter").clicked() {
                            app.state.mates.filter = None;
                            app.state.ui.mate_list_state.select(None);
                        }
                    },
                    None => {
                        ui.heading("Mates");
                    }
                }
                
                ui.add_space(4.0);

                // Add Mate button
                if ui.button("➕ Add Mate").clicked() {
                    *dialog_state = DialogState::MateEdit(MateEditData::default());
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Update to use filtered_mates
                egui::ScrollArea::vertical()
                    .id_source("mates_list_scroll")
                    .show(ui, |ui| {
                        let mut delete_index = None;
                        let filtered_mates = app.state.mates.filtered_mates();

                        for (index, mate) in filtered_mates.iter().enumerate() {
                            let feature_a = find_feature(app, &mate.component_a, &mate.feature_a);
                            let feature_b = find_feature(app, &mate.component_b, &mate.feature_b);
                            
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

                            let is_selected = app.state.ui.mate_list_state.selected() == Some(index);
                            
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                let style = if !validation.is_valid {
                                    ui.style_mut().visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(64, 0, 0);
                                };

                                let response = ui.selectable_label(
                                    is_selected,
                                    format!(
                                        "{}.{} ↔ {}.{}\nFit Type: {:?}",
                                        mate.component_a, mate.feature_a,
                                        mate.component_b, mate.feature_b,
                                        mate.fit_type
                                    )
                                );

                                if response.clicked() {
                                    app.state.ui.mate_list_state.select(Some(index));
                                }

                                response.context_menu(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        *dialog_state = DialogState::MateEdit(MateEditData {
                                            component_a: mate.component_a.clone(),
                                            feature_a: mate.feature_a.clone(),
                                            component_b: mate.component_b.clone(),
                                            feature_b: mate.feature_b.clone(),
                                            fit_type: mate.fit_type.clone(),
                                            is_editing: true,
                                            mate_index: Some(index),
                                        });
                                        ui.close_menu();
                                    }
                                    
                                    ui.separator();
                                    
                                    if ui.button(egui::RichText::new("🗑 Delete")
                                        .color(egui::Color32::RED)).clicked() 
                                    {
                                        delete_index = Some(index);
                                        ui.close_menu();
                                    }
                                });

                                if !validation.is_valid {
                                    if let Some(error) = validation.error_message {
                                        ui.colored_label(egui::Color32::RED, error);
                                    }
                                }
                            });
                            ui.add_space(4.0);
                        }

                        if let Some(index) = delete_index {
                            app.state.mates.mates.remove(index);
                            
                            // Update selection after deletion
                            if app.state.mates.mates.is_empty() {
                                app.state.ui.mate_list_state.select(None);
                            } else if index >= app.state.mates.mates.len() {
                                app.state.ui.mate_list_state
                                    .select(Some(app.state.mates.mates.len() - 1));
                            }

                            // Save changes
                            let mates_file = crate::file::mates::MatesFile {
                                version: "1.0.0".to_string(),
                                mates: app.state.mates.mates.clone(),
                            };
                            if let Err(e) = app.state.file_manager.save_mates(&mates_file) {
                                println!("Error saving mates: {}", e);
                            }
                        }
                    });
            });

            // Right panel - Mate Details
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.6);
                ui.set_min_height(available_size.y);

                if let Some(selected) = app.state.ui.mate_list_state.selected() {
                    if let Some(mate) = app.state.mates.mates.get(selected) {
                        let feature_a = find_feature(app, &mate.component_a, &mate.feature_a);
                        let feature_b = find_feature(app, &mate.component_b, &mate.feature_b);

                        if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                            ui.heading("Mate Details");
                            ui.add_space(8.0);

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
                            ui.colored_label(
                                egui::Color32::RED,
                                "⚠ One or more features not found"
                            );
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

pub fn show_mate_edit_dialog(
    ctx: &egui::Context,
    dialog_state: &mut DialogState,
    app: &mut App,
) {
    let mut should_close = false;
    let mut save_changes = false;

    if let DialogState::MateEdit(data) = dialog_state {
        let title = if data.is_editing { "Edit Mate" } else { "New Mate" };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .fixed_size([400.0, 400.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Component A selection
                    ui.group(|ui| {
                        ui.heading("Component A");
                        ui.push_id("component_a_selection", |ui| {
                            egui::ComboBox::from_label("Select Component")
                                .selected_text(&data.component_a)
                                .show_ui(ui, |ui| {
                                    for component in &app.state.project.components {
                                        ui.selectable_value(
                                            &mut data.component_a,
                                            component.name.clone(),
                                            &component.name
                                        );
                                    }
                                });
                        });

                        ui.push_id("feature_a_selection", |ui| {
                            if let Some(component) = get_component_by_name(
                                &app.state.project.components,
                                &data.component_a
                            ) {
                                egui::ComboBox::from_label("Select Feature")
                                    .selected_text(&data.feature_a)
                                    .show_ui(ui, |ui| {
                                        for feature in &component.features {
                                            ui.selectable_value(
                                                &mut data.feature_a,
                                                feature.name.clone(),
                                                &feature.name
                                            );
                                        }
                                    });
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Component B selection
                    ui.group(|ui| {
                        ui.heading("Component B");
                        egui::ComboBox::from_label("Select Component")
                            .selected_text(&data.component_b)
                            .show_ui(ui, |ui| {
                                for component in &app.state.project.components {
                                    ui.selectable_value(
                                        &mut data.component_b,
                                        component.name.clone(),
                                        &component.name
                                    );
                                }
                            });

                        if let Some(component) = get_component_by_name(
                            &app.state.project.components,
                            &data.component_b
                        ) {
                            egui::ComboBox::from_label("Select Feature")
                                .selected_text(&data.feature_b)
                                .show_ui(ui, |ui| {
                                    for feature in &component.features {
                                        ui.selectable_value(
                                            &mut data.feature_b,
                                            feature.name.clone(),
                                            &feature.name
                                        );
                                    }
                                });
                        }
                    });

                    ui.add_space(8.0);

                    // Fit Type selection
                    ui.group(|ui| {
                        ui.heading("Fit Type");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut data.fit_type, FitType::Clearance, "Clearance");
                            ui.radio_value(&mut data.fit_type, FitType::Transition, "Transition");
                            ui.radio_value(&mut data.fit_type, FitType::Interference, "Interference");
                        });
                    });

                    ui.add_space(16.0);

                    // Preview/Validation
                    if let (Some(feature_a), Some(feature_b)) = (
                        find_feature(app, &data.component_a, &data.feature_a),
                        find_feature(app, &data.component_b, &data.feature_b)
                    ) {
                        let validation = crate::config::mate::Mate::new(
                            uuid::Uuid::new_v4().to_string(),
                            data.component_a.clone(),
                            data.feature_a.clone(),
                            data.component_b.clone(),
                            data.feature_b.clone(),
                            data.fit_type.clone()
                        ).validate(feature_a, feature_b);

                        let validation_color = if validation.is_valid {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };

                        ui.group(|ui| {
                            if !validation.is_valid {
                                // Use a warning color and show the error message
                                ui.colored_label(
                                    egui::Color32::YELLOW, 
                                    format!("⚠ Warning: {}", validation.error_message.unwrap_or_default())
                                );
                            }
                    
                            // Always show fit details
                            ui.label(format!(
                                "Nominal Fit: {:.3}\nMin Fit: {:.3}\nMax Fit: {:.3}",
                                validation.nominal_fit,
                                validation.min_fit,
                                validation.max_fit
                            ));
                        });
                        let can_save = true;
                    }

                    ui.add_space(16.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_save = !data.component_a.is_empty() 
                            && !data.feature_a.is_empty()
                            && !data.component_b.is_empty() 
                            && !data.feature_b.is_empty();

                        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                            save_changes = true;
                            should_close = true;
                        }

                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        // Apply changes after UI
        if save_changes {
            let new_mate = crate::config::mate::Mate::new(
                uuid::Uuid::new_v4().to_string(),
                data.component_a.clone(),
                data.feature_a.clone(),
                data.component_b.clone(),
                data.feature_b.clone(),
                data.fit_type.clone()
            );

            if data.is_editing {
                if let Some(idx) = data.mate_index {
                    if let Some(mate) = app.state.mates.mates.get_mut(idx) {
                        *mate = new_mate;
                    }
                }
            } else {
                app.state.mates.mates.push(new_mate);
            }

            // Update dependency graph
            app.state.mates.update_dependency_graph(&app.state.project.components);

            // Save mates file
            let mates_file = crate::file::mates::MatesFile {
                version: "1.0.0".to_string(),
                mates: app.state.mates.mates.clone(),
            };
            if let Err(e) = app.state.file_manager.save_mates(&mates_file) {
                println!("Error saving mates: {}", e);
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
}

fn find_feature<'a>(
    app: &'a App,
    component_name: &str,
    feature_name: &str
) -> Option<&'a Feature> {
    app.state.project.components.iter()
        .find(|c| c.name == component_name)?
        .features.iter()
        .find(|f| f.name == feature_name)
}