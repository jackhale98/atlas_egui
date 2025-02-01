// src/config/feature.rs
use serde::{Serialize, Deserialize};
use crate::analysis::stackup::DistributionType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum FeatureType {
    External,
    Internal,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub feature_type: FeatureType,
    pub dimension: Dimension,
    pub distribution: Option<DistributionType>,
    pub distribution_params: Option<DistributionParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub value: f64,
    pub plus_tolerance: f64,
    pub minus_tolerance: f64,
}

impl Feature {
    pub fn new(name: String, feature_type: FeatureType, value: f64, plus_tol: f64, minus_tol: f64) -> Self {
        let mut new_feature = Self {
            name,
            feature_type,
            dimension: Dimension {
                value,
                plus_tolerance: plus_tol,
                minus_tolerance: minus_tol,
            },
            distribution: Some(DistributionType::Normal),
            distribution_params: None,
        };

        // Calculate initial distribution parameters
        new_feature.distribution_params = Some(DistributionParams::calculate_from_feature(&new_feature));
        new_feature
    }

    pub fn update_distribution(&mut self, dist_type: DistributionType) {
        self.distribution = Some(dist_type);
        if self.distribution_params.as_ref().map_or(true, |p| p.calculated) {
            self.distribution_params = Some(DistributionParams::calculate_from_feature(self));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionParams {
    pub dist_type: DistributionType,
    pub calculated: bool,  // Whether to use auto-calculated values
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mode: Option<f64>,
    pub shape: Option<f64>,
    pub scale: Option<f64>,
}

impl DistributionParams {
    pub fn calculate_from_feature(feature: &Feature) -> Self {
        let total_tolerance = feature.dimension.plus_tolerance + feature.dimension.minus_tolerance;
        let mean = feature.dimension.value;
        let std_dev = total_tolerance / 6.0; // Using 6-sigma for 99.73% coverage

        Self {
            dist_type: feature.distribution.unwrap_or(DistributionType::Normal),
            calculated: true,
            mean: Some(mean),
            std_dev: Some(std_dev),
            min: Some(mean - 3.0 * std_dev),
            max: Some(mean + 3.0 * std_dev),
            mode: Some(mean),  // For triangular distribution
            shape: Some(2.0),  // Default shape parameter for Weibull
            scale: Some(std_dev), // Default scale parameter for Weibull
        }
    }

    pub fn get_required_params(&self) -> Vec<(&'static str, f64)> {
        match self.dist_type {
            DistributionType::Normal => vec![
                ("Mean", self.mean.unwrap_or(0.0)),
                ("Std Dev", self.std_dev.unwrap_or(0.0)),
            ],
            DistributionType::Uniform => vec![
                ("Min", self.min.unwrap_or(0.0)),
                ("Max", self.max.unwrap_or(0.0)),
            ],
            DistributionType::Triangular => vec![
                ("Min", self.min.unwrap_or(0.0)),
                ("Max", self.max.unwrap_or(0.0)),
                ("Mode", self.mode.unwrap_or(0.0)),
            ],
            DistributionType::LogNormal => vec![
                ("Mean", self.mean.unwrap_or(0.0)),
                ("Std Dev", self.std_dev.unwrap_or(0.0)),
            ],
        }
    }
}
