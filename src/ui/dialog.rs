// src/ui/dialog.rs
use eframe::egui;
use crate::state::{AppState, DialogState};
use crate::config::{Feature, FeatureType};
use crate::config::mate::FitType;
use crate::analysis::stackup::{DistributionType, AnalysisMethod, 
    StackupAnalysis, MonteCarloSettings, StackupContribution};
use uuid::Uuid;
use crate::utils::find_feature;

pub fn show_dialog(ctx: &egui::Context, state: &mut AppState) {
    match &mut state.current_dialog {
        DialogState::None => {},
        
        DialogState::NewComponent { 
            name, revision, description 
        } => {
            show_component_dialog(ctx, state, None, name, revision, description);
        },
        
        DialogState::EditComponent { 
            index, name, revision, description 
        } => {
            show_component_dialog(ctx, state, Some(*index), name, revision, description);
        },
        
        DialogState::NewFeature { 
            component_index, name, value, 
            plus_tolerance, minus_tolerance 
        } => {
            show_feature_dialog(
                ctx, state, *component_index, None, 
                name, value, plus_tolerance, minus_tolerance
            );
        },
        
        DialogState::EditFeature { 
            component_index, feature_index, name, value, 
            plus_tolerance, minus_tolerance 
        } => {
            show_feature_dialog(
                ctx, state, *component_index, Some(*feature_index), 
                name, value, plus_tolerance, minus_tolerance
            );
        },
        
        DialogState::NewMate { 
            component_a, feature_a, 
            component_b, feature_b 
        } => {
            show_mate_dialog(
                ctx, state, None, 
                component_a, feature_a, component_b, feature_b
            );
        },
        
        DialogState::EditMate { 
            index, component_a, feature_a, 
            component_b, feature_b 
        } => {
            show_mate_dialog(
                ctx, state, Some(*index), 
                component_a, feature_a, component_b, feature_b
            );
        },
        
        DialogState::NewAnalysis { 
            name, methods, monte_carlo_settings 
        } => {
            show_analysis_dialog(
                ctx, state, None, 
                name, methods, monte_carlo_settings
            );
        },
        
        DialogState::EditAnalysis { 
            index, name, methods, monte_carlo_settings 
        } => {
            show_analysis_dialog(
                ctx, state, Some(*index), 
                name, methods, monte_carlo_settings
            );
        },
        
        DialogState::NewContribution { 
            analysis_index, component_id, feature_id, 
            direction, half_count 
        } => {
            show_contribution_dialog(
                ctx, state, *analysis_index, None,
                component_id, feature_id, direction, half_count
            );
        },
        
        DialogState::EditContribution { 
            analysis_index, contribution_index, component_id, 
            feature_id, direction, half_count 
        } => {
            show_contribution_dialog(
                ctx, state, *analysis_index, *contribution_index,
                component_id, feature_id, direction, half_count
            );
        },
    }
}

fn show_component_dialog(
    ctx: &egui::Context,
    state: &mut AppState,
    edit_index: Option<usize>,
    name: &mut String,
    revision: &mut String,
    description: &mut String,
) {
    let title = if edit_index.is_some() { "Edit Component" } else { "New Component" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([300.0, 200.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let name_valid = !name.trim().is_empty();
                let revision_valid = !revision.trim().is_empty();

                // Name field
                ui.horizontal(|ui| {
                    ui.label("Name:").on_hover_text("Component name");
                    let response = ui.add(
                        egui::TextEdit::singleline(name)
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
                        egui::TextEdit::singleline(revision)
                            .desired_width(200.0)
                            .hint_text("Enter revision")
                    );
                    if !revision_valid && response.lost_focus() {
                        ui.colored_label(egui::Color32::RED, "⚠");
                    }
                });

                // Description field
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.add(
                        egui::TextEdit::multiline(description)
                            .desired_width(200.0)
                            .desired_rows(3)
                            .hint_text("Enter component description")
                    );
                });

                ui.add_space(8.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        state.current_dialog = DialogState::None;
                    }

                    let can_save = name_valid && revision_valid;
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        let full_name = format!("{} Rev {}", name.trim(), revision.trim());
                        let new_component = crate::config::Component {
                            name: full_name,
                            description: Some(description.trim().to_string()),
                            features: if let Some(idx) = edit_index {
                                state.components[idx].features.clone()
                            } else {
                                Vec::new()
                            },
                        };

                        if let Some(idx) = edit_index {
                            state.components[idx] = new_component;
                        } else {
                            state.components.push(new_component);
                        }

                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                        state.current_dialog = DialogState::None;
                    }
                });

                // Validation message
                if !name_valid || !revision_valid {
                    ui.colored_label(
                        egui::Color32::RED,
                        "Name and revision are required"
                    );
                }
            });
        });
}

