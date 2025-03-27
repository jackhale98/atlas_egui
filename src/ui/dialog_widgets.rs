// src/ui/dialog_widgets.rs

use eframe::egui;
use uuid::Uuid;
use crate::config::{Component, Feature, FeatureType};
use crate::config::mate::{Mate, FitType};
use crate::analysis::stackup::{
    AnalysisMethod, DistributionType, MonteCarloSettings,
    StackupAnalysis, StackupContribution
};
use crate::utils::find_feature;

#[derive(Default)]
pub struct ComponentDialog {
    name: String,
    revision: String,
    description: String,
    open: bool,
}

impl ComponentDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        edit_index: Option<usize>,
        components: &mut Vec<Component>,
        on_close: impl FnOnce(),
    ) -> Option<bool> {
        let mut changed = false;
        
        if self.open {
            let mut should_close = false;
            
            let result = egui::Window::new(if edit_index.is_some() { "Edit Component" } else { "New Component" })
                .collapsible(false)
                .resizable(false)
                .fixed_size([300.0, 200.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        let name_valid = !self.name.trim().is_empty();
                        let revision_valid = !self.revision.trim().is_empty();

                        // Name field
                        ui.horizontal(|ui| {
                            ui.label("Name:").on_hover_text("Component name");
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.name)
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
                                egui::TextEdit::singleline(&mut self.revision)
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
                                egui::TextEdit::multiline(&mut self.description)
                                    .desired_width(200.0)
                                    .desired_rows(3)
                                    .hint_text("Enter component description")
                            );
                        });

                        ui.add_space(8.0);

                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }

                            let can_save = name_valid && revision_valid;
                            if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                                let full_name = format!("{} Rev {}", self.name.trim(), self.revision.trim());
                                let new_component = Component {
                                    name: full_name,
                                    description: Some(self.description.trim().to_string()),
                                    features: if let Some(idx) = edit_index {
                                        components[idx].features.clone()
                                    } else {
                                        Vec::new()
                                    },
                                };

                                if let Some(idx) = edit_index {
                                    components[idx] = new_component;
                                } else {
                                    components.push(new_component);
                                }
                                
                                changed = true;
                                should_close = true;
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

            if should_close {
                self.open = false;
                on_close();
            }

            result.map(|_| changed)
        } else {
            None
        }
    }

    pub fn open(&mut self, component: Option<&Component>) {
        self.open = true;
        
        if let Some(component) = component {
            if let Some(rev_idx) = component.name.rfind(" Rev ") {
                let (name, rev) = component.name.split_at(rev_idx);
                self.name = name.to_string();
                self.revision = rev.replace(" Rev ", "");
            } else {
                self.name = component.name.clone();
                self.revision = "A".to_string();
            }
            self.description = component.description.clone().unwrap_or_default();
        } else {
            self.name.clear();
            self.revision = "A".to_string();
            self.description.clear();
        }
    }
}

#[derive(Default)]
pub struct FeatureDialog {
    name: String,
    value: String,
    plus_tolerance: String,
    minus_tolerance: String,
    feature_type: FeatureType,
    distribution: DistributionType,
    open: bool,
}

