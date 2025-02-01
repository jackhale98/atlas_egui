// src/input/mate.rs
use super::command::Command;
use anyhow::{Result, anyhow};
use crossterm::event::KeyCode;
use crate::state::*;
use crate::state::ui_state::DialogMode;
use crate::state::mate_state::get_component_by_name;
use crate::config::mate::{Mate, FitType};
use crate::state::input_state::{InputMode, MateInputs,
                                EditField, MateSelectionState};
use uuid::Uuid;
use ratatui::widgets::ListState;
use tui_input::Input;
use crate::file::mates::MatesFile;

pub struct MateInputHandler;

impl MateInputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&self, key: KeyCode, state: &AppState) -> Option<Box<dyn Command>> {
        match (&state.input.mode, &state.ui.dialog_mode, key) {
            // Initiate mate creation/editing
            (InputMode::Normal, DialogMode::None, KeyCode::Char('m')) => {
                Some(Box::new(AddMateCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('e')) => {
                if state.ui.mate_list_state.selected().is_some() {
                    Some(Box::new(EditMateCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('d')) => {
                if state.ui.mate_list_state.selected().is_some() {
                    Some(Box::new(DeleteMateCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('c')) => {
                if state.mates.filter.is_some() {
                    Some(Box::new(ClearFilterCommand))
                } else {
                    None
                }
            },

            // Navigation in mate dialog
            (InputMode::Normal, DialogMode::AddMate | DialogMode::EditMate, KeyCode::Char('j')) => {
                Some(Box::new(SelectionDownCommand))
            },
            (InputMode::Normal, DialogMode::AddMate | DialogMode::EditMate, KeyCode::Char('k')) => {
                Some(Box::new(SelectionUpCommand))
            },
            (InputMode::Normal, DialogMode::AddMate | DialogMode::EditMate, KeyCode::Enter) => {
                Some(Box::new(ConfirmSelectionCommand))
            },
            (InputMode::Normal, DialogMode::AddMate | DialogMode::EditMate, KeyCode::Char('t')) => {
                Some(Box::new(ToggleFitTypeCommand))
            },
            (InputMode::Normal, DialogMode::AddMate | DialogMode::EditMate, KeyCode::Char('s')) => {
                Some(Box::new(SaveMateCommand))
            },

            // Handle mate list navigation
            (InputMode::Normal, DialogMode::None, KeyCode::Char('j')) => {
                Some(Box::new(NextMateCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('k')) => {
                Some(Box::new(PrevMateCommand))
            },

            _ => None,
        }
    }
}

pub struct ClearFilterCommand;
impl Command for ClearFilterCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.mates.filter = None;
        state.ui.mate_list_state.select(None);
        Ok(())
    }
}

pub struct AddMateCommand;
impl Command for AddMateCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.dialog_mode = DialogMode::AddMate;
        state.input.mate_inputs = MateInputs::default();
        state.input.mate_selection_state = MateSelectionState::SelectingComponentA;
        // Reset all selection states
        state.ui.component_list_state.select(Some(0));
        state.ui.feature_list_state.select(Some(0));
        Ok(())
    }
}


pub struct EditMateCommand;
impl Command for EditMateCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.mate_list_state.selected() {
            if let Some(mate) = state.mates.mates.get(idx) {
                state.ui.dialog_mode = DialogMode::EditMate;
                state.input.mate_inputs = MateInputs {
                    component_a: Input::new(mate.component_a.clone()),
                    feature_a: Input::new(mate.feature_a.clone()),
                    component_b: Input::new(mate.component_b.clone()),
                    feature_b: Input::new(mate.feature_b.clone()),
                    fit_type: mate.fit_type.clone(),
                };
                state.input.mate_selection_state = MateSelectionState::SelectingComponentA;
                // Reset selection states
                state.ui.component_list_state.select(Some(0));
                state.ui.feature_list_state.select(Some(0));
            }
        }
        Ok(())
    }
}


pub struct SelectionUpCommand;
impl Command for SelectionUpCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mate_selection_state {
            MateSelectionState::SelectingComponentA | MateSelectionState::SelectingComponentB => {
                let list_len = state.project.components.len();
                navigate_list_up(&mut state.ui.component_list_state, list_len)
            },
            MateSelectionState::SelectingFeatureA => {
                if let Some(comp) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_a.value()) {
                    let list_len = comp.features.len();
                    navigate_list_up(&mut state.ui.feature_list_state, list_len)
                }
            },
            MateSelectionState::SelectingFeatureB => {
                if let Some(comp) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_b.value()) {
                    let list_len = comp.features.len();
                    navigate_list_up(&mut state.ui.feature_list_state, list_len)
                }
            },
        }
        Ok(())
    }
}

