// src/input/command.rs
use anyhow::{Result, anyhow};
use crate::state::*;
use crate::state::ui_state::{DialogMode, ScreenMode};
use crate::state::input_state::InputMode;
use std::path::PathBuf;
use crate::config::FeatureType;

pub trait Command {
    fn execute(&self, state: &mut AppState) -> Result<()>;
}

// Only keep basic global commands here
pub struct NextTabCommand;
impl Command for NextTabCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.current_screen = match state.ui.current_screen {
            ScreenMode::Project => ScreenMode::Components,
            ScreenMode::Components => ScreenMode::Mates,
            ScreenMode::Mates => ScreenMode::DependencyMatrix,
            ScreenMode::DependencyMatrix => ScreenMode::Analysis,
            ScreenMode::Analysis => ScreenMode::Project,
        };
        Ok(())
    }
}

pub struct PrevTabCommand;
impl Command for PrevTabCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.current_screen = match state.ui.current_screen {
            ScreenMode::Project => ScreenMode::Analysis,
            ScreenMode::Components => ScreenMode::Project,
            ScreenMode::Mates => ScreenMode::Components,
            ScreenMode::DependencyMatrix => ScreenMode::Mates,
            ScreenMode::Analysis => ScreenMode::DependencyMatrix,
        };
        Ok(())
    }
}

pub struct ClearDialogCommand;
impl Command for ClearDialogCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        state.ui.dialog_mode = DialogMode::None;
        state.ui.dialog_error = None;
        state.input.mode = InputMode::Normal;
        Ok(())
    }
}

pub struct OpenProjectCommand(pub PathBuf);
impl Command for OpenProjectCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        let (project_file, components, mates_file, analyses) = state.file_manager.load_project(&self.0)?;
        
        state.project.project_file = project_file;
        state.project.components = components;
        state.project.project_dir = Some(self.0.parent().unwrap().to_path_buf());
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
        Ok(())
    }
}

pub struct SaveProjectCommand;
impl Command for SaveProjectCommand {
    fn execute(&self, state: &mut AppState) -> Result<()> {
        if state.project.project_dir.is_none() {
            return Err(anyhow!("No project directory selected"));
        }

        state.file_manager.save_project(
            &state.project.project_file,
            &state.project.components,
            &state.analysis.analyses,
        )?;

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
        Ok(())
    }
}