impl FeatureDialog {
    pub fn new() -> Self {
        Self {
            feature_type: FeatureType::External,
            distribution: DistributionType::Normal,
            ..Default::default()
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        component_index: usize,
        feature_index: Option<usize>,
        components: &mut Vec<Component>,
        on_close: impl FnOnce(),
    ) -> Option<bool> {
        let mut changed = false;

        if self.open {
            let mut should_close = false;

            let result = egui::Window::new(if feature_index.is_some() { "Edit Feature" } else { "New Feature" })
                .collapsible(false)
                .resizable(false)
                .fixed_size([320.0, 280.0])
                .show(ctx, |ui| {
                    let name_valid = !self.name.trim().is_empty();
                    let value_valid = self.value.parse::<f64>().is_ok();
                    let plus_tol_valid = self.plus_tolerance.parse::<f64>().is_ok();
                    let minus_tol_valid = self.minus_tolerance.parse::<f64>().is_ok();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        let response = ui.text_edit_singleline(&mut self.name);
                        if !name_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        ui.radio_value(&mut self.feature_type, FeatureType::External, "External");
                        ui.radio_value(&mut self.feature_type, FeatureType::Internal, "Internal");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let response = ui.text_edit_singleline(&mut self.value);
                        if !value_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("+ Tolerance:");
                        let response = ui.text_edit_singleline(&mut self.plus_tolerance);
                        if !plus_tol_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("- Tolerance:");
                        let response = ui.text_edit_singleline(&mut self.minus_tolerance);
                        if !minus_tol_valid && response.lost_focus() {
                            ui.colored_label(egui::Color32::RED, "⚠");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Distribution:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.distribution))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.distribution, DistributionType::Normal, "Normal");
                                ui.selectable_value(&mut self.distribution, DistributionType::Uniform, "Uniform");
                                ui.selectable_value(&mut self.distribution, DistributionType::Triangular, "Triangular");
                                ui.selectable_value(&mut self.distribution, DistributionType::LogNormal, "LogNormal");
                            });
                    });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }

                        let can_save = name_valid && value_valid && plus_tol_valid && minus_tol_valid;
                        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                            if let (Ok(val), Ok(plus), Ok(minus)) = (
                                self.value.parse::<f64>(),
                                self.plus_tolerance.parse::<f64>(),
                                self.minus_tolerance.parse::<f64>(),
                            ) {
                                let new_feature = Feature {
                                    name: self.name.clone(),
                                    feature_type: self.feature_type,
                                    dimension: crate::config::Dimension {
                                        value: val,
                                        plus_tolerance: plus,
                                        minus_tolerance: minus,
                                    },
                                    distribution: Some(self.distribution),
                                    distribution_params: None,
                                };

                                if let Some(idx) = feature_index {
                                    components[component_index].features[idx] = new_feature;
                                } else {
                                    components[component_index].features.push(new_feature);
                                }

                                changed = true;
                                should_close = true;
                            }
                        }
                    });

                    if !name_valid || !value_valid || !plus_tol_valid || !minus_tol_valid {
                        ui.colored_label(egui::Color32::RED, "All fields must be valid numbers");
                    }
                });

            if should_close {
                self.open = false;
                on_close();
            }

            result.map(|_| changed)
        } else {
            None
        }
    }

    pub fn open(&mut self, feature: Option<&Feature>) {
        self.open = true;
        
        if let Some(feature) = feature {
            self.name = feature.name.clone();
            self.value = feature.dimension.value.to_string();
            self.plus_tolerance = feature.dimension.plus_tolerance.to_string();
            self.minus_tolerance = feature.dimension.minus_tolerance.to_string();
            self.feature_type = feature.feature_type;
            self.distribution = feature.distribution.unwrap_or(DistributionType::Normal);
        } else {
            self.name.clear();
            self.value = "0.0".to_string();
            self.plus_tolerance = "0.0".to_string();
            self.minus_tolerance = "0.0".to_string();
            self.feature_type = FeatureType::External;
            self.distribution = DistributionType::Normal;
        }
    }
}

#[derive(Default)]
pub struct MateDialog {
    component_a: String,
    feature_a: String,
    component_b: String,
    feature_b: String,
    fit_type: FitType,
    open: bool,
}

