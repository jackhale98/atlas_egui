// src/state/mod.rs
use crate::file::FileManager;

pub mod ui_state;
pub mod project_state;
pub mod input_state;
pub mod mate_state;

use ui_state::*;
use project_state::*;
use input_state::*;
use mate_state::*;

pub mod analysis_state;
pub use analysis_state::AnalysisState;

#[derive(Debug)]
pub struct AppState {
    pub ui: UiState,
    pub project: ProjectState,
    pub input: InputState,
    pub mates: MateState,
    pub analysis: AnalysisState,
    pub file_manager: FileManager,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ui: UiState::default(),
            project: ProjectState::default(),
            input: InputState::default(),
            mates: MateState::default(),
            analysis: AnalysisState::default(),
            file_manager: FileManager::new(),
        }
    }
}
