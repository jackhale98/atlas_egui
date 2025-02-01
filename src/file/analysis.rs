// src/file/analysis.rs

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use chrono::prelude::*;
use anyhow::anyhow;
use serde::{Serialize, Deserialize};
use csv::Writer;
use crate::analysis::{
    AnalysisMethod,
    StackupAnalysis,
    AnalysisResults,
    MonteCarloResult
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub version: String,
    pub analysis_id: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub raw_data_files: Vec<RawDataFile>,
    pub results_files: Vec<ResultsFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDataFile {
    pub path: String,
    pub analysis_type: AnalysisMethod,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsFile {
    pub path: String,
    pub timestamp: DateTime<Utc>,
    pub analysis_methods: Vec<AnalysisMethod>,
}

#[derive(Debug)]
pub struct AnalysisFileManager {
    base_path: PathBuf,
}

impl AnalysisFileManager {
    pub fn new(project_path: &Path) -> Self {
        Self {
            base_path: project_path.join("analyses"),
        }
    }

    pub fn create_analysis_directories(&self, analysis_id: &str) -> Result<()> {
        let analysis_dir = self.base_path.join("stackups").join(analysis_id);
        fs::create_dir_all(&analysis_dir)?;
        fs::create_dir_all(analysis_dir.join("raw_data"))?;
        fs::create_dir_all(analysis_dir.join("results"))?;
        Ok(())
    }

    pub fn save_analysis(&self, analysis: &StackupAnalysis, results: &AnalysisResults) -> Result<()> {
        // Create required directories
        self.create_analysis_directories(&analysis.id)?;
        
        let base_dir = self.base_path.join("stackups").join(&analysis.id);
        let timestamp = Utc::now();
        let timestamp_str = timestamp.format("%Y%m%d_%H%M%S").to_string();

        // Save the analysis definition
        let analysis_path = base_dir.join("analysis.ron");
        let analysis_content = ron::ser::to_string_pretty(
            analysis,
            ron::ser::PrettyConfig::new()
                .depth_limit(4)
                .separate_tuple_members(true)
        )?;
        fs::write(&analysis_path, analysis_content)?;

        // Save raw data if Monte Carlo was run
        let mut raw_data_files = Vec::new();
        if let Some(mc_results) = &results.monte_carlo {
            let raw_data_path = base_dir
                .join("raw_data")
                .join(format!("monte_carlo_{}.csv", timestamp_str));
            
            self.save_monte_carlo_raw_data(&raw_data_path, mc_results)?;
            
            raw_data_files.push(RawDataFile {
                path: raw_data_path.strip_prefix(&self.base_path)?.to_string_lossy().into_owned(),
                analysis_type: AnalysisMethod::MonteCarlo,
                timestamp,
            });
        }

        // Save analysis results
        let results_path = base_dir
            .join("results")
            .join(format!("results_{}.ron", timestamp_str));
        
        let results_content = ron::ser::to_string_pretty(
            results,
            ron::ser::PrettyConfig::new()
                .depth_limit(4)
                .separate_tuple_members(true)
        )?;
        fs::write(&results_path, results_content)?;

        // Update metadata
        let mut metadata = if let Ok(existing) = self.load_metadata(&analysis.id) {
            existing
        } else {
            AnalysisMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                analysis_id: analysis.id.clone(),
                name: analysis.name.clone(),
                created: timestamp,
                last_run: None,
                raw_data_files: Vec::new(),
                results_files: Vec::new(),
            }
        };

        metadata.last_run = Some(timestamp);
        metadata.raw_data_files.extend(raw_data_files);
        metadata.results_files.push(ResultsFile {
            path: results_path.strip_prefix(&self.base_path)?.to_string_lossy().into_owned(),
            timestamp,
            analysis_methods: analysis.methods.clone(),
        });

        // Save metadata
        self.save_metadata(&analysis.id, &metadata)?;

        Ok(())
    }

    fn save_monte_carlo_raw_data<P: AsRef<Path>>(
        &self,
        path: P,
        results: &MonteCarloResult
    ) -> Result<()> {
        let mut writer = Writer::from_path(path)?;
    
        // Write header
        let mut headers = vec!["stackup_result".to_string()];
        for sensitivity in &results.sensitivity {
            headers.push(format!("{}_{}", sensitivity.component_id, sensitivity.feature_id));
        }
        writer.write_record(&headers)?;
    
        // Only proceed if we have samples in the first sensitivity result
        if let Some(first_samples) = results.sensitivity.first()
            .and_then(|s| s.samples.as_ref()) 
        {
            // Write data row by row
            for i in 0..first_samples.len() {
                let mut record = Vec::new();
                // Get stackup result for this row
                record.push(first_samples[i].1.to_string());
                
                // Get individual feature values from each sensitivity
                for sensitivity in &results.sensitivity {
                    if let Some(samples) = &sensitivity.samples {
                        if let Some(sample) = samples.get(i) {
                            record.push(sample.0.to_string());
                        }
                    }
                }
                writer.write_record(&record)?;
            }
        }
    
        writer.flush()?;
        Ok(())
    }

    pub fn load_metadata(&self, analysis_id: &str) -> Result<AnalysisMetadata> {
        let metadata_path = self.base_path
                                .join("stackups")
                                .join(analysis_id)
                                .join("metadata.ron");

        if !metadata_path.exists() {
            return Err(anyhow!("Metadata file not found: {}", metadata_path.display()));
        }

        let content = fs::read_to_string(&metadata_path)
            .with_context(|| format!("Failed to read metadata file: {}", metadata_path.display()))?;

        ron::from_str(&content)
            .with_context(|| format!("Failed to parse metadata file: {}", metadata_path.display()))
    }

    fn save_metadata(&self, analysis_id: &str, metadata: &AnalysisMetadata) -> Result<()> {
        let metadata_path = self.base_path
            .join("stackups")
            .join(analysis_id)
            .join("metadata.ron");
        
        let content = ron::ser::to_string_pretty(
            metadata,
            ron::ser::PrettyConfig::new()
                .depth_limit(4)
                .separate_tuple_members(true)
        )?;
        fs::write(metadata_path, content)?;
        Ok(())
    }

    pub fn load_analysis(&self, analysis_id: &str) -> Result<(StackupAnalysis, Option<AnalysisResults>)> {
        let base_dir = self.base_path.join("stackups").join(analysis_id);

        // Check if base directory exists
        if !base_dir.exists() {
            return Err(anyhow!("Analysis directory not found: {}", base_dir.display()));
        }

        // Load analysis definition
        let analysis_path = base_dir.join("analysis.ron");
        let analysis: StackupAnalysis = ron::from_str(&fs::read_to_string(&analysis_path)?)
            .with_context(|| format!("Failed to parse analysis file: {}", analysis_path.display()))?;

        // Try to load metadata and results, but don't fail if they don't exist
        let latest_results = match self.load_metadata(analysis_id) {
            Ok(metadata) => {
                if let Some(results_file) = metadata.results_files.last() {
                    let results_path = self.base_path.join(&results_file.path);
                    match fs::read_to_string(&results_path) {
                        Ok(content) => {
                            match ron::from_str(&content) {
                                Ok(results) => Some(results),
                                Err(e) => {
                                    eprintln!("Warning: Failed to parse results file {}: {}", results_path.display(), e);
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Warning: Failed to read results file {}: {}", results_path.display(), e);
                            None
                        }
                    }
                } else {
                    None
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to load metadata for analysis {}: {}", analysis_id, e);
                None
            }
        };

        Ok((analysis, latest_results))
    }
}

