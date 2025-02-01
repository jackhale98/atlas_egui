// src/input/analysis.rs

use super::command::Command;
use anyhow::{Result, anyhow};
use crossterm::event::KeyCode;
use crate::state::*;
use crate::state::ui_state::{DialogMode, AnalysisTab};
use crate::state::input_state::{InputMode, EditField};
use crate::analysis::{AnalysisMethod, 
                    StackupAnalysis, DistributionType, DistributionParams, StackupContribution};
use tui_input::Input;
use crate::state::input_state::ContributionSelectionState;
use crate::config::Feature;

pub struct AnalysisInputHandler;

impl AnalysisInputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&self, key: KeyCode, state: &AppState) -> Option<Box<dyn Command>> {
        match (&state.input.mode, &state.ui.dialog_mode, key) {
            // Tab navigation 
            (InputMode::Normal, DialogMode::None, KeyCode::Char('l')) => {
                Some(Box::new(NextAnalysisTabCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('h')) => {
                Some(Box::new(PrevAnalysisTabCommand))
            },
    
            // Analysis list commands
            (InputMode::Normal, DialogMode::None, KeyCode::Char('a')) => {
                Some(Box::new(AddAnalysisCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('e')) => {
                if state.ui.analysis_list_state.selected().is_some() {
                    Some(Box::new(EditAnalysisCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('d')) => {
                if state.ui.analysis_list_state.selected().is_some() {
                    Some(Box::new(DeleteAnalysisCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::AddContribution, KeyCode::Enter) => {
                Some(Box::new(ContributionConfirmSelectionCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('r')) => {
                if state.ui.analysis_list_state.selected().is_some() {
                    Some(Box::new(RunAnalysisCommand))
                } else {
                    None
                }
            },
    
            // Analysis navigation
            (InputMode::Normal, DialogMode::None, KeyCode::Char('j')) => {
                Some(Box::new(NextAnalysisCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('k')) => {
                Some(Box::new(PrevAnalysisCommand))
            },




    
            // Analysis dialog commands
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('j')) => {
                Some(Box::new(NextContributionCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('k')) => {
                Some(Box::new(PrevContributionCommand))
            },
            
            // Contribution selection navigation
            (InputMode::Normal, DialogMode::AddContribution, KeyCode::Char('j')) => {
                Some(Box::new(ContributionSelectionDownCommand))
            },
            (InputMode::Normal, DialogMode::AddContribution, KeyCode::Char('k')) => {
                Some(Box::new(ContributionSelectionUpCommand))
            },
                    // Edit/Delete contribution
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('e')) => {
                if state.ui.contribution_list_state.selected().is_some() {
                    Some(Box::new(EditContributionCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('d')) => {
                if state.ui.contribution_list_state.selected().is_some() {
                    Some(Box::new(DeleteContributionCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('n')) => {
                Some(Box::new(EditAnalysisNameCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('m')) => {
                Some(Box::new(ToggleMethodCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('c')) => {
                Some(Box::new(AddContributionCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('s')) => {
                Some(Box::new(SaveAnalysisCommand))
            },
    
            // Monte Carlo settings
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('i')) => {
                Some(Box::new(EditMonteCarloIterationsCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('f')) => {
                Some(Box::new(EditMonteCarloConfidenceCommand)) 
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('x')) => {
                Some(Box::new(EditMonteCarloSeedCommand))
            },
    
            // Contribution dialog commands
            (InputMode::Normal, DialogMode::AddContribution | DialogMode::EditContribution, KeyCode::Char('d')) => {
                Some(Box::new(ToggleDirectionCommand))
            },
            (InputMode::Normal, DialogMode::AddContribution | DialogMode::EditContribution, KeyCode::Char('h')) => {
                Some(Box::new(ToggleHalfCountCommand))
            },
            (InputMode::Normal, DialogMode::AddContribution | DialogMode::EditContribution, KeyCode::Char('t')) => {
                Some(Box::new(CycleDistributionTypeCommand))
            },
            (InputMode::Normal, DialogMode::AddContribution | DialogMode::EditContribution, KeyCode::Char('p')) => {
                Some(Box::new(EditDistributionParamsCommand))
            },
            (InputMode::Normal, DialogMode::AddContribution | DialogMode::EditContribution, KeyCode::Char('s')) => {
                Some(Box::new(SaveContributionCommand))
            },
    
            // Text input mode
            (InputMode::Editing(_), _, KeyCode::Enter) => {
                Some(Box::new(FinishEditingCommand))
            },
            (InputMode::Editing(_), _, KeyCode::Char(c)) => {
                Some(Box::new(AnalysisInputCharCommand(c)))
            },
            (InputMode::Editing(_), _, KeyCode::Backspace) => {
                Some(Box::new(AnalysisDeleteCharCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char(' ')) => {
                if state.ui.method_list_state.selected().is_some() {
                    Some(Box::new(ToggleMethodCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('J')) => {
                Some(Box::new(NextMethodCommand))
            },
            (InputMode::Normal, DialogMode::AddAnalysis | DialogMode::EditAnalysis, KeyCode::Char('K')) => {
                Some(Box::new(PrevMethodCommand))
            },    
            (InputMode::Editing(EditField::MonteCarloIterations), _, KeyCode::Backspace) |
            (InputMode::Editing(EditField::MonteCarloConfidence), _, KeyCode::Backspace) |
            (InputMode::Editing(EditField::MonteCarloSeed), _, KeyCode::Backspace) => {
                Some(Box::new(AnalysisDeleteCharCommand))
            },
    
            _ => None,
        }
    }
}

// Tab navigation commands
pub struct NextAnalysisTabCommand;
impl Command for NextAnalysisTabCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.analysis_tab = match state.ui.analysis_tab {
            AnalysisTab::List => AnalysisTab::Details,
            AnalysisTab::Details => AnalysisTab::Results,
            AnalysisTab::Results => AnalysisTab::Visualization,
            AnalysisTab::Visualization => AnalysisTab::List,
        };
        Ok(())
    }
}

pub struct PrevAnalysisTabCommand;
impl Command for PrevAnalysisTabCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.analysis_tab = match state.ui.analysis_tab {
            AnalysisTab::List => AnalysisTab::Visualization,
            AnalysisTab::Details => AnalysisTab::List,
            AnalysisTab::Results => AnalysisTab::Details,
            AnalysisTab::Visualization => AnalysisTab::Results,
        };
        Ok(())
    }
}

// Analysis management commands
pub struct AddAnalysisCommand;
impl Command for AddAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.analysis_inputs = Default::default();
        state.ui.dialog_mode = DialogMode::AddAnalysis;
        state.ui.method_list_state.select(Some(0));
        Ok(())
    }
}

pub struct EditAnalysisCommand;
impl Command for EditAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.analysis_list_state.selected() {
            if let Some(analysis) = state.analysis.analyses.get(idx) {
                state.ui.dialog_mode = DialogMode::EditAnalysis;
                state.ui.selected_analysis_index = Some(idx); // Add this field to UiState
                state.input.analysis_inputs.name = Input::new(analysis.name.clone());
                state.input.analysis_inputs.methods = analysis.methods.clone();
                state.input.analysis_inputs.monte_carlo_settings = analysis.monte_carlo_settings.clone()
                    .unwrap_or_default();
                state.input.analysis_inputs.contributions = analysis.contributions.clone();
                state.input.analysis_inputs.id = Some(analysis.id.clone()); // Add this field to AnalysisInputs
                state.ui.method_list_state.select(Some(0));
            }
        }
        Ok(())
    }
}

pub struct DeleteAnalysisCommand;
impl Command for DeleteAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.analysis_list_state.selected() {
            state.analysis.analyses.remove(idx);
            
            // Update selection
            if state.analysis.analyses.is_empty() {
                state.ui.analysis_list_state.select(None);
            } else if idx >= state.analysis.analyses.len() {
                state.ui.analysis_list_state.select(Some(state.analysis.analyses.len() - 1));
            }

            // Save updated project file
            state.file_manager.save_project(&state.project.project_file, &state.project.components, &state.analysis.analyses,)?;
        }
        Ok(())
    }
}

pub struct NextAnalysisCommand;
impl Command for NextAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if !state.analysis.analyses.is_empty() {
            let i = match state.ui.analysis_list_state.selected() {
                Some(i) => {
                    if i >= state.analysis.analyses.len().saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            state.ui.analysis_list_state.select(Some(i));
        }
        Ok(())
    }
}

pub struct PrevAnalysisCommand;
impl Command for PrevAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if !state.analysis.analyses.is_empty() {
            let i = match state.ui.analysis_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        state.analysis.analyses.len().saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            state.ui.analysis_list_state.select(Some(i));
        }
        Ok(())
    }
}

pub struct EditAnalysisNameCommand;
impl Command for EditAnalysisNameCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::AnalysisName);
        Ok(())
    }
}

pub struct ToggleMethodCommand;
impl Command for ToggleMethodCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.method_list_state.selected() {
            let method = match idx {
                0 => AnalysisMethod::WorstCase,
                1 => AnalysisMethod::Rss,
                2 => AnalysisMethod::MonteCarlo,
                _ => return Ok(()),
            };

            if state.input.analysis_inputs.methods.contains(&method) {
                state.input.analysis_inputs.methods.retain(|&m| m != method);
            } else {
                state.input.analysis_inputs.methods.push(method);
            }

            // Ensure list state stays selected
            state.ui.method_list_state.select(Some(idx));
        }
        Ok(())
    }
}

pub struct NextMethodCommand;
impl Command for NextMethodCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(i) = state.ui.method_list_state.selected() {
            if i < 2 {
                state.ui.method_list_state.select(Some(i + 1));
            } else {
                state.ui.method_list_state.select(Some(0));
            }
        } else {
            state.ui.method_list_state.select(Some(0));
        }
        Ok(())
    }
}

pub struct PrevMethodCommand;
impl Command for PrevMethodCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(i) = state.ui.method_list_state.selected() {
            if i > 0 {
                state.ui.method_list_state.select(Some(i - 1));
            } else {
                state.ui.method_list_state.select(Some(2));
            }
        } else {
            state.ui.method_list_state.select(Some(2));
        }
        Ok(())
    }
}


pub struct RunAnalysisCommand;
impl Command for RunAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.analysis_list_state.selected() {
            if let Some(analysis) = state.analysis.analyses.get(idx) {
                let results = analysis.run_analysis(&state.project.components);
                state.analysis.latest_results.insert(analysis.id.clone(), results);
                
                // Save results
                state.file_manager.analysis_handler.save_analysis(analysis, 
                    state.analysis.latest_results.get(&analysis.id).unwrap())?;
            }
        }
        Ok(())
    }
}

pub struct SaveAnalysisCommand;
impl Command for SaveAnalysisCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let inputs = &state.input.analysis_inputs;
        if inputs.name.value().trim().is_empty() {
            return Err(anyhow!("Analysis name cannot be empty"));
        }
        if inputs.methods.is_empty() {
            return Err(anyhow!("At least one analysis method must be selected"));
        }

        let mut analysis = StackupAnalysis::new(inputs.name.value().trim().to_string());
        analysis.methods = inputs.methods.clone();
        analysis.contributions = inputs.contributions.clone();
        
        if inputs.methods.contains(&AnalysisMethod::MonteCarlo) {
            analysis.monte_carlo_settings = Some(inputs.monte_carlo_settings.clone());
        }

        match state.ui.dialog_mode {
            DialogMode::AddAnalysis => {
                state.analysis.analyses.push(analysis);
            },
            DialogMode::EditAnalysis => {
                // Use the stored ID and index for updating
                if let Some(idx) = state.ui.selected_analysis_index {
                    if let Some(id) = &inputs.id {
                        analysis.id = id.clone();
                        if let Some(existing) = state.analysis.analyses.get_mut(idx) {
                            *existing = analysis;
                        }
                    }
                }
            },
            _ => return Ok(()),
        }

        // Save project file
        state.file_manager.save_project(
            &state.project.project_file,
            &state.project.components,
            &state.analysis.analyses
        )?;

        state.ui.dialog_mode = DialogMode::None;
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

// Contribution management commands
pub struct AddContributionCommand;
impl Command for AddContributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.contribution_inputs = Default::default();
        state.ui.dialog_mode = DialogMode::AddContribution;
        // Reset selection states
        state.ui.component_list_state.select(Some(0));
        state.ui.feature_list_state.select(None);
        state.input.contribution_selection_state = ContributionSelectionState::SelectingComponent;
        Ok(())
    }
}




pub struct ToggleDirectionCommand;
impl Command for ToggleDirectionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.contribution_inputs.direction *= -1.0;
        Ok(())
    }
}

pub struct ToggleHalfCountCommand;
impl Command for ToggleHalfCountCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.contribution_inputs.half_count = !state.input.contribution_inputs.half_count;
        Ok(())
    }
}

pub struct CycleDistributionTypeCommand;
impl Command for CycleDistributionTypeCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.contribution_inputs.distribution_type = match state.input.contribution_inputs.distribution_type {
            DistributionType::Normal => DistributionType::Uniform,
            DistributionType::Uniform => DistributionType::Triangular,
            DistributionType::Triangular => DistributionType::LogNormal,
            DistributionType::LogNormal => DistributionType::Normal,
        };
        Ok(())
    }
}

