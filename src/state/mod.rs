// src/state/mod.rs
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

use crate::config::{ProjectFile, Component};
use crate::config::mate::Mate;
use crate::analysis::{StackupAnalysis, AnalysisResults};
use crate::file::FileManager;
// CLI doesn't need complex state management

// CLI doesn't need dialog state or screen tracking - removed for simplicity

// Core application state
#[derive(Debug)]
pub struct AppState {
    // Project data
    pub project_file: ProjectFile,
    pub project_dir: Option<PathBuf>,
    pub components: Vec<Component>,
    
    // Dependency & mate tracking 
    pub mates: Vec<Mate>,
    pub mate_graph: petgraph::Graph<String, String>,
    
    // Analysis data
    pub analyses: Vec<StackupAnalysis>,
    pub latest_results: HashMap<String, AnalysisResults>,
    
    // CLI doesn't need UI state - removed
    
    // File management
    pub file_manager: FileManager,

    pub selected_component: Option<usize>,
    pub selected_feature: Option<usize>, 
    pub selected_mate: Option<usize>,
    pub selected_analysis: Option<usize>,

    // Simplified state for CLI

    pub dependency_map_cache: Option<HashMap<((String, String), (String, String)), usize>>,
    pub dependency_map_cache_dirty: bool,

    // Git control state removed for CLI
}

impl AppState {
    pub fn new() -> Self {
        Self {
            project_file: ProjectFile::default(),
            project_dir: None,
            components: Vec::new(),
            mates: Vec::new(),
            mate_graph: petgraph::Graph::new(),
            // Simplified state for CLI
            analyses: Vec::new(),
            latest_results: HashMap::new(),
            // CLI doesn't need UI state
            file_manager: FileManager::new(),
            selected_component: None,
            selected_feature: None,
            selected_mate: None, 
            selected_analysis: None,

            dependency_map_cache: None,
            dependency_map_cache_dirty: true,

            // Git control state removed for CLI
        }
    }

    pub fn save_project(&mut self) -> Result<()> {
        if self.project_dir.is_none() {
            return Err(anyhow::anyhow!("No project directory selected"));
        }

        self.file_manager.save_project(
            &self.project_file,
            &self.components
        )?;
        
        // Mark the dependency cache as dirty after saving
        self.mark_dependency_cache_dirty();

        Ok(())
    }

    pub fn update_mate_graph(&mut self) {
        self.mate_graph = petgraph::Graph::new();
        let mut nodes = HashMap::new();

        // Create nodes for all features
        for component in &self.components {
            for feature in &component.features {
                let node_id = self.mate_graph.add_node(feature.name.clone());
                nodes.insert(
                    (component.name.clone(), feature.name.clone()),
                    node_id
                );
            }
        }

        // Add edges for mates
        for mate in &self.mates {
            if let (Some(&node_a), Some(&node_b)) = (
                nodes.get(&(mate.component_a.clone(), mate.feature_a.clone())),
                nodes.get(&(mate.component_b.clone(), mate.feature_b.clone()))
            ) {
                self.mate_graph.add_edge(
                    node_a,
                    node_b,
                    format!("{:?}", mate.fit_type)
                );
            }
        }
    }
    // Simplified dependency management for CLI
    pub fn mark_dependency_cache_dirty(&mut self) {
        self.dependency_map_cache_dirty = true;
    }
    pub fn update_dependencies(&mut self) {
        self.update_mate_graph();
        self.mark_dependency_cache_dirty();
    }
}

