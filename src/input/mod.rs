// src/input/mod.rs
use anyhow::Result;
use crossterm::event::KeyCode;
use crate::state::AppState;
use crate::state::ui_state::{ScreenMode, DialogMode};
use crate::input::command::Command;
pub mod command;
pub mod project;
pub mod component;
pub mod feature;
pub mod mate;

pub struct InputHandler {
    project_handler: project::ProjectInputHandler,
    component_handler: component::ComponentInputHandler,
    feature_handler: feature::FeatureInputHandler,
    mate_handler: mate::MateInputHandler,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            project_handler: project::ProjectInputHandler::new(),
            component_handler: component::ComponentInputHandler::new(),
            feature_handler: feature::FeatureInputHandler::new(),
            mate_handler: mate::MateInputHandler::new(), 
        }
    }

    pub fn handle_input(&self, key: KeyCode, state: &mut AppState) -> Result<()> {
        // First try mode-specific handlers
        if let Some(cmd) = match state.ui.current_screen {
            ScreenMode::Project => self.project_handler.handle_key(key, state),
            ScreenMode::Components => match state.ui.dialog_mode {
                DialogMode::AddFeature | DialogMode::EditFeature =>
                    self.feature_handler.handle_key(key, state),
                DialogMode::AddMate | DialogMode::EditMate =>
                    self.mate_handler.handle_key(key, state),
                _ => {
                    // Try component handler first, then feature handler for unhandled keys
                    self.component_handler.handle_key(key, state)
                        .or_else(|| self.feature_handler.handle_key(key, state))
                },
            },
            ScreenMode::Mates => self.mate_handler.handle_key(key, state),
            _ => None,
        } {
            return cmd.execute(state);
        }

        // Then try global handlers
        if let Some(cmd) = self.handle_global_keys(key) {
            return cmd.execute(state);
        }

        Ok(())
    }

    fn handle_global_keys(&self, key: KeyCode) -> Option<Box<dyn Command>> {
        match key {
            KeyCode::Char('L') => Some(Box::new(command::NextTabCommand)),
            KeyCode::Char('H') => Some(Box::new(command::PrevTabCommand)),
            KeyCode::Esc => Some(Box::new(command::ClearDialogCommand)),
            _ => None,
        }
    }
}