// Text input commands
pub struct AnalysisInputCharCommand(pub char);
impl Command for AnalysisInputCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::AnalysisName) => {
                let new_value = format!("{}{}", state.input.analysis_inputs.name.value(), self.0);
                state.input.analysis_inputs.name = Input::new(new_value);
            },
            InputMode::Editing(EditField::MonteCarloIterations) => {
                if self.0.is_ascii_digit() {
                    let new_value = format!("{}{}", 
                        state.input.analysis_inputs.monte_carlo_settings.iterations, self.0);
                    if let Ok(value) = new_value.parse::<usize>() {
                        state.input.analysis_inputs.monte_carlo_settings.iterations = value;
                    }
                }
            },
            InputMode::Editing(EditField::MonteCarloConfidence) => {
                if self.0.is_ascii_digit() || self.0 == '.' {
                    let current = state.input.analysis_inputs.monte_carlo_settings.confidence * 100.0; // Convert to percentage
                    let new_value = format!("{}{}", current, self.0);
                    if let Ok(value) = new_value.parse::<f64>() {
                        // Convert back to decimal and clamp to valid range
                        state.input.analysis_inputs.monte_carlo_settings.confidence = (value / 100.0).clamp(0.0, 0.9999);
                    }
                }
            },
            InputMode::Editing(EditField::MonteCarloSeed) => {
                if self.0.is_ascii_digit() {
                    let current = state.input.analysis_inputs.monte_carlo_settings.seed.unwrap_or(0);
                    let new_value = format!("{}{}", current, self.0);
                    if let Ok(value) = new_value.parse::<u64>() {
                        state.input.analysis_inputs.monte_carlo_settings.seed = Some(value);
                    }
                }
            },
            InputMode::Editing(EditField::DistributionParam1) => {
                let new_value = format!("{}{}", state.input.contribution_inputs.dist_param1.value(), self.0);
                state.input.contribution_inputs.dist_param1 = Input::new(new_value);
            },
            InputMode::Editing(EditField::DistributionParam2) => {
                let new_value = format!("{}{}", state.input.contribution_inputs.dist_param2.value(), self.0);
                state.input.contribution_inputs.dist_param2 = Input::new(new_value);
            },
            InputMode::Editing(EditField::DistributionParam3) => {
                let new_value = format!("{}{}", state.input.contribution_inputs.dist_param3.value(), self.0);
                state.input.contribution_inputs.dist_param3 = Input::new(new_value);
            },
            _ => {},
        }
        Ok(())
    }
}

