// src/input/feature.rs
use super::command::Command;
use anyhow::{Result, anyhow};
use crossterm::event::KeyCode;
use tui_input::Input;
use crate::state::*;
use crate::config::{Feature, FeatureType};
use crate::state::input_state::{InputMode, EditField, FeatureInputs, ToleranceField};
use crate::state::ui_state::{DialogMode, ScreenMode};
use crate::state::mate_state::MateFilter;
use crate::analysis::stackup::{DistributionType, StackupAnalysis};

pub struct FeatureInputHandler;

impl FeatureInputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&self, key: KeyCode, state: &AppState) -> Option<Box<dyn Command>> {
        match (&state.input.mode, &state.ui.dialog_mode, key) {
            // Only handle 'f' key when in normal mode and component is selected
            (InputMode::Normal, DialogMode::None, KeyCode::Char('f')) => {
                if state.ui.component_list_state.selected().is_some() {
                    Some(Box::new(AddFeatureCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('m')) => {
                if state.ui.component_list_state.selected().is_some() &&
                    state.ui.feature_list_state.selected().is_some() {
                        Some(Box::new(ShowFeatureMatesCommand))
                    } else {
                        None
                    }
            },
            // Field editing commands
            (InputMode::Normal, DialogMode::AddFeature | DialogMode::EditFeature, KeyCode::Char('n')) => {
                Some(Box::new(EditFeatureNameCommand))
            },
            (InputMode::Normal, DialogMode::AddFeature | DialogMode::EditFeature, KeyCode::Char('v')) => {
                Some(Box::new(EditFeatureValueCommand))
            },
            (InputMode::Normal, DialogMode::AddFeature | DialogMode::EditFeature, KeyCode::Char('t')) => {
                Some(Box::new(EditFeatureToleranceCommand))
            },
            (InputMode::Editing(EditField::FeatureTolerance), _, KeyCode::Char('T')) => {
                Some(Box::new(ToggleToleranceFieldCommand))
            },
            (InputMode::Normal, DialogMode::AddFeature | DialogMode::EditFeature, KeyCode::Char('y')) => {
                Some(Box::new(ToggleFeatureTypeCommand))
            },
            // Save command
            (InputMode::Normal, DialogMode::AddFeature | DialogMode::EditFeature, KeyCode::Char('s')) => {
                Some(Box::new(SaveFeatureCommand))
            },
            // Finish editing field without saving feature
            (InputMode::Editing(EditField::FeatureName), _, KeyCode::Enter) |
            (InputMode::Editing(EditField::FeatureValue), _, KeyCode::Enter) |
            (InputMode::Editing(EditField::FeatureTolerance), _, KeyCode::Enter) => {
                Some(Box::new(FinishFeatureEditingCommand))
            },
            (InputMode::Normal, DialogMode::AddFeature | DialogMode::EditFeature, KeyCode::Char('d')) => {
                Some(Box::new(CycleFeatureDistributionCommand))
            },
            (InputMode::Editing(_), _, KeyCode::Char(c)) => {
                Some(Box::new(FeatureInputCharCommand(c)))
            },
            (InputMode::Editing(_), _, KeyCode::Backspace) => {
                Some(Box::new(FeatureDeleteCharCommand))
            },
            _ => None,
        }
    }
}

pub struct FinishFeatureEditingCommand;
impl Command for FinishFeatureEditingCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        // Just return to normal mode without saving the feature
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct ToggleFeatureTypeCommand;
impl Command for ToggleFeatureTypeCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.feature_inputs.feature_type = match state.input.feature_inputs.feature_type {
            FeatureType::External => FeatureType::Internal,
            FeatureType::Internal => FeatureType::External,
        };
        // Ensure we stay in dialog mode
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}


pub struct AddFeatureCommand;
impl Command for AddFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if state.ui.component_list_state.selected().is_some() {
            state.ui.dialog_mode = DialogMode::AddFeature;
            state.input.mode = InputMode::Editing(EditField::FeatureName);
            state.input.feature_inputs = FeatureInputs {
                name: Input::default(),
                feature_type: FeatureType::External,
                value: Input::default(),
                plus_tolerance: Input::default(),
                minus_tolerance: Input::default(),
                distribution: Some(DistributionType::Normal), // Set default distribution
            };
        }
        Ok(())
    }
}

