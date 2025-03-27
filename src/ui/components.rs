// src/ui/components.rs
use eframe::egui;
use crate::state::{AppState, DialogState, Screen};
use crate::analysis::stackup::DistributionType;

pub fn show_components_view(ui: &mut egui::Ui, state: &mut AppState) {
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
                    state.current_dialog = DialogState::NewComponent { 
                        name: String::new(),
                        revision: "A".to_string(),
                        description: String::new(),
                    };
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_source("components_list_scroll")
                    .show(ui, |ui| {
                        let components = state.components.clone(); // Clone components to avoid borrow issues
                        for (index, component) in components.iter().enumerate() {
                            let is_selected = state.selected_component == Some(index);
                            
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                
                                let response = ui.selectable_label(
                                    is_selected,
                                    format!("{} ({} features)", component.name, component.features.len())
                                );

                                if response.clicked() {
                                    state.selected_component = Some(index);
                                    state.selected_feature = None;
                                }

                                response.context_menu(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        let (name, revision) = if let Some(rev_idx) = component.name.rfind(" Rev ") {
                                            let (name, rev) = component.name.split_at(rev_idx);
                                            (name.to_string(), rev.replace(" Rev ", ""))
                                        } else {
                                            (component.name.clone(), "A".to_string())
                                        };

                                        state.current_dialog = DialogState::EditComponent {
                                            index,
                                            name,
                                            revision,
                                            description: component.description.clone().unwrap_or_default(),
                                        };
                                        ui.close_menu();
                                    }

                                    if ui.button("🔍 Show All Mates").clicked() {
                                        state.mate_state.filter = Some(crate::state::mate_state::MateFilter::Component(
                                            component.name.clone()
                                        ));
                                        state.current_screen = Screen::Mates;
                                        ui.close_menu();
                                    }

                                    ui.separator();

                                    let delete_clicked = ui.button(
                                        egui::RichText::new("🗑 Delete").color(egui::Color32::RED)
                                    ).clicked();
                                    
                                    if delete_clicked {
                                        let state_ptr = state as *mut AppState;
                                        unsafe {
                                            (*state_ptr).components.remove(index);
                                            if (*state_ptr).components.is_empty() {
                                                (*state_ptr).selected_component = None;
                                            } else if index >= (*state_ptr).components.len() {
                                                (*state_ptr).selected_component = Some((*state_ptr).components.len() - 1);
                                            }
                                            if let Err(e) = (*state_ptr).save_project() {
                                                (*state_ptr).error_message = Some(e.to_string());
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                });
                            });
                            ui.add_space(4.0);
                        }
                    });
            });

            // Right panel - Component Details & Features
            ui.vertical(|ui| {
                ui.set_min_width(available_size.x * 0.7);
                ui.set_min_height(available_size.y);

                if let Some(selected_idx) = state.selected_component {
                    if let Some(component) = state.components.get(selected_idx) {
                        let component = component.clone(); // Clone to avoid borrow issues
                        ui.heading(&component.name);
                        if let Some(desc) = &component.description {
                            ui.label(desc);
                        }
                        ui.add_space(16.0);

                        ui.heading("Features");
                        ui.add_space(4.0);
                        
                        if ui.button("➕ Add Feature").clicked() {
                            state.current_dialog = DialogState::NewFeature {
                                component_index: selected_idx,
                                name: String::new(),
                                value: 0.0,
                                plus_tolerance: 0.0,
                                minus_tolerance: 0.0,
                            };
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .id_source("features_list_scroll")
                            .show(ui, |ui| {
                                for (index, feature) in component.features.iter().enumerate() {
                                    let is_selected = state.selected_feature == Some(index);
                                    
                                    ui.group(|ui| {
                                        ui.set_width(ui.available_width());
                                        
                                        let feature_text = format!(
                                            "{} ({:?})\n{:.3} [{:+.3}/{:+.3}] {:?}", 
                                            feature.name, 
                                            feature.feature_type,
                                            feature.dimension.value,
                                            feature.dimension.plus_tolerance,
                                            feature.dimension.minus_tolerance,
                                            feature.distribution.unwrap_or(DistributionType::Normal)
                                        );
                                        
                                        let response = ui.selectable_label(is_selected, feature_text);
                    
                                        if response.clicked() {
                                            state.selected_feature = Some(index);
                                        }

                                        response.context_menu(|ui| {
                                            if ui.button("✏ Edit").clicked() {
                                                state.current_dialog = DialogState::EditFeature {
                                                    component_index: selected_idx,
                                                    feature_index: index,
                                                    name: feature.name.clone(),
                                                    value: feature.dimension.value,
                                                    plus_tolerance: feature.dimension.plus_tolerance,
                                                    minus_tolerance: feature.dimension.minus_tolerance,
                                                };
                                                ui.close_menu();
                                            }

                                            if ui.button("🔍 Show Feature Mates").clicked() {
                                                state.mate_state.filter = Some(crate::state::mate_state::MateFilter::Feature(
                                                    component.name.clone(), 
                                                    feature.name.clone()
                                                ));
                                                state.current_screen = Screen::Mates;
                                                ui.close_menu();
                                            }
                                        
                                            ui.separator();
                                            
                                            let delete_clicked = ui.button(
                                                egui::RichText::new("🗑 Delete").color(egui::Color32::RED)
                                            ).clicked();

                                            if delete_clicked {
                                                let state_ptr = state as *mut AppState;
                                                unsafe {
                                                    if let Some(component) = (*state_ptr).components.get_mut(selected_idx) {
                                                        component.features.remove(index);
                                                        
                                                        if component.features.is_empty() {
                                                            (*state_ptr).selected_feature = None;
                                                        } else if index >= component.features.len() {
                                                            (*state_ptr).selected_feature = Some(component.features.len() - 1);
                                                        }

                                                        if let Err(e) = (*state_ptr).save_project() {
                                                            (*state_ptr).error_message = Some(e.to_string());
                                                        }
                                                    }
                                                }
                                                ui.close_menu();
                                            }
                                        });

                                        // Show related mates if selected
                                        if is_selected {
                                            let related_mates = state.mates.iter()
                                                .filter(|m| {
                                                    (m.component_a == component.name && m.feature_a == feature.name) ||
                                                    (m.component_b == component.name && m.feature_b == feature.name)
                                                });

                                            ui.add_space(4.0);
                                            ui.label("Related Mates:");
                                            for mate in related_mates {
                                                let other_component = if mate.component_a == component.name {
                                                    &mate.component_b
                                                } else {
                                                    &mate.component_a
                                                };
                                                let other_feature = if mate.component_a == component.name {
                                                    &mate.feature_b
                                                } else {
                                                    &mate.feature_a
                                                };

                                                ui.label(format!(
                                                    "• {} with {}.{}",
                                                    mate.fit_type,
                                                    other_component,
                                                    other_feature
                                                ));
                                            }
                                        }
                                    });
                                    ui.add_space(4.0);
                                }
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