pub struct AnalysisDeleteCharCommand;
impl Command for AnalysisDeleteCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::AnalysisName) => {
                let mut value = state.input.analysis_inputs.name.value().to_string();
                value.pop();
                state.input.analysis_inputs.name = Input::new(value);
            },
            InputMode::Editing(EditField::MonteCarloIterations) => {
                let mut value = state.input.analysis_inputs.monte_carlo_settings.iterations.to_string();
                value.pop();
                if let Ok(num) = value.parse::<usize>() {
                    state.input.analysis_inputs.monte_carlo_settings.iterations = num;
                }
            },
            InputMode::Editing(EditField::MonteCarloConfidence) => {
                let mut value = state.input.analysis_inputs.monte_carlo_settings.confidence.to_string();
                value.pop();
                if let Ok(num) = value.parse::<f64>() {
                    state.input.analysis_inputs.monte_carlo_settings.confidence = num;
                }
            },
            InputMode::Editing(EditField::MonteCarloSeed) => {
                if let Some(seed) = state.input.analysis_inputs.monte_carlo_settings.seed {
                    let mut value = seed.to_string();
                    value.pop();
                    if let Ok(num) = value.parse::<u64>() {
                        state.input.analysis_inputs.monte_carlo_settings.seed = Some(num);
                    }
                }
            },
            _ => {},
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

