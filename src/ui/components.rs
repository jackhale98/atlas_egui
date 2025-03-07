// src/ui/components.rs
use eframe::egui;
use crate::app::App;
use crate::config::{Feature, FeatureType};
use crate::ui::dialog::{ComponentEditData, DialogState, FeatureEditData};
use crate::config::ComponentReference;
use crate::analysis::DistributionType;
use crate::state::mate_state::MateFilter;
use crate::state::ui_state::ScreenMode;


fn find_feature<'a>(
    app: &'a App,
    component_name: &str,
    feature_name: &str,
) -> Option<&'a Feature> {
    app.state.project.components
        .iter()
        .find(|c| c.name == component_name)?
        .features
        .iter()
        .find(|f| f.name == feature_name)
}

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
                    // Get the old component name before updating
                    let old_name = app.state.project.components[idx].name.clone();
                    let old_filename = format!("components/{}.ron", old_name.to_lowercase().replace(" ", "_"));
        
                    // Update component
                    if let Some(component) = app.state.project.components.get_mut(idx) {
                        component.name = full_name.clone();
                        component.description = Some(data.description.clone());
                    }
        
                    // Update reference in project file
                    let new_filename = format!("components/{}.ron", full_name.to_lowercase().replace(" ", "_"));
                    if let Some(reference) = app.state.project.project_file.component_references
                        .iter_mut()
                        .find(|r| r.path == old_filename)
                    {
                        reference.path = new_filename;
                    }
                }
            } else {
                // Create the new component
                app.state.project.components.push(crate::config::Component {
                    name: full_name.clone(),
                    description: Some(data.description.clone()),
                    features: Vec::new(),
                });
        
                // Update the component references in the project file
                let filename = format!("{}.ron", full_name.to_lowercase().replace(" ", "_"));
                let rel_path = format!("components/{}", filename).replace('\\', "/");
                
                // Add the new component reference if it doesn't exist
                if !app.state.project.project_file.component_references
                    .iter()
                    .any(|r| r.path == rel_path)
                {
                    app.state.project.project_file.component_references
                        .push(ComponentReference { path: rel_path });
                }
            }
        
            // Save all changes
            if let Err(e) = app.state.file_manager.save_project(
                &app.state.project.project_file,
                &app.state.project.components,
                &app.state.analysis.analyses,
            ) {
                println!("Error saving project: {}", e);
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
}

