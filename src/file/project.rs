// src/file/project.rs
use super::FileHandler;
use crate::config::ProjectFile;
use std::path::Path;
use std::fs;
use anyhow::{Result, Context};

#[derive(Debug)]
pub struct ProjectFileHandler;

impl ProjectFileHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FileHandler<ProjectFile> for ProjectFileHandler {
    fn load(&self, path: &Path) -> Result<ProjectFile> {
        let content = fs::read_to_string(path)?;
        ron::from_str(&content).context("Failed to parse project file")
    }

    fn save(&self, data: &ProjectFile, path: &Path) -> Result<()> {
        let content = ron::ser::to_string_pretty(
            data,
            ron::ser::PrettyConfig::new()
                .new_line("\n".to_string())
                .depth_limit(4)
                .separate_tuple_members(true)
        )?;
        fs::write(path, content)?;
        Ok(())
    }
}
