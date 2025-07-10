// src/analysis/cli_support.rs
// CLI-specific analysis support functions and types

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rand::prelude::*;
use rand_distr::{Normal, Uniform};
use indicatif::ProgressBar;

use crate::state::AppState;
// All analysis types are now defined in this module

// Analysis method types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AnalysisMethod {
    WorstCase,
    RootSumSquare,
    MonteCarlo,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DistributionType {
    Normal,
    Uniform,
    Triangular,
    LogNormal,
}

impl Default for DistributionType {
    fn default() -> Self {
        DistributionType::Normal
    }
}

// CLI-compatible types that match what our prompts expect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupAnalysis {
    pub id: String,
    pub name: String,
    pub methods: Vec<AnalysisMethod>,
    pub monte_carlo_settings: MonteCarloSettings,
    pub contributions: Vec<Contribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloSettings {
    pub num_samples: u32,
    pub seed: Option<u64>,
}

impl Default for MonteCarloSettings {
    fn default() -> Self {
        Self {
            num_samples: 10000,
            seed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub component_id: String,
    pub feature_id: String,
    pub direction: f64,
    pub half_count: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub cp: Option<f64>,
    pub cpk: Option<f64>,
    pub percentiles: Vec<(f64, f64)>,
    pub histogram_data: Vec<f64>,
}

/// Run Monte Carlo analysis
pub fn run_monte_carlo_analysis(
    analysis: &StackupAnalysis,
    state: &AppState,
    progress_bar: Option<&ProgressBar>,
) -> Result<AnalysisResults> {
    let num_samples = analysis.monte_carlo_settings.num_samples as usize;
    let mut rng = if let Some(seed) = analysis.monte_carlo_settings.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    };

    let mut samples = Vec::with_capacity(num_samples);

    // Find features for each contribution
    let mut contribution_features = Vec::new();
    for contribution in &analysis.contributions {
        let component = state.components.iter()
            .find(|c| c.name == contribution.component_id)
            .ok_or_else(|| anyhow::anyhow!("Component '{}' not found", contribution.component_id))?;
        
        let feature = component.features.iter()
            .find(|f| f.name == contribution.feature_id)
            .ok_or_else(|| anyhow::anyhow!("Feature '{}' not found in component '{}'", contribution.feature_id, contribution.component_id))?;
        
        contribution_features.push((contribution, feature));
    }

    // Generate samples
    for i in 0..num_samples {
        if let Some(pb) = progress_bar {
            if i % 100 == 0 {
                pb.set_position(i as u64);
            }
        }

        let mut stackup_value = 0.0;

        for (contribution, feature) in &contribution_features {
            let feature_value = sample_feature_value(feature, &mut rng)?;
            let contribution_value = feature_value * contribution.direction;
            
            let final_contribution = if contribution.half_count {
                contribution_value * 0.5
            } else {
                contribution_value
            };

            stackup_value += final_contribution;
        }

        samples.push(stackup_value);
    }

    if let Some(pb) = progress_bar {
        pb.set_position(num_samples as u64);
    }

    // Calculate statistics
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (samples.len() - 1) as f64;
    let std_dev = variance.sqrt();
    
    let mut sorted_samples = samples.clone();
    sorted_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let min = sorted_samples[0];
    let max = sorted_samples[sorted_samples.len() - 1];

    // Calculate percentiles
    let mut percentiles = Vec::new();
    let percentile_values = vec![0.1, 1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0, 99.9];
    
    for p in percentile_values {
        let index = ((p / 100.0) * (sorted_samples.len() - 1) as f64).round() as usize;
        let index = index.min(sorted_samples.len() - 1);
        percentiles.push((p, sorted_samples[index]));
    }

    Ok(AnalysisResults {
        mean,
        std_dev,
        min,
        max,
        cp: None, // Would need specification limits
        cpk: None, // Would need specification limits
        percentiles,
        histogram_data: samples,
    })
}

/// Sample a value from a feature based on its distribution
fn sample_feature_value(feature: &crate::config::Feature, rng: &mut StdRng) -> Result<f64> {
    let distribution = feature.distribution.unwrap_or(DistributionType::Normal);
    
    match distribution {
        DistributionType::Normal => {
            let mean = feature.dimension.value;
            let total_tolerance = feature.dimension.plus_tolerance + feature.dimension.minus_tolerance;
            let std_dev = total_tolerance / 6.0; // 6-sigma approach
            
            let normal = Normal::new(mean, std_dev)
                .map_err(|e| anyhow::anyhow!("Invalid normal distribution parameters: {}", e))?;
            Ok(normal.sample(rng))
        },
        DistributionType::Uniform => {
            let min = feature.dimension.value - feature.dimension.minus_tolerance;
            let max = feature.dimension.value + feature.dimension.plus_tolerance;
            
            let uniform = Uniform::new(min, max);
            Ok(uniform.sample(rng))
        },
        DistributionType::Triangular => {
            // Simplified triangular distribution using uniform with mode bias
            let min = feature.dimension.value - feature.dimension.minus_tolerance;
            let max = feature.dimension.value + feature.dimension.plus_tolerance;
            let mode = feature.dimension.value;
            
            // Use simple triangular approximation
            let u1: f64 = rng.gen();
            let u2: f64 = rng.gen();
            
            let fc = (mode - min) / (max - min);
            
            if u1 <= fc {
                Ok(min + (u1 * (max - min) * (mode - min)).sqrt())
            } else {
                Ok(max - ((1.0 - u1) * (max - min) * (max - mode)).sqrt())
            }
        },
        DistributionType::LogNormal => {
            // For lognormal, we need to be careful about parameters
            let mean = feature.dimension.value.ln();
            let total_tolerance = feature.dimension.plus_tolerance + feature.dimension.minus_tolerance;
            let std_dev = (total_tolerance / feature.dimension.value / 6.0).ln().abs();
            
            let normal = Normal::new(mean, std_dev)
                .map_err(|e| anyhow::anyhow!("Invalid lognormal distribution parameters: {}", e))?;
            Ok(normal.sample(rng).exp())
        },
    }
}