pub fn draw_components_view(ui: &mut egui::Ui, app: &mut App, dialog_state: &mut DialogState) {
    let available_size = ui.available_size();

    egui::Grid::new("components_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            // Left panel - Component List
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.3);
                ui.set_min_height(available_size.y);
                
                ui.heading("Components");
                ui.add_space(4.0);

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

                egui::ScrollArea::vertical()
                    .id_source("components_list_scroll")
                    .show(ui, |ui| {
                        let mut delete_index = None;
                        let mut switch_to_mates = false;
                        let mut new_filter = None;
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

                                if ui.button("🔍 Show All Mates").clicked() {
                                    switch_to_mates = true;
                                    new_filter = Some(MateFilter::Component(component.name.clone()));
                                    ui.close_menu();
                                }
                
                                ui.separator();

                                if let Some(selected_idx) = app.state.ui.component_list_state.selected() {
                                    if ui.button("➕ Add Feature").clicked() {
                                        *dialog_state = DialogState::FeatureEdit(FeatureEditData {
                                            name: String::new(),
                                            feature_type: FeatureType::default(),
                                            value: String::new(),
                                            plus_tolerance: String::new(),
                                            minus_tolerance: String::new(),
                                            distribution: DistributionType::default(),
                                            is_editing: false,
                                            feature_index: None,
                                            component_index: Some(selected_idx),
                                        });
                                    }
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

                        if let Some(filter) = new_filter {
                            app.state.mates.filter = Some(filter);
                        }
                        if switch_to_mates {
                            app.switch_tab(ScreenMode::Mates);
                        }

                        if let Some(index) = delete_index {
                            app.state.project.components.remove(index);
                            if app.state.project.components.is_empty() {
                                app.state.ui.component_list_state.select(None);
                            } else if index >= app.state.project.components.len() {
                                app.state.ui.component_list_state.select(Some(app.state.project.components.len() - 1));
                            }
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
                ui.set_min_width(available_size.x * 0.7);
                ui.set_min_height(available_size.y);

                if let Some(selected_idx) = app.state.ui.component_list_state.selected() {
                    let component_name = app.state.project.components[selected_idx].name.clone();
                    let component_desc = app.state.project.components[selected_idx].description.clone();
                    let features_display: Vec<_> = app.state.project.components[selected_idx]
                        .features
                        .iter()
                        .map(|f| (
                            f.name.clone(),
                            f.feature_type,
                            f.dimension.value,
                            f.dimension.plus_tolerance,
                            f.dimension.minus_tolerance,
                            f.distribution
                        ))
                        .collect();

                    ui.heading(&component_name);
                    if let Some(desc) = &component_desc {
                        ui.label(desc);
                    }
                    ui.add_space(16.0);

                    ui.heading("Features");
                    ui.add_space(4.0);
                    
                    if ui.button("➕ Add Feature").clicked() {
                        *dialog_state = DialogState::FeatureEdit(FeatureEditData {
                            name: String::new(),
                            feature_type: FeatureType::default(),
                            value: String::new(),
                            plus_tolerance: String::new(),
                            minus_tolerance: String::new(),
                            distribution: DistributionType::default(),
                            is_editing: false,
                            feature_index: None,
                            component_index: Some(selected_idx),
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_source("features_list_scroll")
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            let mut delete_index = None;
                            let mut switch_to_mates = false;
                            let mut new_filter = None;
                            
                            // Get the current component's name
                            let component_name = if let Some(comp_idx) = app.state.ui.component_list_state.selected() {
                                app.state.project.components[comp_idx].name.clone()
                            } else {
                                return;
                            };
                            
                            for (index, (name, ftype, value, plus_tol, minus_tol, distribution)) 
                                in features_display.iter().enumerate() 
                            {
                                let is_selected = app.state.ui.feature_list_state.selected() == Some(index);
                                
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    
                                    let feature_text = format!(
                                        "{} ({:?}) - {:.3} [{:+.3}/{:+.3}] {:?}", 
                                        name, 
                                        ftype,
                                        value,
                                        plus_tol,
                                        minus_tol,
                                        distribution.unwrap_or(DistributionType::Normal)
                                    );
                                    
                                    let response = ui.selectable_label(is_selected, feature_text);
                    
                                    if response.clicked() {
                                        app.state.ui.feature_list_state.select(Some(index));
                                        
                                        // Set filter when feature is selected
                                        app.state.mates.filter = Some(MateFilter::Feature(
                                            component_name.clone(),
                                            name.clone()
                                        ));
                                    }

                                    response.context_menu(|ui| {
                                        // Existing menu items remain the same
                                        if ui.button("✏ Edit").clicked() {
                                                if let Some(feature) = &app.state.project.components[selected_idx].features.get(index) {
                                                    *dialog_state = DialogState::FeatureEdit(FeatureEditData {
                                                        name: feature.name.clone(),
                                                        feature_type: feature.feature_type,
                                                        value: feature.dimension.value.to_string(),
                                                        plus_tolerance: feature.dimension.plus_tolerance.to_string(),
                                                        minus_tolerance: feature.dimension.minus_tolerance.to_string(),
                                                        distribution: feature.distribution.unwrap_or_default(),
                                                        is_editing: true,
                                                        feature_index: Some(index),
                                                        component_index: Some(selected_idx),
                                                    });
                                                }
                                            ui.close_menu();
                                        }

                                        if ui.button("🔍 Show Feature Mates").clicked() {
                                            switch_to_mates = true;
                                            new_filter = Some(MateFilter::Feature(
                                                component_name.clone(),
                                                name.clone()
                                            ));
                                            ui.close_menu();
                                        }
                                    
                                        // Add option to show all mates for the parent component
                                        if ui.button("🔍 Show Component Mates").clicked() {
                                            app.state.mates.filter = Some(MateFilter::Component(
                                                component_name.clone()
                                            ));
                                            app.switch_tab(ScreenMode::Mates);
                                            ui.close_menu();
                                        }
                                        
                                        ui.separator();
                                        
                                        if ui.button(egui::RichText::new("🗑 Delete").color(egui::Color32::RED)).clicked() {
                                            delete_index = Some(index);
                                            ui.close_menu();
                                        }
                                    });

                                    // Add related mates display when selected
                                    if is_selected {
                                        let related_mates = app.state.mates.get_related_mates(&component_name, name);
                                        if !related_mates.is_empty() {
                                            ui.add_space(4.0);
                                            ui.label("Related Mates:");
                                            for mate in related_mates {
                                                let other_component = if mate.component_a == component_name {
                                                    &mate.component_b
                                                } else {
                                                    &mate.component_a
                                                };
                                                let other_feature = if mate.component_a == component_name {
                                                    &mate.feature_b
                                                } else {
                                                    &mate.feature_a
                                                };
                                                
                                                let feature_a = find_feature(app, &mate.component_a, &mate.feature_a);
                                                let feature_b = find_feature(app, &mate.component_b, &mate.feature_b);
                                                let validation_status = if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                                                    if mate.validate(feat_a, feat_b).is_valid {
                                                        "Valid"
                                                    } else {
                                                        "Invalid"
                                                    }
                                                } else {
                                                    "Unknown"
                                                };
                                                
                                                ui.label(format!(
                                                    "• {} with {}.{} ({})",
                                                    mate.fit_type,
                                                    other_component,
                                                    other_feature,
                                                    validation_status
                                                ));
                                            }
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                            }

                            if let Some(filter) = new_filter {
                                app.state.mates.filter = Some(filter);
                            }
                            if switch_to_mates {
                                app.switch_tab(ScreenMode::Mates);
                            }

                            if let Some(index) = delete_index {
                                if let Some(component) = app.state.project.components.get_mut(selected_idx) {
                                    component.features.remove(index);
                                    
                                    if let Some(feat_idx) = app.state.ui.feature_list_state.selected() {
                                        if feat_idx >= component.features.len() {
                                            app.state.ui.feature_list_state
                                                .select(Some(component.features.len().saturating_sub(1)));
                                        }
                                    }

                                    if let Err(e) = app.state.file_manager.save_project(
                                        &app.state.project.project_file,
                                        &app.state.project.components,
                                        &app.state.analysis.analyses,
                                    ) {
                                        println!("Error saving project after feature delete: {}", e);
                                    }
                                }
                            }
                        });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a component to view details");
                    });
                }
            });
        });
}



