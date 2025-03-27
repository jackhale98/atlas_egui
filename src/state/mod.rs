// src/state/mod.rs
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

use crate::config::{ProjectFile, Component};
use crate::config::mate::Mate;
use crate::analysis::{StackupAnalysis, AnalysisResults};
use crate::file::FileManager;
use crate::analysis::stackup::{AnalysisMethod, MonteCarloSettings};
use crate::state::mate_state::MateState;

pub mod mate_state;

// Core dialog tracking
#[derive(Debug, Clone)]
pub enum DialogState {
    None,
    NewComponent {
        name: String,
        revision: String,
        description: String,
    },
    EditComponent {
        index: usize,
        name: String,
        revision: String,
        description: String,
    },
    NewFeature {
        component_index: usize,
        name: String,
        value: f64,
        plus_tolerance: f64,
        minus_tolerance: f64,
    },
    EditFeature {
        component_index: usize,
        feature_index: usize,
        name: String,
        value: f64,
        plus_tolerance: f64,
        minus_tolerance: f64,
    },
    NewMate {
        component_a: String,
        feature_a: String,
        component_b: String,
        feature_b: String,
    },
    EditMate {
        index: usize,
        component_a: String,
        feature_a: String,
        component_b: String,
        feature_b: String,
    },
    NewAnalysis {
        name: String,
        methods: Vec<AnalysisMethod>,
        monte_carlo_settings: MonteCarloSettings,
    },
    EditAnalysis {
        index: usize,
        name: String,
        methods: Vec<AnalysisMethod>,
        monte_carlo_settings: MonteCarloSettings,
    },
    NewContribution {
        analysis_index: usize,
        component_id: String,
        feature_id: String,
        direction: f64,
        half_count: bool,
    },
    EditContribution {
        analysis_index: usize,
        contribution_index: Option<usize>,
        component_id: String,
        feature_id: String,
        direction: f64,
        half_count: bool,
    },
}

// Screen/tab tracking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Project,
    Components,
    Mates,
    DependencyMatrix,
    Analysis,
    GitControl,
}

// Analysis view tabs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisTab {
    Details,
    Results,
    Visualization,
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
    
    // Minimal UI state
    pub current_screen: Screen,
    pub current_dialog: DialogState,
    pub analysis_tab: AnalysisTab,
    pub error_message: Option<String>,
    
    // File management
    pub file_manager: FileManager,

    pub selected_component: Option<usize>,
    pub selected_feature: Option<usize>, 
    pub selected_mate: Option<usize>,
    pub selected_analysis: Option<usize>,

    pub mate_state: mate_state::MateState,

    pub dependency_map_cache: Option<HashMap<((String, String), (String, String)), usize>>,
    pub dependency_map_cache_dirty: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            project_file: ProjectFile::default(),
            project_dir: None,
            components: Vec::new(),
            mates: Vec::new(),
            mate_graph: petgraph::Graph::new(),
            mate_state: mate_state::MateState::default(),
            analyses: Vec::new(),
            latest_results: HashMap::new(),
            current_screen: Screen::Project,
            current_dialog: DialogState::None,
            analysis_tab: AnalysisTab::Details,
            error_message: None,
            file_manager: FileManager::new(),
            selected_component: None,
            selected_feature: None,
            selected_mate: None, 
            selected_analysis: None,

            dependency_map_cache: None,
            dependency_map_cache_dirty: true,
        }
    }

    pub fn save_project(&mut self) -> Result<()> {
        if self.project_dir.is_none() {
            return Err(anyhow::anyhow!("No project directory selected"));
        }

        self.file_manager.save_project(
            &self.project_file,
            &self.components,
            &self.analyses
        )?;

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
    pub fn update_mate_state(&mut self) {
        self.mate_state.mates = self.mates.clone();
        self.mate_state.update_dependency_graph(&self.components);
    }
}