impl MateDialog {
    pub fn new() -> Self {
        Self {
            fit_type: FitType::Clearance,
            ..Default::default()
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        edit_index: Option<usize>,
        components: &[Component],
        mates: &mut Vec<Mate>,
        on_close: impl FnOnce(),
    ) -> Option<bool> {
        let mut changed = false;

        if self.open {
            let mut should_close = false;

            let result = egui::Window::new(if edit_index.is_some() { "Edit Mate" } else { "New Mate" })
                .collapsible(false)
                .resizable(false)
                .fixed_size([400.0, 400.0])
                .show(ctx, |ui| {
                    // Component A selection
                    ui.group(|ui| {
                        ui.heading("Component A");
                        ui.push_id("component_a_selection", |ui| {
                            egui::ComboBox::from_label("Select Component")
                                .selected_text(&self.component_a)
                                .show_ui(ui, |ui| {
                                    for component in components {
                                        ui.selectable_value(
                                            &mut self.component_a,
                                            component.name.clone(),
                                            &component.name
                                        );
                                    }
                                });
                        });

                        if let Some(component) = components.iter().find(|c| c.name == self.component_a) {
                            ui.push_id("feature_a_selection", |ui| {
                                egui::ComboBox::from_label("Select Feature")
                                    .selected_text(&self.feature_a)
                                    .show_ui(ui, |ui| {
                                        for feature in &component.features {
                                            ui.selectable_value(
                                                &mut self.feature_a,
                                                feature.name.clone(),
                                                &feature.name
                                            );
                                        }
                                    });
                            });
                        }
                    });

                    ui.add_space(8.0);

                    // Component B selection
                    ui.group(|ui| {
                        ui.heading("Component B");
                        egui::ComboBox::from_label("Select Component")
                            .selected_text(&self.component_b)
                            .show_ui(ui, |ui| {
                                for component in components {
                                    ui.selectable_value(
                                        &mut self.component_b,
                                        component.name.clone(),
                                        &component.name
                                    );
                                }
                            });

                            if let Some(component) = components.iter().find(|c| c.name == self.component_b) {
                                egui::ComboBox::from_label("Select Feature")
                                    .selected_text(&self.feature_b)
                                    .show_ui(ui, |ui| {
                                        for feature in &component.features {
                                            ui.selectable_value(
                                                &mut self.feature_b,
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
                                ui.radio_value(&mut self.fit_type, FitType::Clearance, "Clearance");
                                ui.radio_value(&mut self.fit_type, FitType::Transition, "Transition");
                                ui.radio_value(&mut self.fit_type, FitType::Interference, "Interference");
                            });
                        });
    
                        ui.add_space(16.0);
    
                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }
    
                            let can_save = !self.component_a.is_empty() && !self.feature_a.is_empty() 
                                && !self.component_b.is_empty() && !self.feature_b.is_empty();
    
                            if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                                let new_mate = Mate {
                                    id: Uuid::new_v4().to_string(),
                                    component_a: self.component_a.clone(),
                                    feature_a: self.feature_a.clone(),
                                    component_b: self.component_b.clone(),
                                    feature_b: self.feature_b.clone(),
                                    fit_type: self.fit_type.clone(),
                                };
    
                                if let Some(idx) = edit_index {
                                    mates[idx] = new_mate;
                                } else {
                                    mates.push(new_mate);
                                }
    
                                changed = true;
                                should_close = true;
                            }
                        });
                    });
    
                if should_close {
                    self.open = false;
                    on_close();
                }
    
                result.map(|_| changed)
            } else {
                None
            }
        }
    
