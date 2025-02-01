// src/input/project.rs
use super::command::Command;
use anyhow::{Result, anyhow};
use tui_input::Input;
use crate::state::*;
use crate::config::Units;
use crossterm::event::KeyCode;
use crate::state::input_state::{InputMode, EditField};
use std::path::PathBuf;
use rfd::FileDialog;

pub struct ProjectInputHandler;


impl ProjectInputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&self, key: KeyCode, state: &AppState) -> Option<Box<dyn Command>> {
        match (&state.input.mode, &state.ui.dialog_mode, key) {
            (InputMode::Normal, _, KeyCode::Char('n')) => {
                Some(Box::new(EditProjectNameCommand))
            },
            (InputMode::Normal, _, KeyCode::Char('d')) => {
                Some(Box::new(EditProjectDescriptionCommand))
            },
            (InputMode::Normal, _, KeyCode::Char('o')) => {
                Some(Box::new(OpenProjectDirCommand))
            },
            (InputMode::Normal, _, KeyCode::Char('l')) => {
                Some(Box::new(LoadProjectCommand))
            },
            (InputMode::Normal, _, KeyCode::Char('u')) => {
                Some(Box::new(ToggleUnitsCommand))
            },
            (InputMode::Normal, _, KeyCode::Char('s')) => {
                Some(Box::new(SaveProjectCommand))
            },
            (InputMode::Editing(EditField::ProjectName), _, KeyCode::Enter) |
            (InputMode::Editing(EditField::Description), _, KeyCode::Enter) => {
                Some(Box::new(FinishProjectEditingCommand))
            },
            (InputMode::Editing(_), _, KeyCode::Char(c)) => {
                Some(Box::new(ProjectInputCharCommand(c)))
            },
            (InputMode::Editing(_), _, KeyCode::Backspace) => {
                Some(Box::new(ProjectDeleteCharCommand))
            },
            _ => None,
        }
    }
}


pub struct FinishProjectEditingCommand;
impl Command for FinishProjectEditingCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        // Update project with current input values
        state.project.project_file.name = state.input.project_inputs.name.value().to_string();
        state.project.project_file.description = Some(state.input.project_inputs.description.value().to_string());

        // Reset input mode
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct OpenProjectDirCommand;
impl Command for OpenProjectDirCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(dir) = FileDialog::new()
            .set_title("Select Project Directory")
            .pick_folder()
        {
            state.file_manager.set_project_dir(dir.clone())?;
            state.project.project_dir = Some(dir);
        }
        Ok(())
    }
}

pub struct LoadProjectCommand;
impl Command for LoadProjectCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if let Some(file) = FileDialog::new()
            .set_title("Load Project File")
            .add_filter("RON files", &["ron"])
            .pick_file()
        {
            let project_dir = file.parent().unwrap().to_path_buf();
            state.file_manager.set_project_dir(project_dir.clone())?;
            state.project.project_dir = Some(project_dir);

            let (project_file, components, mates_file, analyses) = state.file_manager.load_project(&file)?;

            // Initialize input state with loaded values
            state.input.project_inputs.name = Input::new(project_file.name.clone());
            state.input.project_inputs.description = Input::new(
                project_file.description.clone().unwrap_or_default()
            );

            state.project.project_file = project_file;
            state.project.components = components;
            state.mates.mates = mates_file.mates;

            // Load analyses and results
            state.analysis.analyses.clear();
            state.analysis.latest_results.clear();
            for (analysis, results) in analyses {
                if let Some(res) = results {
                    state.analysis.latest_results.insert(analysis.id.clone(), res);
                }
                state.analysis.analyses.push(analysis);
            }

            // Update dependency graph with loaded mates
            state.mates.update_dependency_graph(&state.project.components);
        }
        Ok(())
    }
}



pub struct EditProjectNameCommand;
impl Command for EditProjectNameCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::ProjectName);
        state.input.project_inputs.name = Input::new(state.project.project_file.name.clone());
        Ok(())
    }
}

pub struct EditProjectDescriptionCommand;
impl Command for EditProjectDescriptionCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.input.mode = InputMode::Editing(EditField::Description);
        state.input.project_inputs.description = Input::new(
            state.project.project_file.description.clone().unwrap_or_default()
        );
        Ok(())
    }
}

pub struct SaveProjectCommand;
impl Command for SaveProjectCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if state.project.project_dir.is_none() {
            return Err(anyhow!("No project directory selected"));
        }

        // Only update if actually editing
        match state.input.mode {
            InputMode::Editing(_) => {
                state.project.project_file.name = state.input.project_inputs.name.value().to_string();
                if !state.input.project_inputs.description.value().is_empty() {
                    state.project.project_file.description = Some(
                        state.input.project_inputs.description.value().to_string()
                    );
                }
            },
            _ => {} // Don't modify values if not editing
        }

        // Save to file system with analyses
        state.file_manager.save_project(
            &state.project.project_file,
            &state.project.components,
            &state.analysis.analyses
        )?;

        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct ToggleUnitsCommand;
impl Command for ToggleUnitsCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.project.project_file.units = match state.project.project_file.units {
            Units::Metric => Units::Imperial,
            Units::Imperial => Units::Metric,
        };
        Ok(())
    }
}

// Project-specific input commands
pub struct ProjectInputCharCommand(pub char);
impl Command for ProjectInputCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::ProjectName) => {
                let new_value = format!("{}{}", state.input.project_inputs.name.value(), self.0);
                state.input.project_inputs.name = Input::new(new_value);
            },
            InputMode::Editing(EditField::Description) => {
                let new_value = format!("{}{}", state.input.project_inputs.description.value(), self.0);
                state.input.project_inputs.description = Input::new(new_value);
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct ProjectDeleteCharCommand;
impl Command for ProjectDeleteCharCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        match state.input.mode {
            InputMode::Editing(EditField::ProjectName) => {
                let mut value = state.input.project_inputs.name.value().to_string();
                value.pop();
                state.input.project_inputs.name = Input::new(value);
            },
            InputMode::Editing(EditField::Description) => {
                let mut value = state.input.project_inputs.description.value().to_string();
                value.pop();
                state.input.project_inputs.description = Input::new(value);
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct SetProjectDirCommand(pub PathBuf);
impl Command for SetProjectDirCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.file_manager.set_project_dir(self.0.clone())?;
        state.project.project_dir = Some(self.0.clone());
        Ok(())
    }
}
