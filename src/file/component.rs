// src/file/component.rs
use super::FileHandler;
use crate::config::Component;
use std::path::Path;
use std::fs;
use anyhow::{Result, Context};

#[derive(Debug)]
pub struct ComponentFileHandler;

impl ComponentFileHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FileHandler<Component> for ComponentFileHandler {
    fn load(&self, path: &Path) -> Result<Component> {
        let content = fs::read_to_string(path)?;
        ron::from_str(&content).context("Failed to parse component file")
    }

    fn save(&self, data: &Component, path: &Path) -> Result<()> {
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
