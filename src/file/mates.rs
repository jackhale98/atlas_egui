// src/file/mates.rs
use serde::{Serialize, Deserialize};
use super::FileHandler;
use std::path::Path;
use anyhow::{Result, Context};
use std::fs;
use crate::config::Mate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatesFile {
    pub version: String,
    pub mates: Vec<Mate>,
}

impl MatesFile {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            mates: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct MatesFileHandler;

impl MatesFileHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FileHandler<MatesFile> for MatesFileHandler {
    fn load(&self, path: &Path) -> Result<MatesFile> {
        if !path.exists() {
            return Ok(MatesFile::new());
        }
        let content = fs::read_to_string(path)?;
        ron::from_str(&content).context("Failed to parse mates file")
    }

    fn save(&self, data: &MatesFile, path: &Path) -> Result<()> {
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