pub struct ContributionSelectionUpCommand;
impl Command for ContributionSelectionUpCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.contribution_selection_state {
            ContributionSelectionState::SelectingComponent => {
                if let Some(i) = state.ui.component_list_state.selected() {
                    let new_i = if i == 0 {
                        state.project.components.len().saturating_sub(1)
                    } else {
                        i - 1
                    };
                    state.ui.component_list_state.select(Some(new_i));
                }
            },
            ContributionSelectionState::SelectingFeature => {
                if let Some(comp_idx) = state.ui.component_list_state.selected() {
                    if let Some(component) = state.project.components.get(comp_idx) {
                        if let Some(i) = state.ui.feature_list_state.selected() {
                            let new_i = if i == 0 {
                                component.features.len().saturating_sub(1)
                            } else {
                                i - 1
                            };
                            state.ui.feature_list_state.select(Some(new_i));
                        }
                    }
                }
            },
        }
        Ok(())
    }
}

pub struct ContributionSelectionDownCommand;
impl Command for ContributionSelectionDownCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.contribution_selection_state {
            ContributionSelectionState::SelectingComponent => {
                if let Some(i) = state.ui.component_list_state.selected() {
                    let new_i = if i >= state.project.components.len().saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    };
                    state.ui.component_list_state.select(Some(new_i));
                }
            },
            ContributionSelectionState::SelectingFeature => {
                if let Some(comp_idx) = state.ui.component_list_state.selected() {
                    if let Some(component) = state.project.components.get(comp_idx) {
                        if let Some(i) = state.ui.feature_list_state.selected() {
                            let new_i = if i >= component.features.len().saturating_sub(1) {
                                0
                            } else {
                                i + 1
                            };
                            state.ui.feature_list_state.select(Some(new_i));
                        }
                    }
                }
            },
        }
        Ok(())
    }
}

