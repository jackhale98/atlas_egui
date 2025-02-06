// src/analysis/stackup.rs

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rand::prelude::*;
use rand_distr::{Distribution, Normal as RandNormal, Uniform, LogNormal};
use crate::config::Component;
use crate::config::Feature;
use uuid::Uuid;
use chrono;
use statrs::distribution::{Normal as StatsNormal, ContinuousCDF};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AnalysisMethod {
    WorstCase,
    Rss,
    MonteCarlo,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DistributionType {
    Normal,
    Uniform,
    Triangular,
    LogNormal,
}

// Add this impl after the DistributionType enum definition
impl Default for DistributionType {
    fn default() -> Self {
        DistributionType::Normal // Most common distribution type as default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionParams {
    pub dist_type: DistributionType,
    pub mean: f64,
    pub std_dev: f64,           // Used for Normal, LogNormal
    pub min: f64,               // Used for Uniform, Triangular
    pub max: f64,               // Used for Uniform, Triangular
    pub mode: Option<f64>,      // Used for Triangular
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCapability {
    pub upper_spec: Option<f64>,
    pub lower_spec: Option<f64>,
    pub cp: Option<f64>,
    pub cpk: Option<f64>,
    pub ppm_above: Option<f64>,
    pub ppm_below: Option<f64>,
    pub pph_above: Option<f64>,
    pub pph_below: Option<f64>,
}

impl DistributionParams {
    pub fn new_normal(mean: f64, std_dev: f64) -> Self {
        Self {
            dist_type: DistributionType::Normal,
            mean,
            std_dev,
            min: 0.0,
            max: 0.0,
            mode: None,
        }
    }

    pub fn new_uniform(min: f64, max: f64) -> Self {
        Self {
            dist_type: DistributionType::Uniform,
            mean: 0.0,
            std_dev: 0.0,
            min,
            max,
            mode: None,
        }
    }

    pub fn new_triangular(min: f64, max: f64, mode: f64) -> Self {
        Self {
            dist_type: DistributionType::Triangular,
            mean: 0.0,
            std_dev: 0.0,
            min,
            max,
            mode: Some(mode),
        }
    }

    pub fn new_lognormal(mean: f64, std_dev: f64) -> Self {
        Self {
            dist_type: DistributionType::LogNormal,
            mean,
            std_dev,
            min: 0.0,
            max: 0.0,
            mode: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupContribution {
    pub component_id: String,
    pub feature_id: String,
    pub direction: f64,         // 1.0 or -1.0
    pub half_count: bool,       // For cases where only half the tolerance applies
    pub distribution: Option<DistributionParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupAnalysis {
    pub id: String,
    pub name: String,
    pub contributions: Vec<StackupContribution>,
    pub methods: Vec<AnalysisMethod>,
    pub monte_carlo_settings: Option<MonteCarloSettings>,
    pub upper_spec_limit: Option<f64>, 
    pub lower_spec_limit: Option<f64>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloSettings {
    pub iterations: usize,
    pub confidence: f64,
    pub seed: Option<u64>,
}
impl Default for MonteCarloSettings {
    fn default() -> Self {
        Self {
            iterations: 10000,
            confidence: 0.9995,
            seed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub analysis_id: String,
    pub timestamp: String,
    pub nominal: f64,
    pub worst_case: Option<WorstCaseResult>,
    pub rss: Option<RssResult>,
    pub monte_carlo: Option<MonteCarloResult>,
    pub process_capability: Option<ProcessCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorSensitivity {
    pub component_id: String,
    pub feature_id: String,
    pub contribution_percent: f64,    // Percentage contribution to total variation
    pub nominal_value: f64,           // Original nominal value
    pub variation_range: (f64, f64),  // Min/max or statistical range
    pub correlation: Option<f64>,     // Only used for Monte Carlo
    pub samples: Option<Vec<(f64, f64)>>, // Optional (feature_value, stackup_result) pairs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorstCaseResult {
    pub min: f64,
    pub max: f64,
    pub sensitivity: Vec<ContributorSensitivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssResult {
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub sensitivity: Vec<ContributorSensitivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub confidence_intervals: Vec<ConfidenceInterval>,
    pub histogram: Vec<(f64, usize)>,
    pub sensitivity: Vec<ContributorSensitivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub confidence_level: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl StackupAnalysis {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            contributions: Vec::new(),
            methods: vec![AnalysisMethod::WorstCase],
            monte_carlo_settings: None,
            upper_spec_limit: None,  
            lower_spec_limit: None,  
        }
    }

    pub fn add_contribution(
        &mut self,
        component_id: String,
        feature_id: String,
        direction: f64,
        half_count: bool,
        distribution: Option<DistributionParams>
    ) {
        self.contributions.push(StackupContribution {
            component_id,
            feature_id,
            direction,
            half_count,
            distribution,
        });
    }

    pub fn calculate_nominal(&self, components: &[Component]) -> f64 {
        self.contributions.iter().fold(0.0, |acc, contrib| {
            if let Some(value) = self.get_feature_value(components, contrib) {
                acc + (value * contrib.direction * if contrib.half_count { 0.5 } else { 1.0 })
            } else {
                acc
            }
        })
    }

    fn get_feature_value(&self, components: &[Component], contrib: &StackupContribution) -> Option<f64> {
        components.iter()
            .find(|c| c.name == contrib.component_id)?
            .features.iter()
            .find(|f| f.name == contrib.feature_id)
            .map(|f| f.dimension.value)
    }

    pub fn get_feature<'a>(&self, components: &'a [Component], contrib: &StackupContribution) -> Option<&'a Feature> {
        components.iter()
            .find(|c| c.name == contrib.component_id)?
            .features.iter()
            .find(|f| f.name == contrib.feature_id)
    }
    pub fn calculate_distribution_params(feature: &Feature) -> DistributionParams {
        // Get feature's selected distribution type or default to Normal
        let dist_type = feature.distribution.unwrap_or(DistributionType::Normal);

        // Calculate total tolerance and standard deviation
        let total_tolerance = feature.dimension.plus_tolerance + feature.dimension.minus_tolerance;
        let std_dev = total_tolerance / 6.0; // Using 6-sigma for 99.73% coverage

        match dist_type {
            DistributionType::Normal => DistributionParams::new_normal(
                feature.dimension.value,
                std_dev
            ),
            DistributionType::Uniform => DistributionParams::new_uniform(
                feature.dimension.value - total_tolerance/2.0,
                feature.dimension.value + total_tolerance/2.0
            ),
            DistributionType::Triangular => DistributionParams::new_triangular(
                feature.dimension.value - total_tolerance/2.0,
                feature.dimension.value + total_tolerance/2.0,
                feature.dimension.value // mode is nominal value
            ),
            DistributionType::LogNormal => DistributionParams::new_lognormal(
                feature.dimension.value,
                std_dev
            ),
        }
    }

    pub fn run_analysis(&self, components: &[Component]) -> AnalysisResults {
        let mut results = AnalysisResults {
            analysis_id: self.id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            nominal: self.calculate_nominal(components),
            worst_case: None,
            rss: None,
            monte_carlo: None,
            process_capability: None,
        };

        for method in &self.methods {
            match method {
                AnalysisMethod::WorstCase => {
                    results.worst_case = Some(self.calculate_worst_case(components));
                },
                AnalysisMethod::Rss => {
                    results.rss = Some(self.calculate_rss(components));
                },
                AnalysisMethod::MonteCarlo => {
                    if let Some(settings) = &self.monte_carlo_settings {
                        results.monte_carlo = Some(self.run_monte_carlo(components, settings));
                    }
                }
            }
        }
        if let Some(mc) = &results.monte_carlo {
            let process_capability = if let (Some(usl), Some(lsl)) = 
                (self.upper_spec_limit, self.lower_spec_limit) {
                let std_dev = mc.std_dev;
                let mean = mc.mean;
                
                // Calculate Cp
                let cp = if std_dev > 0.0 {
                    Some((usl - lsl) / (6.0 * std_dev))
                } else {
                    None
                };

                // Calculate Cpk
                let cpu = (usl - mean) / (3.0 * std_dev);
                let cpl = (mean - lsl) / (3.0 * std_dev);
                let cpk = Some(cpu.min(cpl));

                // Calculate PPM using normal distribution
                let normal = StatsNormal::new(mean, std_dev).unwrap();
                let ppm_below = normal.cdf(lsl) * 1_000_000.0;
                let ppm_above = (1.0 - normal.cdf(usl)) * 1_000_000.0;
                
                // Calculate PPH (parts per hour assuming 3600 parts per hour)
                let pph_below = ppm_below * 3.6;
                let pph_above = ppm_above * 3.6;

                Some(ProcessCapability {
                    upper_spec: Some(usl),
                    lower_spec: Some(lsl),
                    cp,
                    cpk,
                    ppm_above: Some(ppm_above),
                    ppm_below: Some(ppm_below),
                    pph_above: Some(pph_above),
                    pph_below: Some(pph_below),
                })
            } else {
                None
            };

            results.process_capability = process_capability;
        }

        results
    }

    fn calculate_worst_case(&self, components: &[Component]) -> WorstCaseResult {
        let mut min = 0.0;
        let mut max = 0.0;
        let mut sensitivities = Vec::new();
        let mut total_variation = 0.0;

        // Calculate total variation first
        for contrib in &self.contributions {
            if let Some(feature) = self.get_feature(components, contrib) {
                let multiplier: f64 = if contrib.half_count { 0.5 } else { 1.0 };
                let total_tol = (feature.dimension.plus_tolerance + feature.dimension.minus_tolerance) 
                    * multiplier.abs();
                total_variation += total_tol;
            }
        }

        // Calculate individual contributions and overall min/max
        for contrib in &self.contributions {
            if let Some(feature) = self.get_feature(components, contrib) {
                let multiplier = if contrib.half_count { 0.5 } else { 1.0 };
                let direction = contrib.direction;

                // Calculate contribution to overall stack
                let nominal = feature.dimension.value * direction * multiplier;
                let total_tol = (feature.dimension.plus_tolerance + feature.dimension.minus_tolerance) 
                    * multiplier.abs();
                
                // Calculate individual min/max contributions
                let (contrib_min, contrib_max) = if direction > 0.0 {
                    (nominal - feature.dimension.minus_tolerance * multiplier,
                     nominal + feature.dimension.plus_tolerance * multiplier)
                } else {
                    (nominal - feature.dimension.plus_tolerance * multiplier,
                     nominal + feature.dimension.minus_tolerance * multiplier)
                };

                min += contrib_min;
                max += contrib_max;

                // Calculate sensitivity
                let contribution_percent = if total_variation > 0.0 {
                    total_tol / total_variation * 100.0
                } else {
                    0.0
                };

                sensitivities.push(ContributorSensitivity {
                    component_id: contrib.component_id.clone(),
                    feature_id: contrib.feature_id.clone(),
                    contribution_percent,
                    nominal_value: feature.dimension.value,
                    variation_range: (contrib_min, contrib_max),
                    correlation: None,
                    samples: None,
                });
            }
        }

        // Sort sensitivities by contribution percentage
        sensitivities.sort_by(|a, b| b.contribution_percent.partial_cmp(&a.contribution_percent).unwrap());

        WorstCaseResult { min, max, sensitivity: sensitivities }
    }

    fn calculate_rss(&self, components: &[Component]) -> RssResult {
        let mut nominal = 0.0;
        let mut sum_squares = 0.0;
        let mut sensitivities = Vec::new();
        let mut individual_variances = Vec::new();
    
        // First pass: calculate nominal and sum of squares
        for contrib in &self.contributions {
            if let Some(feature) = self.get_feature(components, contrib) {
                let multiplier = if contrib.half_count { 0.5 } else { 1.0 };
                let direction = contrib.direction;
                
                nominal += feature.dimension.value * direction * multiplier;
                
                // For RSS, use RMS of the plus and minus tolerances
                let effective_tolerance = ((feature.dimension.plus_tolerance
                                + feature.dimension.minus_tolerance) / 2.0)
                                * multiplier;
                
                // Square the tolerance and apply direction and multiplier
                let variance = (effective_tolerance).powi(2);
                sum_squares += variance;
                individual_variances.push((contrib, feature, variance));
            }
        }
    
        let std_dev = sum_squares.sqrt() / 3.0;
    
        // Second pass: calculate sensitivities
        for (contrib, feature, variance) in individual_variances {
            let contribution_percent = if sum_squares > 0.0 {
                variance / sum_squares * 100.0
            } else {
                0.0
            };
    
            sensitivities.push(ContributorSensitivity {
                component_id: contrib.component_id.clone(),
                feature_id: contrib.feature_id.clone(),
                contribution_percent,
                nominal_value: feature.dimension.value,
                variation_range: (
                    nominal - 3.0 * (variance).sqrt(),
                    nominal + 3.0 * (variance).sqrt()
                ),
                correlation: None,
                samples: None,
            });
        }
    
        // Sort sensitivities by contribution percentage
        sensitivities.sort_by(|a, b| b.contribution_percent.partial_cmp(&a.contribution_percent).unwrap());
    
        RssResult {
            min: nominal - 3.0 * std_dev,
            max: nominal + 3.0 * std_dev,
            std_dev,
            sensitivity: sensitivities,
        }
    }

    fn sample_distribution(params: &DistributionParams, rng: &mut StdRng) -> f64 {
        match params.dist_type {
            DistributionType::Normal => {
                let normal = RandNormal::new(params.mean, params.std_dev).unwrap();
                normal.sample(rng)
            },
            DistributionType::Uniform => {
                let uniform = Uniform::new(params.min, params.max);
                uniform.sample(rng)
            },
            DistributionType::Triangular => {
                Self::sample_triangular(
                    params.min,
                    params.max,
                    params.mode.unwrap_or((params.min + params.max) / 2.0),
                    rng
                )
            },
            DistributionType::LogNormal => {
                let lognormal = LogNormal::new(params.mean.ln(), params.std_dev).unwrap();
                lognormal.sample(rng)
            },
        }
    }
    fn sample_triangular(min: f64, max: f64, mode: f64, rng: &mut StdRng) -> f64 {
        let u: f64 = rng.gen();

        // Ensure mode is between min and max
        let safe_mode = mode.max(min).min(max);

        // Calculate cumulative probability at the mode
        let f_c = (safe_mode - min) / (max - min);

        if u < f_c {
            // Sample from the left side of the triangle
            min + ((u * (safe_mode - min) * (max - min)).sqrt())
        } else {
            // Sample from the right side of the triangle
            max - (((1.0 - u) * (max - safe_mode) * (max - min)).sqrt())
        }
    }

    fn calculate_correlation(&self, x: &[f64], y: &[f64], x_mean: f64, y_mean: f64) -> f64 {
        let n = x.len() as f64;
        
        let covariance = x.iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - x_mean) * (yi - y_mean))
            .sum::<f64>() / (n - 1.0);
            
        let x_std = (x.iter()
            .map(|xi| (xi - x_mean).powi(2))
            .sum::<f64>() / (n - 1.0))
            .sqrt();
            
        let y_std = (y.iter()
            .map(|yi| (yi - y_mean).powi(2))
            .sum::<f64>() / (n - 1.0))
            .sqrt();
            
        covariance / (x_std * y_std)
    }

    fn run_monte_carlo(&self, components: &[Component], settings: &MonteCarloSettings) -> MonteCarloResult {
        let mut rng = if let Some(seed) = settings.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        // Store all samples and their contributions to the total
        let mut all_samples: HashMap<(String, String), Vec<(f64, f64)>> = HashMap::new();
        let mut stackup_results = Vec::with_capacity(settings.iterations);
        
        // Initialize sample storage with vectors that will store (value, contribution) pairs
        for contrib in &self.contributions {
            all_samples.insert(
                (contrib.component_id.clone(), contrib.feature_id.clone()),
                Vec::with_capacity(settings.iterations)
            );
        }
        
        // Run simulation
        for _ in 0..settings.iterations {
            let mut stack = 0.0;
            let mut iteration_samples = Vec::new();

            // Generate all samples first
            for contrib in &self.contributions {
                if let Some(feature) = self.get_feature(components, contrib) {
                    let multiplier = if contrib.half_count { 0.5 } else { 1.0 };
                    
                    let value = if let Some(dist_params) = &contrib.distribution {
                        Self::sample_distribution(dist_params, &mut rng)
                    } else {
                        let default_params = Self::calculate_distribution_params(feature);
                        Self::sample_distribution(&default_params, &mut rng)
                    };
                    
                    // Store the raw sample and its contribution to the total
                    let contribution = value * contrib.direction * multiplier;
                    iteration_samples.push((contrib.clone(), value, contribution));
                    stack += contribution;
                }
            }

            // Store the samples and their contributions
            for (contrib, value, contribution) in iteration_samples {
                if let Some(samples) = all_samples.get_mut(&(contrib.component_id, contrib.feature_id)) {
                    samples.push((value, contribution));
                }
            }
            
            stackup_results.push(stack);
        }

        // Calculate overall statistics
        let mean = stackup_results.iter().sum::<f64>() / stackup_results.len() as f64;
        let variance = if stackup_results.len() > 1 {
            stackup_results.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / (stackup_results.len() - 1) as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        // Calculate sensitivities
        let mut sensitivities = Vec::new();
        let total_variance = variance; // Use the already calculated overall variance

        // First calculate all the variances and correlations
        let mut contrib_stats: Vec<(f64, f64)> = Vec::new(); // (variance, correlation)

        for contrib in &self.contributions {
            if let Some(samples) = all_samples.get(&(contrib.component_id.clone(), contrib.feature_id.clone())) {
                let values: Vec<f64> = samples.iter().map(|(val, _)| *val).collect();
                let contributions: Vec<f64> = samples.iter().map(|(_, contrib)| *contrib).collect();
                
                let contrib_mean = values.iter().sum::<f64>() / values.len() as f64;
                let contribution_mean = contributions.iter().sum::<f64>() / contributions.len() as f64;
                
                // Calculate contribution variance
                let contrib_variance = if contributions.len() > 1 {
                    contributions.iter()
                        .map(|x| (x - contribution_mean).powi(2))
                        .sum::<f64>() / (contributions.len() - 1) as f64
                } else {
                    0.0
                };

                // Calculate correlation
                let correlation = if total_variance > 0.0 && contrib_variance > 0.0 {
                    let covariance = contributions.iter()
                        .zip(stackup_results.iter())
                        .map(|(x, y)| (x - contribution_mean) * (y - mean))
                        .sum::<f64>() / (contributions.len() - 1) as f64;

                    covariance / (contrib_variance.sqrt() * total_variance.sqrt())
                } else {
                    0.0
                };

                contrib_stats.push((contrib_variance, correlation));
            }
        }

        // Calculate total of all variance contributions
        let total_contrib = contrib_stats.iter()
            .map(|(variance, correlation)| variance * correlation.abs())
            .sum::<f64>();

        // Now create sensitivities with properly normalized percentages
        for (i, contrib) in self.contributions.iter().enumerate() {
            if let Some(samples) = all_samples.get(&(contrib.component_id.clone(), contrib.feature_id.clone())) {
                let values: Vec<f64> = samples.iter().map(|(val, _)| *val).collect();
                let contrib_mean = values.iter().sum::<f64>() / values.len() as f64;
                let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
                let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

                let (variance, correlation) = contrib_stats[i];
                
                // Calculate normalized contribution percentage
                let contribution_percent = if total_contrib > 0.0 {
                    (variance * correlation.abs() / total_contrib) * 100.0
                } else if i == 0 {
                    100.0 // If no variance, assign all contribution to first component
                } else {
                    0.0
                };

                // Create visualization samples
                let sample_count = samples.len().min(1000);
                let step = samples.len().checked_div(sample_count).unwrap_or(1);
                let visualization_samples = samples.iter()
                    .step_by(step)
                    .take(sample_count)
                    .map(|(val, _)| (*val, mean))
                    .collect();

                sensitivities.push(ContributorSensitivity {
                    component_id: contrib.component_id.clone(),
                    feature_id: contrib.feature_id.clone(),
                    contribution_percent,
                    nominal_value: contrib_mean,
                    variation_range: (min_val, max_val),
                    correlation: Some(correlation),
                    samples: Some(visualization_samples),
                });
            }
        }

        // Sort by contribution percentage
        sensitivities.sort_by(|a, b| b.contribution_percent
            .partial_cmp(&a.contribution_percent)
            .unwrap_or(std::cmp::Ordering::Equal));

        MonteCarloResult {
            min: stackup_results.iter().copied().fold(f64::INFINITY, f64::min),
            max: stackup_results.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            mean,
            std_dev,
            confidence_intervals: Self::calculate_confidence_intervals(&mut stackup_results, settings.confidence),
            histogram: Self::calculate_histogram(&stackup_results, 20),
            sensitivity: sensitivities,
        }
    }

/// Calculate confidence intervals directly from Monte Carlo results
/// Uses actual simulation data which naturally accounts for the combined effects
/// of different distributions in the stack.
/// Calculate confidence intervals directly from Monte Carlo results
fn calculate_confidence_intervals(results: &mut Vec<f64>, user_confidence: f64) -> Vec<ConfidenceInterval> {
    // Sort results for percentile calculations
    results.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = results.len();
    if n == 0 {
        return Vec::new();
    }

    // Start with 100% interval using actual min/max values
    let mut intervals = vec![ConfidenceInterval {
        confidence_level: 1.0,
        lower_bound: results[0], // Min value
        upper_bound: results[n - 1], // Max value
    }];

    // Add standard confidence levels
    let standard_levels = vec![
        0.90f64,
        0.95f64,
        0.99f64,
        user_confidence.clamp(0.0, 0.9999),
    ];

    intervals.extend(standard_levels.into_iter().map(|confidence| {
        let alpha = 1.0 - confidence;
        let lower_index = ((alpha / 2.0) * (n as f64)).round() as usize;
        let upper_index = ((1.0 - alpha / 2.0) * (n as f64)).round() as usize;

        // Clamp indices to valid range
        let lower_index = lower_index.min(n - 1);
        let upper_index = upper_index.min(n - 1);

        ConfidenceInterval {
            confidence_level: confidence,
            lower_bound: results[lower_index],
            upper_bound: results[upper_index],
        }
    }));

    intervals
}

    fn calculate_histogram(results: &[f64], num_bins: usize) -> Vec<(f64, usize)> {
        if results.is_empty() {
            return Vec::new();
        }

        let min = results.iter().copied().fold(f64::INFINITY, f64::min);
        let max = results.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bin_width = (max - min) / num_bins as f64;
        
        let mut histogram = vec![(0.0, 0); num_bins];
        
        for i in 0..num_bins {
            let bin_start = min + i as f64 * bin_width;
            histogram[i] = (
                bin_start,
                results.iter()
                    .filter(|&x| *x >= bin_start && *x < bin_start + bin_width)
                    .count()
            );
        }

        histogram
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_worst_case_analysis() {
        // Create test components and run worst case analysis
        // TODO: Implement test cases
    }

    #[test]
    fn test_rss_analysis() {
        // Create test components and run RSS analysis
        // TODO: Implement test cases
    }

    #[test]
    fn test_monte_carlo_analysis() {
        // Create test components and run Monte Carlo analysis
        // TODO: Implement test cases
    }
}