pub fn show_feature_edit_dialog(
    ctx: &egui::Context,
    dialog_state: &mut DialogState,
    app: &mut App,
) {
    let mut should_close = false;
    let mut save_changes = false;

    if let DialogState::FeatureEdit(data) = dialog_state {
        let title = if data.is_editing { "Edit Feature" } else { "New Feature" };
        
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .fixed_size([320.0, 280.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let name_valid = !data.name.trim().is_empty();
                    let value_valid = data.value.parse::<f64>().is_ok();
                    let plus_tol_valid = data.plus_tolerance.parse::<f64>().is_ok();
                    let minus_tol_valid = data.minus_tolerance.parse::<f64>().is_ok();

                    // Name field
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut data.name)
                                .hint_text("Enter feature name")
                        );
                        if !name_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    // Type selection
                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        ui.radio_value(&mut data.feature_type, FeatureType::External, "External");
                        ui.radio_value(&mut data.feature_type, FeatureType::Internal, "Internal");
                    });

                    // Value and tolerances
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut data.value)
                                .hint_text("0.000")
                        );
                        if !value_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("+ Tolerance:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut data.plus_tolerance)
                                .hint_text("0.000")
                        );
                        if !plus_tol_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("- Tolerance:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut data.minus_tolerance)
                                .hint_text("0.000")
                        );
                        if !minus_tol_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    // Distribution type
                    ui.horizontal(|ui| {
                        ui.label("Distribution:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", data.distribution))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut data.distribution, DistributionType::Normal, "Normal");
                                ui.selectable_value(&mut data.distribution, DistributionType::Uniform, "Uniform");
                                ui.selectable_value(&mut data.distribution, DistributionType::Triangular, "Triangular");
                                ui.selectable_value(&mut data.distribution, DistributionType::LogNormal, "LogNormal");
                            });
                    });

                    ui.add_space(8.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_save = name_valid && value_valid && plus_tol_valid && minus_tol_valid;
                        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                            save_changes = true;
                            should_close = true;
                        }

                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });

                    // Validation message
                    if !name_valid || !value_valid || !plus_tol_valid || !minus_tol_valid {
                        ui.colored_label(egui::Color32::RED, "All fields must be valid numbers");
                    }
                });
            });

        // Apply changes after the UI is done
        if save_changes {
            if let (Ok(value), Ok(plus_tol), Ok(minus_tol)) = (
                data.value.parse::<f64>(),
                data.plus_tolerance.parse::<f64>(),
                data.minus_tolerance.parse::<f64>(),
            ) {
                let new_feature = Feature {
                    name: data.name.clone(),
                    feature_type: data.feature_type,
                    dimension: crate::config::Dimension {
                        value,
                        plus_tolerance: plus_tol,
                        minus_tolerance: minus_tol,
                    },
                    distribution: Some(data.distribution),
                    distribution_params: None, // Will be calculated automatically
                };

                if data.is_editing {
                    if let (Some(comp_idx), Some(feat_idx)) = (data.component_index, data.feature_index) {
                        if let Some(component) = app.state.project.components.get_mut(comp_idx) {
                            if let Some(feature) = component.features.get_mut(feat_idx) {
                                *feature = new_feature;
                            }
                        }
                    }
                } else if let Some(comp_idx) = data.component_index {
                    if let Some(component) = app.state.project.components.get_mut(comp_idx) {
                        component.features.push(new_feature);
                    }
                }

                // Save project
                if let Err(e) = app.state.file_manager.save_project(
                    &app.state.project.project_file,
                    &app.state.project.components,
                    &app.state.analysis.analyses,
                ) {
                    println!("Error saving project after feature update: {}", e);
                }
            }
        }
    }

    if should_close {
        *dialog_state = DialogState::None;
    }
}