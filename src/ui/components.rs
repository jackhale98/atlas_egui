// src/ui/components.rs
use eframe::egui;
use crate::app::App;
use crate::config::Feature;
use crate::ui::dialog::{ComponentEditData, DialogState};


// Add to src/ui/components.rs

// Update in src/ui/components.rs
pub fn show_component_edit_dialog(
    ctx: &egui::Context,
    dialog_state: &mut DialogState,
    app: &mut App,
) {
    let mut should_close = false;
    let mut save_changes = false;

    if let DialogState::ComponentEdit(data) = dialog_state {
        let mut open = true;
        let mut temp_name = data.name.clone();
        let mut temp_description = data.description.clone();
        let is_editing = data.is_editing;
        let component_index = data.component_index;

        egui::Window::new("Edit Component")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut temp_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut temp_description);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_changes = true;
                        should_close = true;
                    }

                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });

        if !open {
            should_close = true;
        }

        // Apply changes after the UI is done
        if save_changes {
            if is_editing {
                if let Some(idx) = component_index {
                    if let Some(component) = app.state.project.components.get_mut(idx) {
                        component.name = temp_name;
                        component.description = Some(temp_description);
                    }
                }
            } else {
                app.state.project.components.push(crate::config::Component {
                    name: temp_name,
                    description: Some(temp_description),
                    features: Vec::new(),
                });
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
}

pub fn draw_components_view(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    // Split view into components list and details
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        
        // Left panel - Component List
        egui::ScrollArea::vertical()
            .id_source("components_list")
            .show(ui, |ui| {
                ui.set_min_width(250.0);
                ui.vertical(|ui| {
                    ui.heading("Components");
                    ui.add_space(4.0);

                    // Add Component button
                    if ui.button("Add Component").clicked() {
                        // TODO: Show component creation dialog
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Component list
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

                        // Context menu for component actions
                        response.context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                *dialog_state = DialogState::ComponentEdit(ComponentEditData {
                                    name: component.name.clone(),
                                    description: component.description.clone().unwrap_or_default(),
                                    is_editing: true,
                                    component_index: Some(index),
                                });
                                ui.close_menu();
                            }
                            if ui.button("Add Feature").clicked() {
                                // TODO: Show feature creation dialog
                                ui.close_menu();
                            }
                            if ui.button("View Mates").clicked() {
                                // TODO: Switch to mates view filtered by component
                                ui.close_menu();
                            }
                            ui.separator();
                            let delete_text = format!("Delete '{}'", component.name);
                            if ui.button(egui::RichText::new(delete_text).color(egui::Color32::RED))
                                .clicked() 
                            {
                                // TODO: Show delete confirmation
                                ui.close_menu();
                            }
                        });
                    }
                });
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Right panel - Details View
        if let Some(selected_idx) = app.state.ui.component_list_state.selected() {
            let component = app.state.project.components.get(selected_idx).cloned();
            if let Some(component) = component {
                egui::ScrollArea::vertical()
                    .id_source("component_details")
                    .show(ui, |ui| {
                        draw_component_details(ui, app, &component);
                    });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(32.0);
                ui.weak("Select a component to view details");
            });
        }
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