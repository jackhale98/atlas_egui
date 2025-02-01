// src/state/project_state.rs
use crate::config::ProjectFile;
use crate::config::Component;
use std::path::PathBuf;


#[derive(Debug)]
pub struct ProjectState {
    pub project_file: ProjectFile,
    pub components: Vec<Component>,
    pub project_dir: Option<PathBuf>,
}

impl Default for ProjectState {
    fn default() -> Self {
        Self {
            project_file: ProjectFile::default(),
            components: Vec::new(),
            project_dir: None,
        }
    }
}