pub struct SelectionDownCommand;
impl Command for SelectionDownCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mate_selection_state {
            MateSelectionState::SelectingComponentA | MateSelectionState::SelectingComponentB => {
                let list_len = state.project.components.len();
                navigate_list_down(&mut state.ui.component_list_state, list_len)
            },
            MateSelectionState::SelectingFeatureA => {
                if let Some(comp) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_a.value()) {
                    let list_len = comp.features.len();
                    navigate_list_down(&mut state.ui.feature_list_state, list_len)
                }
            },
            MateSelectionState::SelectingFeatureB => {
                if let Some(comp) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_b.value()) {
                    let list_len = comp.features.len();
                    navigate_list_down(&mut state.ui.feature_list_state, list_len)
                }
            },
        }
        Ok(())
    }
}

pub struct ConfirmSelectionCommand;
impl Command for ConfirmSelectionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mate_selection_state {
            MateSelectionState::SelectingComponentA => {
                if let Some(idx) = state.ui.component_list_state.selected() {
                    if let Some(component) = state.project.components.get(idx) {
                        state.input.mate_inputs.component_a = Input::new(component.name.clone());
                        state.input.mate_selection_state = MateSelectionState::SelectingFeatureA;
                        // Reset feature selection state when switching components
                        state.ui.feature_list_state.select(Some(0));
                    }
                }
            },
            MateSelectionState::SelectingFeatureA => {
                if let Some(idx) = state.ui.feature_list_state.selected() {
                    if let Some(component) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_a.value()) {
                        if let Some(feature) = component.features.get(idx) {
                            state.input.mate_inputs.feature_a = Input::new(feature.name.clone());
                            state.input.mate_selection_state = MateSelectionState::SelectingComponentB;
                            // Reset component selection state for second component
                            state.ui.component_list_state.select(Some(0));
                        }
                    }
                }
            },
            MateSelectionState::SelectingComponentB => {
                if let Some(idx) = state.ui.component_list_state.selected() {
                    if let Some(component) = state.project.components.get(idx) {
                        state.input.mate_inputs.component_b = Input::new(component.name.clone());
                        state.input.mate_selection_state = MateSelectionState::SelectingFeatureB;
                        state.ui.feature_list_state.select(Some(0));
                    }
                }
            },
            MateSelectionState::SelectingFeatureB => {
                if let Some(idx) = state.ui.feature_list_state.selected() {
                    if let Some(component) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_b.value()) {
                        if let Some(feature) = component.features.get(idx) {
                            state.input.mate_inputs.feature_b = Input::new(feature.name.clone());
                            // Stay in SelectingFeatureB state to allow for changes
                        }
                    }
                }
            },
        }
        Ok(())
    }
}

fn navigate_list_up(list_state: &mut ListState, list_length: usize) {
    let new_index = match list_state.selected() {
        Some(i) => if i == 0 { list_length - 1 } else { i - 1 },
        None => 0,
    };
    list_state.select(Some(new_index));
}

fn navigate_list_down(list_state: &mut ListState, list_length: usize) {
    let new_index = match list_state.selected() {
        Some(i) => if i >= list_length - 1 { 0 } else { i + 1 },
        None => 0,
    };
    list_state.select(Some(new_index));
}

pub struct NextMateCommand;
impl Command for NextMateCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let filtered_mates = state.mates.filtered_mates();
        if !filtered_mates.is_empty() {
            let i = match state.ui.mate_list_state.selected() {
                Some(i) => {
                    if i >= filtered_mates.len().saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            state.ui.mate_list_state.select(Some(i));
        }
        Ok(())
    }
}

pub struct PrevMateCommand;
impl Command for PrevMateCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let filtered_mates = state.mates.filtered_mates();
        if !filtered_mates.is_empty() {
            let i = match state.ui.mate_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        filtered_mates.len().saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            state.ui.mate_list_state.select(Some(i));
        }
        Ok(())
    }
}

pub struct DeleteMateCommand;
impl Command for DeleteMateCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.mate_list_state.selected() {
            state.mates.mates.remove(idx);

            // Update selection
            if state.mates.mates.is_empty() {
                state.ui.mate_list_state.select(None);
            } else if idx >= state.mates.mates.len() {
                state.ui.mate_list_state.select(Some(state.mates.mates.len() - 1));
            }

            // Update dependency graph
            state.mates.update_dependency_graph(&state.project.components);

