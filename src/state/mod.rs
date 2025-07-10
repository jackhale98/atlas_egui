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

#[derive(Debug, Clone)]
pub struct ProjectSnapshot {
    pub components: Vec<Component>,
    pub mates: Vec<Mate>,
    pub analyses: Vec<StackupAnalysis>,
    pub description: String,
}

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

    // Undo/Redo system
    pub undo_stack: Vec<ProjectSnapshot>,
    pub redo_stack: Vec<ProjectSnapshot>,
    pub max_undo_history: usize,

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

            // Undo/Redo system
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_history: 50,

            // Git control state removed for CLI
        }
    }

    pub fn save_project(&mut self) -> Result<()> {
        if self.project_dir.is_none() {
            return Err(anyhow::anyhow!("No project directory selected"));
        }

        self.file_manager.save_project(
            &self.project_file,
            &self.components,
            &self.mates
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

    /// Create a snapshot of the current project state
    pub fn create_snapshot(&self, description: String) -> ProjectSnapshot {
        ProjectSnapshot {
            components: self.components.clone(),
            mates: self.mates.clone(),
            analyses: self.analyses.clone(),
            description,
        }
    }

    /// Save current state to undo stack before making changes
    pub fn save_to_undo_stack(&mut self, description: String) {
        let snapshot = self.create_snapshot(description);
        self.undo_stack.push(snapshot);

        // Limit undo history size
        if self.undo_stack.len() > self.max_undo_history {
            self.undo_stack.remove(0);
        }

        // Clear redo stack when new action is performed
        self.redo_stack.clear();
    }

    /// Undo the last action
    pub fn undo(&mut self) -> Result<Option<String>> {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current_snapshot = self.create_snapshot("Redo point".to_string());
            self.redo_stack.push(current_snapshot);

            // Restore from snapshot
            self.components = snapshot.components;
            self.mates = snapshot.mates;
            self.analyses = snapshot.analyses;
            
            self.update_dependencies();
            
            // Auto-save after undo
            if let Err(e) = self.save_project() {
                eprintln!("⚠️  Warning: Failed to save project after undo: {}", e);
            }

            Ok(Some(snapshot.description))
        } else {
            Ok(None)
        }
    }

    /// Redo the last undone action
    pub fn redo(&mut self) -> Result<Option<String>> {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current_snapshot = self.create_snapshot("Undo point".to_string());
            self.undo_stack.push(current_snapshot);

            // Restore from snapshot
            self.components = snapshot.components;
            self.mates = snapshot.mates;
            self.analyses = snapshot.analyses;
            
            self.update_dependencies();
            
            // Auto-save after redo
            if let Err(e) = self.save_project() {
                eprintln!("⚠️  Warning: Failed to save project after redo: {}", e);
            }

            Ok(Some(snapshot.description))
        } else {
            Ok(None)
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get description of the last action that can be undone
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|s| s.description.as_str())
    }

    /// Get description of the last action that can be redone
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|s| s.description.as_str())
    }
}

