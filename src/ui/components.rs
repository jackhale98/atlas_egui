// src/ui/components.rs
use eframe::egui;
use crate::app::App;
use crate::config::Feature;
use crate::ui::dialog::{ComponentEditData, DialogState};



// In src/ui/components.rs
pub fn show_component_edit_dialog(
    ctx: &egui::Context,
    dialog_state: &mut DialogState,
    app: &mut App,
) {
    let mut should_close = false;
    let mut save_changes = false;

    if let DialogState::ComponentEdit(data) = dialog_state {
        let title = if data.is_editing { "Edit Component" } else { "New Component" };
        
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 200.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let name_valid = !data.name.trim().is_empty();
                    let revision_valid = !data.revision.trim().is_empty();

                    // Name field
                    ui.horizontal(|ui| {
                        ui.label("Name:").on_hover_text("Component name");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut data.name)
                                .desired_width(200.0)
                                .hint_text("Enter component name")
                        );
                        if !name_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    // Revision field
                    ui.horizontal(|ui| {
                        ui.label("Rev:").on_hover_text("Component revision");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut data.revision)
                                .desired_width(200.0)
                                .hint_text("Enter revision (e.g. A, B, 01)")
                        );
                        if !revision_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    // Description field
                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.add(
                            egui::TextEdit::multiline(&mut data.description)
                                .desired_width(200.0)
                                .desired_rows(3)
                                .hint_text("Enter component description")
                        );
                    });

                    ui.add_space(8.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }

                            let can_save = name_valid && revision_valid;
                            if ui.add_enabled(
                                can_save,
                                egui::Button::new(egui::RichText::new("Save").strong())
                            ).clicked() {
                                save_changes = true;
                                should_close = true;
                            }
                        });
                    });

                    // Validation message
                    if !name_valid || !revision_valid {
                        ui.add_space(4.0);
                        ui.colored_label(
                            egui::Color32::RED,
                            "Name and revision are required"
                        );
                    }
                });
            });

        // Apply changes after the UI is done
        if save_changes {
            let full_name = format!("{} Rev {}", data.name.trim(), data.revision.trim());
            
            if data.is_editing {
                if let Some(idx) = data.component_index {
                    if let Some(component) = app.state.project.components.get_mut(idx) {
                        component.name = full_name;
                        component.description = Some(data.description.clone());
                    }
                }
            } else {
                app.state.project.components.push(crate::config::Component {
                    name: full_name,
                    description: Some(data.description.clone()),
                    features: Vec::new(),
                });
            }

            // Optionally trigger a save of the project file here
            if let Err(e) = app.state.file_manager.save_project(
                &app.state.project.project_file,
                &app.state.project.components,
                &app.state.analysis.analyses,
            ) {
                // TODO: Handle save error - maybe add an error message to the UI
                println!("Error saving project: {}", e);
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
}

// In src/ui/components.rs
// In src/ui/components.rs
pub fn draw_components_view(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    egui::Grid::new("components_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            // Left panel - Component List
            ui.vertical(|ui| {
                ui.set_min_width(250.0);
                
                ui.heading("Components");
                ui.add_space(4.0);

                // Add Component button at top
                if ui.button("➕ Add Component").clicked() {
                    *dialog_state = DialogState::ComponentEdit(ComponentEditData {
                        name: String::new(),
                        revision: String::from("A"),
                        description: String::new(),
                        is_editing: false,
                        component_index: None,
                    });
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Component list with scrolling
                egui::ScrollArea::vertical()
                    .id_source("components_list_scroll")  // Add unique ID
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        let mut delete_index = None;
                        
                        for (index, component) in app.state.project.components.iter().enumerate() {
                            let is_selected = Some(index) == app.state.ui.component_list_state.selected();
                            let response = ui.selectable_label(
                                is_selected,
                                format!("{} ({} features)", component.name, component.features.len())
                            );

                            if response.clicked() {
                                app.state.ui.component_list_state.select(Some(index));
                                app.state.ui.feature_list_state.select(None);
                            }

                            response.context_menu(|ui| {
                                if ui.button("✏ Edit").clicked() {
                                    let (name, revision) = if let Some(rev_idx) = component.name.rfind(" Rev ") {
                                        let (name, rev) = component.name.split_at(rev_idx);
                                        (name.to_string(), rev.replace(" Rev ", ""))
                                    } else {
                                        (component.name.clone(), "A".to_string())
                                    };

                                    *dialog_state = DialogState::ComponentEdit(ComponentEditData {
                                        name,
                                        revision,
                                        description: component.description.clone().unwrap_or_default(),
                                        is_editing: true,
                                        component_index: Some(index),
                                    });
                                    ui.close_menu();
                                }

                                if ui.button("🔧 Add Feature").clicked() {
                                    // TODO: Will implement feature dialog
                                    ui.close_menu();
                                }

                                ui.separator();

                                let delete_text = format!("🗑 Delete '{}'", component.name);
                                if ui.button(egui::RichText::new(delete_text).color(egui::Color32::RED))
                                    .clicked() 
                                {
                                    delete_index = Some(index);
                                    ui.close_menu();
                                }
                            });
                        }

                        // Handle deletion after the iteration
                        if let Some(index) = delete_index {
                            app.state.project.components.remove(index);
                            
                            // Update selection
                            if app.state.project.components.is_empty() {
                                app.state.ui.component_list_state.select(None);
                            } else if index >= app.state.project.components.len() {
                                app.state.ui.component_list_state.select(Some(app.state.project.components.len() - 1));
                            }

                            // Save changes
                            if let Err(e) = app.state.file_manager.save_project(
                                &app.state.project.project_file,
                                &app.state.project.components,
                                &app.state.analysis.analyses,
                            ) {
                                println!("Error saving project after delete: {}", e);
                            }
                        }
                    });
            });

            // Right panel - Component Details & Features
            ui.vertical(|ui| {
                if let Some(selected_idx) = app.state.ui.component_list_state.selected() {
                    if let Some(component) = app.state.project.components.get(selected_idx) {
                        // Component details section
                        ui.heading(&component.name);
                        if let Some(desc) = &component.description {
                            ui.label(desc);
                        }
                        ui.add_space(16.0);

                        // Features section
                        ui.heading("Features");
                        ui.add_space(4.0);
                        
                        if ui.button("➕ Add Feature").clicked() {
                            // TODO: Show feature creation dialog
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Features list with scrolling
                        egui::ScrollArea::vertical()
                            .id_source("features_list_scroll") 
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                for (index, feature) in component.features.iter().enumerate() {
                                    let is_selected = app.state.ui.feature_list_state.selected() == Some(index);
                                    ui.group(|ui| {
                                        let response = ui.selectable_label(
                                            is_selected,
                                            format!("{} ({:?})", feature.name, feature.feature_type)
                                        );

                                        if response.clicked() {
                                            app.state.ui.feature_list_state.select(Some(index));
                                        }

                                        if is_selected {
                                            ui.add_space(4.0);
                                            ui.horizontal(|ui| {
                                                ui.label("Value:");
                                                ui.strong(format!("{:.3}", feature.dimension.value));
                                                ui.label("Tolerances:");
                                                ui.strong(format!("[{:+.3}/{:+.3}]", 
                                                    feature.dimension.plus_tolerance,
                                                    feature.dimension.minus_tolerance));
                                            });

                                            if let Some(dist) = &feature.distribution {
                                                ui.label(format!("Distribution: {:?}", dist));
                                            }
                                        }
                                    });
                                }
                            });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a component to view details");
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a component to view details");
                    });
                }
            });
        });
}