pub struct FinishEditingCommand;
impl Command for FinishEditingCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct EditFeatureCommand;
impl Command for EditFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(comp_idx) = state.ui.component_list_state.selected() {
            if let Some(feat_idx) = state.ui.feature_list_state.selected() {
                if let Some(component) = state.project.components.get(comp_idx) {
                    if let Some(feature) = component.features.get(feat_idx) {
                        state.ui.dialog_mode = DialogMode::EditFeature;
                        state.input.feature_inputs = FeatureInputs {
                            name: Input::new(feature.name.clone()),
                            feature_type: feature.feature_type.clone(),
                            value: Input::new(feature.dimension.value.to_string()),
                            plus_tolerance: Input::new(feature.dimension.plus_tolerance.to_string()),
                            minus_tolerance: Input::new(feature.dimension.minus_tolerance.to_string()),
                            distribution: feature.distribution, // Make sure we load the existing distribution
                        };
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct CycleFeatureDistributionCommand;
impl Command for CycleFeatureDistributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.feature_inputs.distribution = Some(match state.input.feature_inputs.distribution.unwrap_or(DistributionType::Normal) {
            DistributionType::Normal => DistributionType::Uniform,
            DistributionType::Uniform => DistributionType::Triangular,
            DistributionType::Triangular => DistributionType::LogNormal,
            DistributionType::LogNormal => DistributionType::Normal,
        });
        Ok(())
    }
}

pub struct EditFeatureNameCommand;
impl Command for EditFeatureNameCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::FeatureName);
        Ok(())
    }
}

pub struct EditFeatureValueCommand;
impl Command for EditFeatureValueCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::FeatureValue);
        Ok(())
    }
}

pub struct EditFeatureToleranceCommand;
impl Command for EditFeatureToleranceCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::FeatureTolerance);
        Ok(())
    }
}

pub struct ToggleToleranceFieldCommand;
impl Command for ToggleToleranceFieldCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.tolerance_focus = match state.input.tolerance_focus {
            ToleranceField::Plus => ToleranceField::Minus,
            ToleranceField::Minus => ToleranceField::Plus,
        };
        Ok(())
    }
}

pub struct SaveFeatureCommand;
impl Command for SaveFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        // Only proceed if we're in a feature dialog mode
        match state.ui.dialog_mode {
            DialogMode::AddFeature | DialogMode::EditFeature => {
                let name = state.input.feature_inputs.name.value().trim();
                if name.is_empty() {
                    return Err(anyhow!("Feature name cannot be empty"));
                }

                // Check if all required fields are filled
                let value = state.input.feature_inputs.value.value().trim();
                let plus_tol = state.input.feature_inputs.plus_tolerance.value().trim();
                let minus_tol = state.input.feature_inputs.minus_tolerance.value().trim();

                if value.is_empty() || plus_tol.is_empty() || minus_tol.is_empty() {
                    return Err(anyhow!("Please fill in all feature fields"));
                }

                let value = value.parse::<f64>()
                    .map_err(|_| anyhow!("Invalid value"))?;
                let plus_tol = plus_tol.parse::<f64>()
                    .map_err(|_| anyhow!("Invalid positive tolerance"))?;
                let minus_tol = minus_tol.parse::<f64>()
                    .map_err(|_| anyhow!("Invalid negative tolerance"))?;

                    let mut feature = Feature::new(
                        name.to_string(),
                        state.input.feature_inputs.feature_type.clone(),
                        value,
                        plus_tol,
                        minus_tol,
                    );
    
                    feature.distribution = state.input.feature_inputs.distribution;
                    if let Some(dist_type) = feature.distribution {
                        feature.update_distribution(dist_type);
                    }
    
                    if let Some(comp_idx) = state.ui.component_list_state.selected() {
                        if let Some(component) = state.project.components.get_mut(comp_idx) {
                            match state.ui.dialog_mode {
                                DialogMode::AddFeature => {
                                    component.features.push(feature);
                                },
                                DialogMode::EditFeature => {
                                    if let Some(feat_idx) = state.ui.feature_list_state.selected() {
                                        if let Some(existing_feature) = component.features.get_mut(feat_idx) {
                                            // Store component and feature names before updating
                                            let comp_name = component.name.clone();
                                            let feat_name = existing_feature.name.clone();
                                            *existing_feature = feature;
    
                                            // Update distribution parameters in all analyses that use this feature
                                            for analysis in &mut state.analysis.analyses {
                                                for contribution in &mut analysis.contributions {
                                                    if contribution.component_id == comp_name && 
                                                       contribution.feature_id == feat_name {
                                                        // Recalculate distribution parameters for this contribution
                                                        contribution.distribution = Some(
                                                            StackupAnalysis::calculate_distribution_params(existing_feature)
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                            state.file_manager.save_project(
                                &state.project.project_file, 
                                &state.project.components,
                                &state.analysis.analyses
                            )?;
                        }
                    }
    
                    state.ui.dialog_mode = DialogMode::None;
                    state.input.mode = InputMode::Normal;
                },
                _ => {}
            }
        Ok(())
    }
}

pub struct DeleteFeatureCommand;
impl Command for DeleteFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(comp_idx) = state.ui.component_list_state.selected() {
            // First check if the component has any features
            if let Some(component) = state.project.components.get(comp_idx) {
                if component.features.is_empty() {
                    return Ok(());  // Early return if no features
                }
            }

            if let Some(feat_idx) = state.ui.feature_list_state.selected() {
                if let Some(component) = state.project.components.get_mut(comp_idx) {
                    component.features.remove(feat_idx);

                    // Update selection after deletion
                    if component.features.is_empty() {
                        state.ui.feature_list_state.select(None);  // Clear selection if no features left
                    } else if feat_idx >= component.features.len() {
                        state.ui.feature_list_state.select(Some(component.features.len() - 1));
                    }

                    // Save changes to file
                    state.file_manager.save_project(
                        &state.project.project_file,
                        &state.project.components,
                        &state.analysis.analyses,
                    )?;
                }
            }
        }
        state.ui.dialog_mode = DialogMode::None;
        Ok(())
    }
}

pub struct NextFeatureCommand;
impl Command for NextFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(comp_idx) = state.ui.component_list_state.selected() {
            if let Some(component) = state.project.components.get(comp_idx) {
                let i = match state.ui.feature_list_state.selected() {
                    Some(i) => {
                        if i >= component.features.len().saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.ui.feature_list_state.select(Some(i));
            }
        }
        Ok(())
    }
}

pub struct PrevFeatureCommand;
impl Command for PrevFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(comp_idx) = state.ui.component_list_state.selected() {
            if let Some(component) = state.project.components.get(comp_idx) {
                let i = match state.ui.feature_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            component.features.len().saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.ui.feature_list_state.select(Some(i));
            }
        }
        Ok(())
    }
}

pub struct CreateMateFromFeatureCommand;
impl Command for CreateMateFromFeatureCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let (Some(comp_idx), Some(feat_idx)) = (
            state.ui.component_list_state.selected(),
            state.ui.feature_list_state.selected()
        ) {
            if let Some(component) = state.project.components.get(comp_idx) {
                if let Some(feature) = component.features.get(feat_idx) {
                    // Pre-fill the first component and feature
                    state.input.mate_inputs.component_a = Input::new(component.name.clone());
                    state.input.mate_inputs.feature_a = Input::new(feature.name.clone());
                    state.ui.dialog_mode = DialogMode::AddMate;
                }
            }
        }
        Ok(())
    }
}

