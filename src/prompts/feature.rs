// src/prompts/feature.rs
use anyhow::Result;
use inquire::{Text, Select, CustomType};

use crate::config::{Feature, FeatureType, Dimension};
use crate::analysis::DistributionType;

/// Prompt for new feature creation
pub fn prompt_new_feature() -> Result<Feature> {
    let name = Text::new("Feature name:")
        .with_help_message("Enter a descriptive name for the feature")
        .prompt()?;

    let feature_type = select_feature_type()?;

    let value: f64 = CustomType::new("Nominal value:")
        .with_help_message("Enter the nominal dimension value")
        .prompt()?;

    let plus_tolerance: f64 = CustomType::new("Plus tolerance:")
        .with_default(0.1)
        .with_help_message("Enter the plus tolerance")
        .prompt()?;

    let minus_tolerance: f64 = CustomType::new("Minus tolerance:")
        .with_default(0.1)
        .with_help_message("Enter the minus tolerance")
        .prompt()?;

    let distribution = select_distribution_type()?;

    let drawing_location = Text::new("Drawing location (optional):")
        .with_help_message("Enter drawing location/reference (e.g., 'View A-A', 'Sheet 2, Detail B', etc.)")
        .prompt_skippable()?;

    let mut feature = Feature::new(name, feature_type, value, plus_tolerance, minus_tolerance);
    feature.update_distribution(distribution);
    feature.drawing_location = drawing_location;

    Ok(feature)
}

/// Prompt for feature editing
pub fn prompt_edit_feature(feature: &Feature) -> Result<Feature> {
    println!("Editing feature: {}", feature.name);
    
    let name = Text::new("Feature name:")
        .with_default(&feature.name)
        .prompt()?;

    let feature_type = select_feature_type_with_default(feature.feature_type)?;

    let value: f64 = CustomType::new("Nominal value:")
        .with_default(feature.dimension.value)
        .prompt()?;

    let plus_tolerance: f64 = CustomType::new("Plus tolerance:")
        .with_default(feature.dimension.plus_tolerance)
        .prompt()?;

    let minus_tolerance: f64 = CustomType::new("Minus tolerance:")
        .with_default(feature.dimension.minus_tolerance)
        .prompt()?;

    let current_dist = feature.distribution.unwrap_or(DistributionType::Normal);
    let distribution = select_distribution_type_with_default(current_dist)?;

    let drawing_location = Text::new("Drawing location (optional):")
        .with_default(&feature.drawing_location.as_deref().unwrap_or(""))
        .with_help_message("Enter drawing location/reference (e.g., 'View A-A', 'Sheet 2, Detail B', etc.)")
        .prompt_skippable()?;

    let mut new_feature = Feature::new(name, feature_type, value, plus_tolerance, minus_tolerance);
    new_feature.update_distribution(distribution);
    new_feature.drawing_location = drawing_location;

    Ok(new_feature)
}

/// Select feature type
fn select_feature_type() -> Result<FeatureType> {
    let options = vec![
        ("External", FeatureType::External),
        ("Internal", FeatureType::Internal),
    ];

    let choice = Select::new("Feature type:", options.iter().map(|(name, _)| *name).collect())
        .prompt()?;

    options.iter()
        .find(|(name, _)| *name == choice)
        .map(|(_, ft)| *ft)
        .ok_or_else(|| anyhow::anyhow!("Invalid feature type selected"))
}

/// Select feature type with default
fn select_feature_type_with_default(default: FeatureType) -> Result<FeatureType> {
    let options = vec![
        ("External", FeatureType::External),
        ("Internal", FeatureType::Internal),
    ];

    let default_name = match default {
        FeatureType::External => "External",
        FeatureType::Internal => "Internal",
    };

    let choice = Select::new("Feature type:", options.iter().map(|(name, _)| *name).collect())
        .with_starting_cursor(options.iter().position(|(name, _)| *name == default_name).unwrap_or(0))
        .prompt()?;

    options.iter()
        .find(|(name, _)| *name == choice)
        .map(|(_, ft)| *ft)
        .ok_or_else(|| anyhow::anyhow!("Invalid feature type selected"))
}

/// Select distribution type
fn select_distribution_type() -> Result<DistributionType> {
    let options = vec![
        ("Normal", DistributionType::Normal),
        ("Uniform", DistributionType::Uniform),
        ("Triangular", DistributionType::Triangular),
        ("Log Normal", DistributionType::LogNormal),
    ];

    let choice = Select::new("Distribution type:", options.iter().map(|(name, _)| *name).collect())
        .with_starting_cursor(0) // Default to Normal
        .prompt()?;

    options.iter()
        .find(|(name, _)| *name == choice)
        .map(|(_, dt)| *dt)
        .ok_or_else(|| anyhow::anyhow!("Invalid distribution type selected"))
}

/// Select distribution type with default
fn select_distribution_type_with_default(default: DistributionType) -> Result<DistributionType> {
    let options = vec![
        ("Normal", DistributionType::Normal),
        ("Uniform", DistributionType::Uniform),
        ("Triangular", DistributionType::Triangular),
        ("Log Normal", DistributionType::LogNormal),
    ];

    let default_name = match default {
        DistributionType::Normal => "Normal",
        DistributionType::Uniform => "Uniform",
        DistributionType::Triangular => "Triangular",
        DistributionType::LogNormal => "Log Normal",
    };

    let choice = Select::new("Distribution type:", options.iter().map(|(name, _)| *name).collect())
        .with_starting_cursor(options.iter().position(|(name, _)| *name == default_name).unwrap_or(0))
        .prompt()?;

    options.iter()
        .find(|(name, _)| *name == choice)
        .map(|(_, dt)| *dt)
        .ok_or_else(|| anyhow::anyhow!("Invalid distribution type selected"))
}

/// Select action for feature management
pub fn select_feature_action() -> Result<String> {
    let actions = vec![
        "Add new feature".to_string(),
        "Edit feature".to_string(),
        "Remove feature".to_string(),
        "List features".to_string(),
        "Back to main menu".to_string(),
    ];

    Select::new("What would you like to do?", actions)
        .prompt()
        .map_err(Into::into)
}