fn show_feature_dialog(
    ctx: &egui::Context,
    state: &mut AppState,
    component_index: usize,
    feature_index: Option<usize>,
    name: &mut String,
    value: &mut f64,
    plus_tolerance: &mut f64,
    minus_tolerance: &mut f64,
) {
    let title = if feature_index.is_some() { "Edit Feature" } else { "New Feature" };
    let mut feature_type = FeatureType::External;
    let mut distribution = DistributionType::Normal;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([320.0, 280.0])
        .show(ctx, |ui| {
            let name_valid = !name.trim().is_empty();

            ui.horizontal(|ui| {
                ui.label("Name:");
                let response = ui.text_edit_singleline(name);
                if !name_valid && response.lost_focus() {
                    ui.colored_label(egui::Color32::RED, "⚠");
                }
            });

            ui.horizontal(|ui| {
                ui.label("Type:");
                ui.radio_value(&mut feature_type, FeatureType::External, "External");
                ui.radio_value(&mut feature_type, FeatureType::Internal, "Internal");
            });

            ui.horizontal(|ui| {
                ui.label("Value:");
                ui.add(egui::DragValue::new(value).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("+ Tolerance:");
                ui.add(egui::DragValue::new(plus_tolerance).speed(0.01));
            });

            ui.horizontal(|ui| {
                ui.label("- Tolerance:");
                ui.add(egui::DragValue::new(minus_tolerance).speed(0.01));
            });

            ui.horizontal(|ui| {
                ui.label("Distribution:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", distribution))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut distribution, DistributionType::Normal, "Normal");
                        ui.selectable_value(&mut distribution, DistributionType::Uniform, "Uniform");
                        ui.selectable_value(&mut distribution, DistributionType::Triangular, "Triangular");
                        ui.selectable_value(&mut distribution, DistributionType::LogNormal, "LogNormal");
                    });
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.current_dialog = DialogState::None;
                }

                let can_save = name_valid;
                if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                    let new_feature = Feature {
                        name: name.clone(),
                        feature_type,
                        dimension: crate::config::Dimension {
                            value: *value,
                            plus_tolerance: *plus_tolerance,
                            minus_tolerance: *minus_tolerance,
                        },
                        distribution: Some(distribution),
                        distribution_params: None,
                    };

                    if let Some(idx) = feature_index {
                        state.components[component_index].features[idx] = new_feature;
                    } else {
                        state.components[component_index].features.push(new_feature);
                    }

                    if let Err(e) = state.save_project() {
                        state.error_message = Some(e.to_string());
                    }
                    state.current_dialog = DialogState::None;
                }
            });

            if !name_valid {
                ui.colored_label(egui::Color32::RED, "Name is required");
            }
        });
}

