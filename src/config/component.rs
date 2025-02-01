// src/config/component.rs
use serde::{Serialize, Deserialize};
use super::Feature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub description: Option<String>,
    pub features: Vec<Feature>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentReference {
    pub path: String,
}

impl ComponentReference {
    // Add this helper method
    pub fn normalized_path(&self) -> String {
        // Always store paths with forward slashes in the RON files
        self.path.replace('\\', "/")
    }
}
