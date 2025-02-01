// src/input/component.rs
use super::command::Command;
use anyhow::{Result, anyhow};
use crossterm::event::KeyCode;
use tui_input::Input;
use crate::input::feature::{AddFeatureCommand, NextFeatureCommand,
                            PrevFeatureCommand, DeleteFeatureCommand,
                            EditFeatureCommand};
use crate::state::*;
use crate::state::ui_state::{DialogMode, ScreenMode};
use crate::state::input_state::EditField;
use crate::config::Component;
use crate::state::input_state::InputMode;
use crate::state::mate_state::MateFilter;

pub struct ComponentInputHandler;

impl ComponentInputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&self, key: KeyCode, state: &AppState) -> Option<Box<dyn Command>> {
        match (&state.input.mode, &state.ui.dialog_mode, key) {
            (InputMode::Normal, DialogMode::None, KeyCode::Char('a')) => {
                Some(Box::new(AddComponentCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('c')) => {
                Some(Box::new(EditComponentCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('D')) => {
                Some(Box::new(DeleteComponentCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('j')) => {
                Some(Box::new(NextComponentCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('k')) => {
                Some(Box::new(PrevComponentCommand))
            },
            (InputMode::Normal, DialogMode::AddComponent | DialogMode::EditComponent, KeyCode::Char('n')) => {
                Some(Box::new(EditComponentNameCommand))
            },
            (InputMode::Normal, DialogMode::AddComponent | DialogMode::EditComponent, KeyCode::Char('d')) => {
                Some(Box::new(EditComponentDescriptionCommand))
            },
            (InputMode::Normal, DialogMode::AddComponent | DialogMode::EditComponent, KeyCode::Char('s')) => {
                Some(Box::new(SaveComponentCommand))
            },
            (InputMode::Editing(EditField::ComponentName), _, KeyCode::Enter) |
            (InputMode::Editing(EditField::ComponentDescription), _, KeyCode::Enter) => {
                Some(Box::new(FinishComponentEditingCommand))
            },
            (InputMode::Editing(_), _, KeyCode::Char(c)) => {
                Some(Box::new(ComponentInputCharCommand(c)))
            },
            (InputMode::Editing(_), _, KeyCode::Backspace) => {
                Some(Box::new(ComponentDeleteCharCommand))
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('f')) => {
                if state.ui.component_list_state.selected().is_some() {
                    Some(Box::new(AddFeatureCommand))
                } else {
                    None
                }
            },
            // Feature navigation
            (InputMode::Normal, DialogMode::None, KeyCode::Char('J')) => {
                if state.ui.component_list_state.selected().is_some() {
                    Some(Box::new(NextFeatureCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('K')) => {
                if state.ui.component_list_state.selected().is_some() {
                    Some(Box::new(PrevFeatureCommand))
                } else {
                    None
                }
            },
            // Feature editing and deletion
            (InputMode::Normal, DialogMode::None, KeyCode::Char('e')) => {
                if state.ui.component_list_state.selected().is_some() &&
                   state.ui.feature_list_state.selected().is_some() {
                    Some(Box::new(EditFeatureCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('d')) => {
                if state.ui.component_list_state.selected().is_some() &&
                   state.ui.feature_list_state.selected().is_some() {
                    Some(Box::new(DeleteFeatureCommand))
                } else {
                    None
                }
            },
            (InputMode::Normal, DialogMode::None, KeyCode::Char('M')) => {
                if state.ui.component_list_state.selected().is_some() {
                    Some(Box::new(ShowComponentMatesCommand))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

pub struct ShowComponentMatesCommand;
impl Command for ShowComponentMatesCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(comp_idx) = state.ui.component_list_state.selected() {
            if let Some(component) = state.project.components.get(comp_idx) {
                // Set component-level filter and switch to mates tab
                state.mates.filter = Some(MateFilter::Component(component.name.clone()));
                state.ui.current_screen = ScreenMode::Mates;
                state.ui.mate_list_state.select(None);
            }
        }
        Ok(())
    }
}


pub struct FinishComponentEditingCommand;
impl Command for FinishComponentEditingCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        // Just return to normal mode without saving the component
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct AddComponentCommand;
impl Command for AddComponentCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.dialog_mode = DialogMode::AddComponent;
        state.input.component_inputs.name = Input::default();
        state.input.component_inputs.description = Input::default();
        Ok(())
    }
}

pub struct EditComponentCommand;
impl Command for EditComponentCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.component_list_state.selected() {
            if let Some(component) = state.project.components.get(idx) {
                state.ui.dialog_mode = DialogMode::EditComponent;
                state.input.component_inputs.name = Input::new(component.name.clone());
                state.input.component_inputs.description = Input::new(
                    component.description.as_ref().unwrap_or(&String::new()).to_string()
                );
            }
        }
        Ok(())
    }
}

pub struct EditComponentNameCommand;
impl Command for EditComponentNameCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::ComponentName);
        Ok(())
    }
}

pub struct EditComponentDescriptionCommand;
impl Command for EditComponentDescriptionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::ComponentDescription);
        Ok(())
    }
}


pub struct DeleteComponentCommand;
impl Command for DeleteComponentCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(idx) = state.ui.component_list_state.selected() {
            // Remove from components list
            state.project.components.remove(idx);

            // Update selection
            if state.project.components.is_empty() {
                state.ui.component_list_state.select(None);
                state.ui.feature_list_state.select(None);
            } else if idx >= state.project.components.len() {
                state.ui.component_list_state.select(Some(state.project.components.len() - 1));
            }

            // Update project file and save everything
            state.file_manager.save_project(
                &state.project.project_file,
                &state.project.components,
                &state.analysis.analyses,
            )?;
        }
        state.ui.dialog_mode = DialogMode::None;
        Ok(())
    }
}

pub struct NextComponentCommand;
impl Command for NextComponentCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if !state.project.components.is_empty() {
            let i = match state.ui.component_list_state.selected() {
                Some(i) => {
                    if i >= state.project.components.len().saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            state.ui.component_list_state.select(Some(i));
        }
        Ok(())
    }
}

pub struct PrevComponentCommand;
impl Command for PrevComponentCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if !state.project.components.is_empty() {
            let i = match state.ui.component_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        state.project.components.len().saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            state.ui.component_list_state.select(Some(i));
        }
        Ok(())
    }
}

pub struct ComponentInputCharCommand(pub char);
impl Command for ComponentInputCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::ComponentName) => {
                let new_value = format!("{}{}", state.input.component_inputs.name.value(), self.0);
                state.input.component_inputs.name = Input::new(new_value);
            },
            InputMode::Editing(EditField::ComponentDescription) => {
                let new_value = format!("{}{}", state.input.component_inputs.description.value(), self.0);
                state.input.component_inputs.description = Input::new(new_value);
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct ComponentDeleteCharCommand;
impl Command for ComponentDeleteCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::ComponentName) => {
                let mut value = state.input.component_inputs.name.value().to_string();
                value.pop();
                state.input.component_inputs.name = Input::new(value);
            },
            InputMode::Editing(EditField::ComponentDescription) => {
                let mut value = state.input.component_inputs.description.value().to_string();
                value.pop();
                state.input.component_inputs.description = Input::new(value);
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct SaveComponentCommand;
impl Command for SaveComponentCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let name = state.input.component_inputs.name.value().trim();
        if name.is_empty() {
            return Err(anyhow!("Component name cannot be empty"));
        }

        match state.ui.dialog_mode {
            DialogMode::AddComponent => {
                let component = Component {
                    name: name.to_string(),
                    description: Some(state.input.component_inputs.description.value().trim().to_string()),
                    features: Vec::new(),
                };
                state.project.components.push(component);
                state.file_manager.save_project(
                    &state.project.project_file,
                    &state.project.components,
                    &state.analysis.analyses,
                )?;
            },
            DialogMode::EditComponent => {
                if let Some(idx) = state.ui.component_list_state.selected() {
                    if let Some(component) = state.project.components.get_mut(idx) {
                        component.name = name.to_string();
                        component.description = Some(state.input.component_inputs.description.value().trim().to_string());
                        state.file_manager.save_project(
                            &state.project.project_file,
                            &state.project.components,
                            &state.analysis.analyses,
                        )?;
                    }
                }
            },
            _ => return Ok(()),
        }

        state.ui.dialog_mode = DialogMode::None;
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}