pub struct NextContributionCommand;
impl Command for NextContributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(i) = state.ui.contribution_list_state.selected() {
            let new_i = if i >= state.input.analysis_inputs.contributions.len().saturating_sub(1) {
                0
            } else {
                i + 1
            };
            state.ui.contribution_list_state.select(Some(new_i));
        } else if !state.input.analysis_inputs.contributions.is_empty() {
            state.ui.contribution_list_state.select(Some(0));
        }
        Ok(())
    }
}

pub struct PrevContributionCommand;
impl Command for PrevContributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(i) = state.ui.contribution_list_state.selected() {
            let new_i = if i == 0 {
                state.input.analysis_inputs.contributions.len().saturating_sub(1)
            } else {
                i - 1
            };
            state.ui.contribution_list_state.select(Some(new_i));
        } else if !state.input.analysis_inputs.contributions.is_empty() {
            state.ui.contribution_list_state.select(Some(state.input.analysis_inputs.contributions.len() - 1));
        }
        Ok(())
    }
}

pub struct EditContributionCommand;
impl Command for EditContributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.contribution_list_state.selected() {
            if let Some(contribution) = state.input.analysis_inputs.contributions.get(idx) {
                state.ui.dialog_mode = DialogMode::EditContribution;
                state.input.contribution_inputs.direction = contribution.direction;
                state.input.contribution_inputs.half_count = contribution.half_count;
                
                // Find and select the component and feature
                for (i, component) in state.project.components.iter().enumerate() {
                    if component.name == contribution.component_id {
                        state.ui.component_list_state.select(Some(i));
                        for (j, feature) in component.features.iter().enumerate() {
                            if feature.name == contribution.feature_id {
                                state.ui.feature_list_state.select(Some(j));
                                break;
                            }
                        }
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct DeleteContributionCommand;
impl Command for DeleteContributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.contribution_list_state.selected() {
            state.input.analysis_inputs.contributions.remove(idx);
            if !state.input.analysis_inputs.contributions.is_empty() {
                if idx >= state.input.analysis_inputs.contributions.len() {
                    state.ui.contribution_list_state.select(
                        Some(state.input.analysis_inputs.contributions.len() - 1)
                    );
                }
            } else {
                state.ui.contribution_list_state.select(None);
            }
        }
        Ok(())
    }
}

pub struct ContributionConfirmSelectionCommand;
impl Command for ContributionConfirmSelectionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.contribution_selection_state {
            ContributionSelectionState::SelectingComponent => {
                if let Some(comp_idx) = state.ui.component_list_state.selected() {
                    if let Some(component) = state.project.components.get(comp_idx) {
                        state.input.contribution_selection_state = ContributionSelectionState::SelectingFeature;
                        state.ui.feature_list_state.select(Some(0));
                    }
                }
            },
            ContributionSelectionState::SelectingFeature => {
                if let Some(comp_idx) = state.ui.component_list_state.selected() {
                    if let Some(feat_idx) = state.ui.feature_list_state.selected() {
                        if let Some(component) = state.project.components.get(comp_idx) {
                            if let Some(feature) = component.features.get(feat_idx) {
                                // Create distribution params using feature's distribution type
                                let distribution = Some(StackupAnalysis::calculate_distribution_params(feature));

                                let contribution = StackupContribution {
                                    component_id: component.name.clone(),
                                    feature_id: feature.name.clone(),
                                    direction: state.input.contribution_inputs.direction,
                                    half_count: state.input.contribution_inputs.half_count,
                                    distribution,
                                };

                                state.input.analysis_inputs.contributions.push(contribution);
                                state.ui.dialog_mode = if state.input.analysis_inputs.id.is_some() {
                                    DialogMode::EditAnalysis
                                } else {
                                    DialogMode::AddAnalysis
                                };
                            }
                        }
                    }
                }
            },
        }
        Ok(())
    }
}


