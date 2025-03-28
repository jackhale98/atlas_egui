// src/analysis/mod.rs
pub mod stackup;

// Re-export commonly used types
pub use stackup::{
    AnalysisMethod,
    StackupAnalysis,
    AnalysisResults,
    MonteCarloResult,
    StackupContribution,
};