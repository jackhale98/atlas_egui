// src/config/mod.rs
pub mod project;
pub mod component;
pub mod feature;
pub mod mate;

// Re-export commonly used types
pub use project::{ProjectFile, Units};
pub use component::{Component, ComponentReference};
pub use feature::{Feature, FeatureType, Dimension};
pub use mate::{Mate, FitType};
