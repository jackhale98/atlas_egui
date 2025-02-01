// src/config/mate.rs
use serde::{Serialize, Deserialize};
use super::Feature;
use super::feature::FeatureType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FitType {
    Clearance,
    Transition,
    Interference,
}

impl Default for FitType {
    fn default() -> Self {
        FitType::Clearance // Makes sense as a default since it's the most common
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitValidation {
    pub is_valid: bool,
    pub nominal_fit: f64,
    pub min_fit: f64,
    pub max_fit: f64,
    pub error_message: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mate {
    pub id: String,
    pub component_a: String,
    pub feature_a: String,
    pub component_b: String,
    pub feature_b: String,
    pub fit_type: FitType,
}

impl Mate {
    pub fn new(
        id: String,
        component_a: String,
        feature_a: String,
        component_b: String,
        feature_b: String,
        fit_type: FitType,
    ) -> Self {
        Self {
            id,
            component_a,
            feature_a,
            component_b,
            feature_b,
            fit_type,
        }
    }

    pub fn calculate_nominal_fit(&self, feature_a: &Feature, feature_b: &Feature) -> f64 {
        match (feature_a.feature_type, feature_b.feature_type) {
            (FeatureType::External, FeatureType::Internal) => {
                feature_b.dimension.value - feature_a.dimension.value
            },
            (FeatureType::Internal, FeatureType::External) => {
                feature_a.dimension.value - feature_b.dimension.value
            },
            _ => 0.0  // Invalid mate combination
        }
    }

    pub fn calculate_min_fit(&self, feature_a: &Feature, feature_b: &Feature) -> f64 {
        match (feature_a.feature_type, feature_b.feature_type) {
            (FeatureType::External, FeatureType::Internal) => {
                (feature_b.dimension.value - feature_b.dimension.minus_tolerance) -
                (feature_a.dimension.value + feature_a.dimension.plus_tolerance)
            },
            (FeatureType::Internal, FeatureType::External) => {
                (feature_a.dimension.value - feature_a.dimension.minus_tolerance) -
                (feature_b.dimension.value + feature_b.dimension.plus_tolerance)
            },
            _ => 0.0
        }
    }

    pub fn calculate_max_fit(&self, feature_a: &Feature, feature_b: &Feature) -> f64 {
        match (feature_a.feature_type, feature_b.feature_type) {
            (FeatureType::External, FeatureType::Internal) => {
                (feature_b.dimension.value + feature_b.dimension.plus_tolerance) -
                (feature_a.dimension.value - feature_a.dimension.minus_tolerance)
            },
            (FeatureType::Internal, FeatureType::External) => {
                (feature_a.dimension.value + feature_a.dimension.plus_tolerance) -
                (feature_b.dimension.value - feature_b.dimension.minus_tolerance)
            },
            _ => 0.0
        }
    }

    pub fn validate(&self, feature_a: &Feature, feature_b: &Feature) -> FitValidation {
        self.fit_type.validate_fit(feature_a, feature_b)
    }
}

impl FitType {
    pub fn validate_fit(&self, feature_a: &Feature, feature_b: &Feature) -> FitValidation {
        // Calculate fits using existing mate calculation methods
        let nominal_fit = match (feature_a.feature_type, feature_b.feature_type) {
            (FeatureType::External, FeatureType::Internal) => {
                feature_b.dimension.value - feature_a.dimension.value
            },
            (FeatureType::Internal, FeatureType::External) => {
                feature_a.dimension.value - feature_b.dimension.value
            },
            _ => return FitValidation {
                is_valid: false,
                nominal_fit: 0.0,
                min_fit: 0.0,
                max_fit: 0.0,
                error_message: Some("Invalid feature type combination".to_string())
            }
        };

        let min_fit = match (feature_a.feature_type, feature_b.feature_type) {
            (FeatureType::External, FeatureType::Internal) => {
                (feature_b.dimension.value - feature_b.dimension.minus_tolerance) -
                (feature_a.dimension.value + feature_a.dimension.plus_tolerance)
            },
            (FeatureType::Internal, FeatureType::External) => {
                (feature_a.dimension.value - feature_a.dimension.minus_tolerance) -
                (feature_b.dimension.value + feature_b.dimension.plus_tolerance)
            },
            _ => 0.0
        };

        let max_fit = match (feature_a.feature_type, feature_b.feature_type) {
            (FeatureType::External, FeatureType::Internal) => {
                (feature_b.dimension.value + feature_b.dimension.plus_tolerance) -
                (feature_a.dimension.value - feature_a.dimension.minus_tolerance)
            },
            (FeatureType::Internal, FeatureType::External) => {
                (feature_a.dimension.value + feature_a.dimension.plus_tolerance) -
                (feature_b.dimension.value - feature_b.dimension.minus_tolerance)
            },
            _ => 0.0
        };

        // Validate based on fit type
        match self {
            FitType::Clearance => {
                if min_fit <= 0.0 {
                    FitValidation {
                        is_valid: false,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: Some("Clearance fit must have positive minimum clearance".to_string())
                    }
                } else {
                    FitValidation {
                        is_valid: true,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: None
                    }
                }
            },
            FitType::Interference => {
                if max_fit >= 0.0 {
                    FitValidation {
                        is_valid: false,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: Some("Interference fit must have negative maximum clearance".to_string())
                    }
                } else {
                    FitValidation {
                        is_valid: true,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: None
                    }
                }
            },
            FitType::Transition => {
                if min_fit >= 0.0 || max_fit <= 0.0 {
                    FitValidation {
                        is_valid: false,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: Some("Transition fit must have both positive and negative clearances".to_string())
                    }
                } else {
                    FitValidation {
                        is_valid: true,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: None
                    }
                }
            }
        }
    }
}
