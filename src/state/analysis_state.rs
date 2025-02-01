// src/state/analysis_state.rs
use std::collections::HashMap;
use crate::analysis::{StackupAnalysis, AnalysisResults};

#[derive(Debug)]
pub struct AnalysisState {
    pub analyses: Vec<StackupAnalysis>,
    pub latest_results: HashMap<String, AnalysisResults>,
}

impl Default for AnalysisState {
    fn default() -> Self {
        Self {
            analyses: Vec::new(),
            latest_results: HashMap::new(),
        }
    }
}