            // Create and save MatesFile
            let mates_file = MatesFile {
                version: "1.0.0".to_string(),
                mates: state.mates.mates.clone(),
            };
            state.file_manager.save_mates(&mates_file)?;
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

pub struct SaveMateCommand;
impl Command for SaveMateCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let inputs = &state.input.mate_inputs;

        // Basic validation
        if inputs.component_a.value().trim().is_empty() ||
           inputs.feature_a.value().trim().is_empty() ||
           inputs.component_b.value().trim().is_empty() ||
           inputs.feature_b.value().trim().is_empty() {
            return Err(anyhow!("All fields must be filled"));
        }

        let new_mate = Mate::new(
            Uuid::new_v4().to_string(),
            inputs.component_a.value().trim().to_string(),
            inputs.feature_a.value().trim().to_string(),
            inputs.component_b.value().trim().to_string(),
            inputs.feature_b.value().trim().to_string(),
            inputs.fit_type.clone(),
        );

        match state.ui.dialog_mode {
            DialogMode::AddMate => {
                state.mates.mates.push(new_mate);
            },
            DialogMode::EditMate => {
                if let Some(idx) = state.ui.selected_mate {
                    if let Some(mate) = state.mates.mates.get_mut(idx) {
                        *mate = new_mate;
                    }
                }
            },
            _ => return Ok(()),
        }

        // Update dependency graph
        state.mates.update_dependency_graph(&state.project.components);

        // Create and save MatesFile
        let mates_file = MatesFile {
            version: "1.0.0".to_string(),
            mates: state.mates.mates.clone(),
        };
        state.file_manager.save_mates(&mates_file)?;

        state.ui.dialog_mode = DialogMode::None;
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct EditMateComponentACommand;
impl Command for EditMateComponentACommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MateComponentA);
        Ok(())
    }
}

pub struct EditMateFeatureACommand;
impl Command for EditMateFeatureACommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MateFeatureA);
        Ok(())
    }
}

pub struct EditMateComponentBCommand;
impl Command for EditMateComponentBCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MateComponentB);
        Ok(())
    }
}

pub struct EditMateFeatureBCommand;
impl Command for EditMateFeatureBCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::MateFeatureB);
        Ok(())
    }
}

pub struct ToggleFitTypeCommand;
impl Command for ToggleFitTypeCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mate_inputs.fit_type = match state.input.mate_inputs.fit_type {
            FitType::Clearance => FitType::Transition,
            FitType::Transition => FitType::Interference,
            FitType::Interference => FitType::Clearance,
        };
        Ok(())
    }
}

pub struct MateInputCharCommand(pub char);
impl Command for MateInputCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::MateComponentA) => {
                let new_value = format!("{}{}", state.input.mate_inputs.component_a.value(), self.0);
                state.input.mate_inputs.component_a = Input::new(new_value);
            },
            InputMode::Editing(EditField::MateFeatureA) => {
                let new_value = format!("{}{}", state.input.mate_inputs.feature_a.value(), self.0);
                state.input.mate_inputs.feature_a = Input::new(new_value);
            },
            InputMode::Editing(EditField::MateComponentB) => {
                let new_value = format!("{}{}", state.input.mate_inputs.component_b.value(), self.0);
                state.input.mate_inputs.component_b = Input::new(new_value);
            },
            InputMode::Editing(EditField::MateFeatureB) => {
                let new_value = format!("{}{}", state.input.mate_inputs.feature_b.value(), self.0);
                state.input.mate_inputs.feature_b = Input::new(new_value);
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct MateDeleteCharCommand;
impl Command for MateDeleteCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::MateComponentA) => {
                let mut value = state.input.mate_inputs.component_a.value().to_string();
                value.pop();
                state.input.mate_inputs.component_a = Input::new(value);
            },
            InputMode::Editing(EditField::MateFeatureA) => {
                let mut value = state.input.mate_inputs.feature_a.value().to_string();
                value.pop();
                state.input.mate_inputs.feature_a = Input::new(value);
            },
            InputMode::Editing(EditField::MateComponentB) => {
                let mut value = state.input.mate_inputs.component_b.value().to_string();
                value.pop();
                state.input.mate_inputs.component_b = Input::new(value);
            },
            InputMode::Editing(EditField::MateFeatureB) => {
                let mut value = state.input.mate_inputs.feature_b.value().to_string();
                value.pop();
                state.input.mate_inputs.feature_b = Input::new(value);
            },
            _ => {}
        }
        Ok(())
    }
}
