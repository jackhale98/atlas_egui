// src/visualization/mod.rs
use anyhow::Result;
use std::path::Path;

pub mod ascii;
pub mod svg;
pub mod plots;

use crate::state::AppState;
use crate::analysis::AnalysisResults;

/// Trait for different visualization backends
pub trait Visualizer {
    fn render_dependency_matrix(&self, state: &AppState) -> Result<String>;
    fn render_analysis_results(&self, results: &AnalysisResults, analysis_name: &str) -> Result<String>;
    fn render_histogram(&self, data: &[f64], title: &str) -> Result<String>;
}

/// ASCII text-based visualizer
pub struct AsciiVisualizer;

/// SVG-based visualizer
pub struct SvgVisualizer;

impl Visualizer for AsciiVisualizer {
    fn render_dependency_matrix(&self, state: &AppState) -> Result<String> {
        ascii::render_dependency_matrix(state)
    }

    fn render_analysis_results(&self, results: &AnalysisResults, analysis_name: &str) -> Result<String> {
        ascii::render_analysis_results(results, analysis_name)
    }

    fn render_histogram(&self, data: &[f64], title: &str) -> Result<String> {
        ascii::render_histogram(data, title)
    }
}

impl Visualizer for SvgVisualizer {
    fn render_dependency_matrix(&self, state: &AppState) -> Result<String> {
        svg::render_dependency_matrix(state)
    }

    fn render_analysis_results(&self, results: &AnalysisResults, analysis_name: &str) -> Result<String> {
        svg::render_analysis_results(results, analysis_name)
    }

    fn render_histogram(&self, data: &[f64], title: &str) -> Result<String> {
        svg::render_histogram(data, title)
    }
}

/// Export visualization to file
pub fn export_to_file<P: AsRef<Path>>(content: &str, path: P) -> Result<()> {
    use std::fs;
    fs::write(path, content)?;
    Ok(())
}

/// Get appropriate file extension for visualization type
pub fn get_file_extension(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "svg" => "svg",
        "ascii" | "text" => "txt",
        _ => "txt",
    }
}