fn draw_component_details(ui: &mut egui::Ui, app: &mut App, component: &crate::config::Component) {
    ui.heading(&component.name);
    
    if let Some(desc) = &component.description {
        ui.label(desc);
    }
    
    ui.add_space(16.0);
    
    // Features section
    ui.heading("Features");
    ui.add_space(4.0);

    // Add Feature button
    if ui.button("Add Feature").clicked() {
        // TODO: Show feature creation dialog
    }

    ui.add_space(8.0);

    // Features list
    for (index, feature) in component.features.iter().enumerate() {
        let is_selected = app.state.ui.feature_list_state.selected() == Some(index);
        
        ui.group(|ui| {
            let response = ui.selectable_label(
                is_selected,
                format!("{} ({:?})", feature.name, feature.feature_type)
            );

            if response.clicked() {
                app.state.ui.feature_list_state.select(Some(index));
            }

            // Feature details if selected
            if is_selected {
                ui.add_space(4.0);
                draw_feature_details(ui, feature);
            }

            // Context menu for feature actions
            response.context_menu(|ui| {
                if ui.button("Edit").clicked() {
                    // TODO: Show feature edit dialog
                    ui.close_menu();
                }
                if ui.button("View Mates").clicked() {
                    // TODO: Switch to mates view filtered by feature
                    ui.close_menu();
                }
                ui.separator();
                let delete_text = format!("Delete '{}'", feature.name);
                if ui.button(egui::RichText::new(delete_text).color(egui::Color32::RED))
                    .clicked() 
                {
                    // TODO: Show delete confirmation
                    ui.close_menu();
                }
            });
        });
    }
}

fn draw_feature_details(ui: &mut egui::Ui, feature: &Feature) {
    ui.horizontal(|ui| {
        ui.label("Value:");
        ui.strong(format!("{:.3}", feature.dimension.value));
        ui.label("Tolerances:");
        ui.strong(format!("[{:+.3}/{:+.3}]", 
            feature.dimension.plus_tolerance,
            feature.dimension.minus_tolerance));
    });

    if let Some(dist) = &feature.distribution {
        ui.label(format!("Distribution: {:?}", dist));
    }

    // TODO: Add small preview of related mates
}