        pub fn open(&mut self, mate: Option<&Mate>) {
            self.open = true;
            
            if let Some(mate) = mate {
                self.component_a = mate.component_a.clone();
                self.feature_a = mate.feature_a.clone();
                self.component_b = mate.component_b.clone();
                self.feature_b = mate.feature_b.clone();
                self.fit_type = mate.fit_type.clone();
            } else {
                self.component_a.clear();
                self.feature_a.clear();
                self.component_b.clear();
                self.feature_b.clear();
                self.fit_type = FitType::Clearance;
            }
        }
    }
    
    #[derive(Default)]
    pub struct AnalysisDialog {
        name: String,
        methods: Vec<AnalysisMethod>,
        monte_carlo_settings: MonteCarloSettings,
        upper_spec_limit_str: String,
        lower_spec_limit_str: String, 
        open: bool,
    }
    
    
    impl AnalysisDialog {
        pub fn new() -> Self {
            Self {
                methods: vec![AnalysisMethod::WorstCase],
                monte_carlo_settings: MonteCarloSettings::default(),
                ..Default::default()
            }
        }
    
        pub fn show(
            &mut self,
            ctx: &egui::Context,
            edit_index: Option<usize>,
            analyses: &mut Vec<StackupAnalysis>,
            on_close: impl FnOnce(),
        ) -> Option<bool> {
            let mut changed = false;
    
            if self.open {
                let mut should_close = false;
    
                let result = egui::Window::new(if edit_index.is_some() { "Edit Analysis" } else { "New Analysis" })
                    .collapsible(false)
                    .resizable(false)
                    .fixed_size([400.0, 500.0])
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            // Name input
                            ui.group(|ui| {
                                ui.heading("Analysis Name");
                                ui.text_edit_singleline(&mut self.name);
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
                                    let mut enabled = self.methods.contains(method);
                                    if ui.checkbox(&mut enabled, format!("{:?}", method)).changed() {
                                        if enabled {
                                            self.methods.push(*method);
                                        } else {
                                            self.methods.retain(|m| m != method);
                                        }
                                    }
                                }
                            });
    
                            // Monte Carlo settings if enabled
                            if self.methods.contains(&AnalysisMethod::MonteCarlo) {
                                ui.add_space(8.0);
                                ui.group(|ui| {
                                    ui.heading("Monte Carlo Settings");
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Iterations:");
                                        let mut iter_str = self.monte_carlo_settings.iterations.to_string();
                                        if ui.text_edit_singleline(&mut iter_str).changed() {
                                            if let Ok(value) = iter_str.parse() {
                                                self.monte_carlo_settings.iterations = value;
                                            }
                                        }
                                    });
    
                                    ui.horizontal(|ui| {
                                        ui.label("Confidence (%):");
                                        let mut conf_str = (self.monte_carlo_settings.confidence * 100.0).to_string();
                                        if ui.text_edit_singleline(&mut conf_str).changed() {
                                            if let Ok(value) = conf_str.parse::<f64>() {
                                                self.monte_carlo_settings.confidence = (value / 100.0).clamp(0.0, 0.9999);
                                            }
                                        }
                                    });
    
                                    ui.horizontal(|ui| {
                                        ui.label("Random Seed (optional):");
                                        let mut seed_str = self.monte_carlo_settings.seed
                                            .map(|s| s.to_string())
                                            .unwrap_or_default();
                                        if ui.text_edit_singleline(&mut seed_str).changed() {
                                            self.monte_carlo_settings.seed = seed_str.parse().ok();
                                        }
                                    });
                                });
                            }
                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.heading("Specification Limits");
                                
                                // Upper Spec Limit
                                ui.horizontal(|ui| {
                                    ui.label("Upper Spec:");
                                    ui.add(egui::TextEdit::singleline(&mut self.upper_spec_limit_str)
                                        .desired_width(100.0)
                                        .hint_text("Enter USL"));
                                });
                        
                                // Lower Spec Limit
                                ui.horizontal(|ui| {
                                    ui.label("Lower Spec:");
                                    ui.add(egui::TextEdit::singleline(&mut self.lower_spec_limit_str)
                                        .desired_width(100.0)
                                        .hint_text("Enter LSL"));
                                });
                            });
    
                            // Action buttons
                            ui.add_space(16.0);
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    should_close = true;
                                }
    
                                let can_save = !self.name.trim().is_empty() && !self.methods.is_empty();
                                if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                                    let new_analysis = StackupAnalysis {
                                        // For editing, preserve the original ID
                                        id: if let Some(idx) = edit_index {
                                            analyses[idx].id.clone()
                                        } else {
                                            Uuid::new_v4().to_string()
                                        },
                                        name: self.name.clone(),
                                        // Preserve existing contributions when editing
                                        contributions: if let Some(idx) = edit_index {
                                            analyses[idx].contributions.clone()
                                        } else {
                                            Vec::new()
                                        },
                                        methods: self.methods.clone(),
                                        monte_carlo_settings: if self.methods.contains(&AnalysisMethod::MonteCarlo) {
                                            Some(self.monte_carlo_settings.clone())
                                        } else {
                                            None
                                        },
                                        upper_spec_limit: if !self.upper_spec_limit_str.is_empty() {
                                            self.upper_spec_limit_str.parse().ok()
                                        } else {
                                            None
                                        },
                                        lower_spec_limit: if !self.lower_spec_limit_str.is_empty() {
                                            self.lower_spec_limit_str.parse().ok()
                                        } else {
                                            None
                                        },
                                    };
                                
                                    if let Some(idx) = edit_index {
                                        analyses[idx] = new_analysis;
                                    } else {
                                        analyses.push(new_analysis);
                                    }
                                
                                    changed = true;
                                    should_close = true;
                                }
                            });
                        });
                    });
    
                if should_close {
                    self.open = false;
                    on_close();
                }
    
                result.map(|_| changed)
            } else {
                None
            }
        }
    
        pub fn open(&mut self, analysis: Option<&StackupAnalysis>) {
            self.open = true;
            
            if let Some(analysis) = analysis {
                self.name = analysis.name.clone();
                self.methods = analysis.methods.clone();
                self.monte_carlo_settings = analysis.monte_carlo_settings.clone()
                    .unwrap_or_default();
                self.upper_spec_limit_str = analysis.upper_spec_limit
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                self.lower_spec_limit_str = analysis.lower_spec_limit
                    .map(|v| v.to_string())
                    .unwrap_or_default();
            } else {
                self.name.clear();
                self.methods = vec![AnalysisMethod::WorstCase];
                self.monte_carlo_settings = MonteCarloSettings::default();
                self.upper_spec_limit_str.clear();
                self.lower_spec_limit_str.clear();
            }
        }
    }
    
    #[derive(Default)]
    pub struct ContributionDialog {
        component_id: String,
        feature_id: String,
        direction: f64,
        half_count: bool,
        open: bool,
    }
    
    impl ContributionDialog {
        pub fn new() -> Self {
            Self {
                direction: 1.0,
                ..Default::default()
            }
        }
    
        pub fn show(
            &mut self,
            ctx: &egui::Context,
            analysis_index: usize,
            contribution_index: Option<usize>,
            components: &[Component],
            analyses: &mut Vec<StackupAnalysis>,
            on_close: impl FnOnce(),
        ) -> Option<bool> {
            let mut changed = false;
    
            if self.open {
                let mut should_close = false;
    
                let result = egui::Window::new(if contribution_index.is_some() { "Edit Contribution" } else { "Add Contribution" })
                    .collapsible(false)
                    .resizable(false)
                    .fixed_size([400.0, 300.0])
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            // Component selection
                            ui.group(|ui| {
                                ui.heading("Component");
                                egui::ComboBox::from_label("Select Component")
                                    .selected_text(&self.component_id)
                                    .show_ui(ui, |ui| {
                                        for component in components {
                                            ui.selectable_value(
                                                &mut self.component_id,
                                                component.name.clone(),
                                                &component.name
                                            );
                                        }
                                    });
    
                                if let Some(component) = components.iter().find(|c| c.name == self.component_id) {
                                    egui::ComboBox::from_label("Select Feature")
                                        .selected_text(&self.feature_id)
                                        .show_ui(ui, |ui| {
                                            for feature in &component.features {
                                                ui.selectable_value(
                                                    &mut self.feature_id,
                                                    feature.name.clone(),
                                                    &feature.name
                                                );
                                            }
                                        });
    
                                    // Show feature details if selected
                                    if let Some(feature) = component.features.iter().find(|f| f.name == self.feature_id) {
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
                                    if ui.radio_value(&mut self.direction, 1.0, "Positive").clicked() ||
                                       ui.radio_value(&mut self.direction, -1.0, "Negative").clicked() {
                                        // Direction updated via radio buttons
                                    }
                                });
    
                                ui.checkbox(&mut self.half_count, "Half Count");
                            });
    
                            // Action buttons
                            ui.add_space(16.0);
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    should_close = true;
                                }
    
                                let can_save = !self.component_id.is_empty() && !self.feature_id.is_empty();
                                if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                                    if let Some(analysis) = analyses.get_mut(analysis_index) {
                                        if let Some(feature) = find_feature(components, &self.component_id, &self.feature_id) {
                                            let contribution = StackupContribution {
                                                component_id: self.component_id.clone(),
                                                feature_id: self.feature_id.clone(),
                                                direction: self.direction,
                                                half_count: self.half_count,
                                                distribution: Some(StackupAnalysis::calculate_distribution_params(feature)),
                                            };
    
                                            if let Some(idx) = contribution_index {
                                                analysis.contributions[idx] = contribution;
                                            } else {
                                                analysis.contributions.push(contribution);
                                            }
    
                                            changed = true;
                                        }
                                    }
                                    should_close = true;
                                }
                            });
                        });
                    });
    
                if should_close {
                    self.open = false;
                    on_close();
                }
    
                result.map(|_| changed)
            } else {
                None
            }
        }
    
        pub fn open(&mut self, contribution: Option<&StackupContribution>) {
            self.open = true;
            
            if let Some(contribution) = contribution {
                self.component_id = contribution.component_id.clone();
                self.feature_id = contribution.feature_id.clone();
                self.direction = contribution.direction;
                self.half_count = contribution.half_count;
            } else {
                self.component_id.clear();
                self.feature_id.clear();
                self.direction = 1.0;
                self.half_count = false;
            }
        }
    }