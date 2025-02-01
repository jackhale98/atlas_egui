// src/config/project.rs
use serde::{Serialize, Deserialize};
use super::ComponentReference;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub units: Units,
    pub component_references: Vec<ComponentReference>,
    pub analyses: Vec<AnalysisReference>,  
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Units {
    Metric,
    Imperial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReference {
    pub path: String,
    pub analysis_type: String,
}

impl AnalysisReference {
    // Add this helper method
    pub fn normalized_path(&self) -> String {
        // Always store paths with forward slashes in the RON files
        self.path.replace('\\', "/")
    }
}

impl Default for ProjectFile {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            version: "1.0.0".to_string(),
            units: Units::Metric,
            component_references: Vec::new(),
            analyses: Vec::new(),
        }
    }
}