pub struct ShowFeatureMatesCommand;
impl Command for ShowFeatureMatesCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let (Some(comp_idx), Some(feat_idx)) = (
            state.ui.component_list_state.selected(),
            state.ui.feature_list_state.selected()
        ) {
            if let Some(component) = state.project.components.get(comp_idx) {
                if let Some(feature) = component.features.get(feat_idx) {
                    // Set feature-level filter and switch to mates tab
                    state.mates.filter = Some(MateFilter::Feature(
                        component.name.clone(),
                        feature.name.clone()
                    ));
                    state.ui.current_screen = ScreenMode::Mates;
                    state.ui.mate_list_state.select(None);
                }
            }
        }
        Ok(())
    }
}

pub struct FeatureInputCharCommand(pub char);
impl Command for FeatureInputCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::FeatureName) => {
                let new_value = format!("{}{}", state.input.feature_inputs.name.value(), self.0);
                state.input.feature_inputs.name = Input::new(new_value);
            },
            InputMode::Editing(EditField::FeatureValue) => {
                let new_value = format!("{}{}", state.input.feature_inputs.value.value(), self.0);
                state.input.feature_inputs.value = Input::new(new_value);
            },
            InputMode::Editing(EditField::FeatureTolerance) => {
                match state.input.tolerance_focus {
                    ToleranceField::Plus => {
                        let new_value = format!("{}{}", state.input.feature_inputs.plus_tolerance.value(), self.0);
                        state.input.feature_inputs.plus_tolerance = Input::new(new_value);
                    },
                    ToleranceField::Minus => {
                        let new_value = format!("{}{}", state.input.feature_inputs.minus_tolerance.value(), self.0);
                        state.input.feature_inputs.minus_tolerance = Input::new(new_value);
                    },
                }
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct FeatureDeleteCharCommand;
impl Command for FeatureDeleteCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::FeatureName) => {
                let mut value = state.input.feature_inputs.name.value().to_string();
                value.pop();
                state.input.feature_inputs.name = Input::new(value);
            },
            InputMode::Editing(EditField::FeatureValue) => {
                let mut value = state.input.feature_inputs.value.value().to_string();
                value.pop();
                state.input.feature_inputs.value = Input::new(value);
            },
            InputMode::Editing(EditField::FeatureTolerance) => {
                match state.input.tolerance_focus {
                    ToleranceField::Plus => {
                        let mut value = state.input.feature_inputs.plus_tolerance.value().to_string();
                        value.pop();
                        state.input.feature_inputs.plus_tolerance = Input::new(value);
                    },
                    ToleranceField::Minus => {
                        let mut value = state.input.feature_inputs.minus_tolerance.value().to_string();
                        value.pop();
                        state.input.feature_inputs.minus_tolerance = Input::new(value);
                    },
                }
            },
            _ => {}
        }
        Ok(())
    }
}