fn show_mate_dialog(
    ctx: &egui::Context,
    state: &mut AppState,
    edit_index: Option<usize>,
    component_a: &mut String,
    feature_a: &mut String,
    component_b: &mut String,
    feature_b: &mut String,
) {
    let title = if edit_index.is_some() { "Edit Mate" } else { "New Mate" };
    let mut fit_type = FitType::Clearance;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 400.0])
        .show(ctx, |ui| {
            // Component A selection
            ui.group(|ui| {
                ui.heading("Component A");
                egui::ComboBox::from_label("Select Component")
                    .selected_text(&*component_a)
                    .show_ui(ui, |ui| {
                        for component in &state.components {
                            ui.selectable_value(
                                component_a,
                                component.name.clone(),
                                &component.name
                            );
                        }
                    });

                if let Some(component) = state.components.iter().find(|c| c.name == *component_a) {
                    egui::ComboBox::from_label("Select Feature")
                        .selected_text(&*feature_a)
                        .show_ui(ui, |ui| {
                            for feature in &component.features {
                                ui.selectable_value(
                                    feature_a,
                                    feature.name.clone(),
                                    &feature.name
                                );
                            }
                        });
                }
            });

            ui.add_space(8.0);

            // Component B selection
            ui.group(|ui| {
                ui.heading("Component B");
                egui::ComboBox::from_label("Select Component")
                    .selected_text(&*component_b)
                    .show_ui(ui, |ui| {
                        for component in &state.components {
                            ui.selectable_value(
                                component_b,
                                component.name.clone(),
                                &component.name
                            );
                        }
                    });

                if let Some(component) = state.components.iter().find(|c| c.name == *component_b) {
                    egui::ComboBox::from_label("Select Feature")
                        .selected_text(&*feature_b)
                        .show_ui(ui, |ui| {
                            for feature in &component.features {
                                ui.selectable_value(
                                    feature_b,
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
                    ui.radio_value(&mut fit_type, FitType::Clearance, "Clearance");
                    ui.radio_value(&mut fit_type, FitType::Transition, "Transition");
                    ui.radio_value(&mut fit_type, FitType::Interference, "Interference");
                });
            });

            ui.add_space(16.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.current_dialog = DialogState::None;
                }

                let can_save = !component_a.is_empty() && !feature_a.is_empty() 
                    && !component_b.is_empty() && !feature_b.is_empty();

                if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                    let new_mate = crate::config::mate::Mate {
                        id: Uuid::new_v4().to_string(),
                        component_a: component_a.clone(),
                        feature_a: feature_a.clone(),
                        component_b: component_b.clone(),
                        feature_b: feature_b.clone(),
                        fit_type,
                    };

                    if let Some(idx) = edit_index {
                        state.mates[idx] = new_mate;
                    } else {
                        state.mates.push(new_mate);
                    }

                    state.update_mate_graph();

                    if let Err(e) = state.save_project() {
                        state.error_message = Some(e.to_string());
                    }
                    state.current_dialog = DialogState::None;
                }
            });
        });
}

fn show_analysis_dialog(
    ctx: &egui::Context,
    state: &mut AppState,
    edit_index: Option<usize>,
    name: &mut String,
    methods: &mut Vec<AnalysisMethod>,
    monte_carlo_settings: &mut MonteCarloSettings,
) {
    let title = if edit_index.is_some() { "Edit Analysis" } else { "New Analysis" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 500.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Name input
                ui.group(|ui| {
                    ui.heading("Analysis Name");
                    ui.text_edit_singleline(name);
                });

                ui.add_space(8.0);

                // Methods selection
                ui.group(|ui| {
                    ui.heading("Analysis Methods");
                    
                    let all_methods = [
                        AnalysisMethod::WorstCase,
                        AnalysisMethod::Rss,
                        AnalysisMethod::MonteCarlo
                    ];

                    for method in &all_methods {
                        let mut enabled = methods.contains(method);
                        if ui.checkbox(&mut enabled, format!("{:?}", method)).changed() {
                            if enabled {
                                methods.push(*method);
                            } else {
                                methods.retain(|m| m != method);
                            }
                        }
                    }
                });

                // Monte Carlo settings if enabled
                if methods.contains(&AnalysisMethod::MonteCarlo) {
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.heading("Monte Carlo Settings");
                        
                        ui.horizontal(|ui| {
                            ui.label("Iterations:");
                            ui.add(egui::DragValue::new(&mut monte_carlo_settings.iterations)
                                .speed(1000)
                                .clamp_range(1000..=1000000));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Confidence (%):");
                            let mut conf_pct = monte_carlo_settings.confidence * 100.0;
                            if ui.add(egui::DragValue::new(&mut conf_pct)
                                .speed(0.1)
                                .clamp_range(90.0..=99.99)).changed() {
                                monte_carlo_settings.confidence = conf_pct / 100.0;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Random Seed:");
                            let mut has_seed = monte_carlo_settings.seed.is_some();
                            if ui.checkbox(&mut has_seed, "Use seed").changed() {
                                monte_carlo_settings.seed = if has_seed { Some(0) } else { None };
                            }
                            if let Some(ref mut seed) = monte_carlo_settings.seed {
                                ui.add(egui::DragValue::new(seed).speed(1));
                            }
                        });
                    });
                }

                // Action buttons
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        state.current_dialog = DialogState::None;
                    }

                    let can_save = !name.trim().is_empty() && !methods.is_empty();
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        let new_analysis = StackupAnalysis {
                            id: if let Some(idx) = edit_index {
                                state.analyses[idx].id.clone()
                            } else {
                                Uuid::new_v4().to_string()
                            },
                            name: name.clone(),
                            contributions: if let Some(idx) = edit_index {
                                state.analyses[idx].contributions.clone()
                            } else {
                                Vec::new()
                            },
                            methods: methods.clone(),
                            monte_carlo_settings: if methods.contains(&AnalysisMethod::MonteCarlo) {
                                Some(monte_carlo_settings.clone())
                            } else {
                                None
                            },
                        };

                        if let Some(idx) = edit_index {
                            state.analyses[idx] = new_analysis;
                        } else {
                            state.analyses.push(new_analysis);
                        }

                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                        state.current_dialog = DialogState::None;
                    }
                });
            });
        });
}