pub struct SaveContributionCommand;
impl Command for SaveContributionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(comp_idx) = state.ui.component_list_state.selected() {
            if let Some(feat_idx) = state.ui.feature_list_state.selected() {
                if let Some(component) = state.project.components.get(comp_idx) {
                    if let Some(feature) = component.features.get(feat_idx) {
                        let inputs = &state.input.contribution_inputs;
                        
                        // Create distribution parameters based on type
                        let distribution = Some(match inputs.distribution_type {
                            DistributionType::Normal => DistributionParams::new_normal(
                                inputs.dist_param1.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid mean value"))?,
                                inputs.dist_param2.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid standard deviation"))?
                            ),
                            DistributionType::Uniform => DistributionParams::new_uniform(
                                inputs.dist_param1.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid minimum value"))?,
                                inputs.dist_param2.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid maximum value"))?
                            ),
                            DistributionType::Triangular => DistributionParams::new_triangular(
                                inputs.dist_param1.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid minimum value"))?,
                                inputs.dist_param2.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid maximum value"))?,
                                inputs.dist_param3.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid mode value"))?
                            ),
                            DistributionType::LogNormal => DistributionParams::new_lognormal(
                                inputs.dist_param1.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid location parameter"))?,
                                inputs.dist_param2.value().parse::<f64>()
                                    .map_err(|_| anyhow!("Invalid scale parameter"))?
                            ),
                        });

                        let contribution = StackupContribution {
                            component_id: component.name.clone(),
                            feature_id: feature.name.clone(),
                            direction: inputs.direction,
                            half_count: inputs.half_count,
                            distribution,
                        };

                        match state.ui.dialog_mode {
                            DialogMode::AddContribution => {
                                state.input.analysis_inputs.contributions.push(contribution);
                            },
                            DialogMode::EditContribution => {
                                if let Some(idx) = state.ui.contribution_list_state.selected() {
                                    if let Some(existing) = state.input.analysis_inputs.contributions.get_mut(idx) {
                                        *existing = contribution;
                                    }
                                }
                            },
                            _ => {},
                        }

                        state.ui.dialog_mode = DialogMode::AddAnalysis;  // Return to analysis dialog
                        state.input.mode = InputMode::Normal;
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct EditDistributionParamsCommand;
impl Command for EditDistributionParamsCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let current_param = match state.input.mode {
            InputMode::Editing(EditField::DistributionParam1) => EditField::DistributionParam2,
            InputMode::Editing(EditField::DistributionParam2) => {
                if state.input.contribution_inputs.distribution_type == DistributionType::Triangular {
                    EditField::DistributionParam3
                } else {
                    EditField::DistributionParam1
                }
            },
            InputMode::Editing(EditField::DistributionParam3) => EditField::DistributionParam1,
            _ => EditField::DistributionParam1,
        };

        state.input.mode = InputMode::Editing(current_param);
        
        // Initialize parameter descriptions based on distribution type
        match state.input.contribution_inputs.distribution_type {
            DistributionType::Normal => {
                state.ui.parameter_descriptions = vec![
                    "Mean".to_string(),
                    "Standard Deviation".to_string(),
                ];
            },
            DistributionType::Uniform => {
                state.ui.parameter_descriptions = vec![
                    "Minimum".to_string(),
                    "Maximum".to_string(),
                ];
            },
            DistributionType::Triangular => {
                state.ui.parameter_descriptions = vec![
                    "Minimum".to_string(),
                    "Maximum".to_string(),
                    "Mode".to_string(),
                ];
            },
            DistributionType::LogNormal => {
                state.ui.parameter_descriptions = vec![
                    "Location (μ)".to_string(),
                    "Scale (σ)".to_string(),
                ];
            },
        }
        
        Ok(())
    }
}

pub struct EditMonteCarloSettingsCommand;
impl Command for EditMonteCarloSettingsCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let current_field = match state.input.mode {
            InputMode::Editing(EditField::MonteCarloIterations) => EditField::MonteCarloConfidence,
            InputMode::Editing(EditField::MonteCarloConfidence) => EditField::MonteCarloSeed,
            InputMode::Editing(EditField::MonteCarloSeed) => EditField::MonteCarloIterations,
            _ => EditField::MonteCarloIterations,
        };

        state.input.mode = InputMode::Editing(current_field);
        Ok(())
    }
}

pub struct EditMonteCarloIterationsCommand;
impl Command for EditMonteCarloIterationsCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MonteCarloIterations);
        Ok(())
    }
}

pub struct EditMonteCarloConfidenceCommand;
impl Command for EditMonteCarloConfidenceCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MonteCarloConfidence);
        Ok(())
    }
}

pub struct EditMonteCarloSeedCommand;
impl Command for EditMonteCarloSeedCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MonteCarloSeed);
        Ok(())
    }
}