fn show_contribution_dialog(
    ctx: &egui::Context,
    state: &mut AppState,
    analysis_index: usize,
    contribution_index: Option<usize>,
    component_id: &mut String,
    feature_id: &mut String,
    direction: &mut f64,
    half_count: &mut bool,
) {
    let title = if contribution_index.is_some() { "Edit Contribution" } else { "Add Contribution" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Component selection
                ui.group(|ui| {
                    ui.heading("Component");
                    egui::ComboBox::from_label("Select Component")
                        .selected_text(&*component_id)
                        .show_ui(ui, |ui| {
                            for component in &state.components {
                                ui.selectable_value(
                                    component_id,
                                    component.name.clone(),
                                    &component.name
                                );
                            }
                        });

                    if let Some(component) = state.components.iter().find(|c| c.name == *component_id) {
                        egui::ComboBox::from_label("Select Feature")
                            .selected_text(&*feature_id)
                            .show_ui(ui, |ui| {
                                for feature in &component.features {
                                    ui.selectable_value(
                                        feature_id,
                                        feature.name.clone(),
                                        &feature.name
                                    );
                                }
                            });

                        // Show feature details if selected
                        if let Some(feature) = component.features.iter().find(|f| f.name == *feature_id) {
                            ui.add_space(4.0);
                            ui.label(format!(
                                "Value: {:.3} [{:+.3}/{:+.3}]",
                                feature.dimension.value,
                                feature.dimension.plus_tolerance,
                                feature.dimension.minus_tolerance
                            ));
                        }
                    }
                });

                ui.add_space(8.0);

                // Direction and half count
                ui.group(|ui| {
                    ui.heading("Properties");
                    
                    ui.horizontal(|ui| {
                        ui.label("Direction:");
                        if ui.radio_value(direction, 1.0, "Positive").clicked() ||
                           ui.radio_value(direction, -1.0, "Negative").clicked() {
                            // Direction updated via radio buttons
                        }
                    });

                    ui.checkbox(half_count, "Half Count");
                });

                // Action buttons
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        state.current_dialog = DialogState::None;
                    }

                    let can_save = !component_id.is_empty() && !feature_id.is_empty();
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        if let Some(analysis) = state.analyses.get_mut(analysis_index) {
                            if let Some(feature) = find_feature(&state.components, component_id, feature_id) {
                                let contribution = StackupContribution {
                                    component_id: component_id.clone(),
                                    feature_id: feature_id.clone(),
                                    direction: *direction,
                                    half_count: *half_count,
                                    distribution: Some(StackupAnalysis::calculate_distribution_params(feature)),
                                };

                                if let Some(idx) = contribution_index {
                                    analysis.contributions[idx] = contribution;
                                } else {
                                    analysis.contributions.push(contribution);
                                }

                                if let Err(e) = state.save_project() {
                                    state.error_message = Some(e.to_string());
                                }
                            }
                        }
                        state.current_dialog = DialogState::None;
                    }
                });
            });
        });
}