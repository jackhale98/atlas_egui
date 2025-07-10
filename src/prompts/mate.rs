// src/prompts/mate.rs
use anyhow::Result;
use inquire::{Select};

use crate::config::mate::{Mate, FitType};
use crate::state::AppState;
use crate::prompts::{fuzzy_select, select_component_feature_pair};

/// Prompt for new mate relationship
pub fn prompt_new_mate(state: &AppState) -> Result<Mate> {
    if state.components.len() < 2 {
        return Err(anyhow::anyhow!("Need at least 2 components to create a mate relationship"));
    }

    println!("Creating new mate relationship between component features");
    
    let (component_a, feature_a) = select_component_feature_pair(state, "First component/feature:")?;
    let (component_b, feature_b) = select_component_feature_pair(state, "Second component/feature:")?;

    // Ensure we're not mating a feature to itself
    if component_a == component_b && feature_a == feature_b {
        return Err(anyhow::anyhow!("Cannot mate a feature to itself"));
    }

    let fit_type = select_fit_type()?;

    Ok(Mate {
        component_a,
        feature_a,
        component_b,
        feature_b,
        fit_type,
    })
}

/// Prompt for mate editing
pub fn prompt_edit_mate(mate: &Mate, state: &AppState) -> Result<Mate> {
    println!("Editing mate: {} {} <-> {} {}", 
             mate.component_a, mate.feature_a,
             mate.component_b, mate.feature_b);

    println!("Select new first component/feature:");
    let (component_a, feature_a) = select_component_feature_pair(state, "First component/feature:")?;
    
    println!("Select new second component/feature:");
    let (component_b, feature_b) = select_component_feature_pair(state, "Second component/feature:")?;

    // Ensure we're not mating a feature to itself
    if component_a == component_b && feature_a == feature_b {
        return Err(anyhow::anyhow!("Cannot mate a feature to itself"));
    }

    let fit_type = select_fit_type_with_default(mate.fit_type)?;

    Ok(Mate {
        component_a,
        feature_a,
        component_b,
        feature_b,
        fit_type,
    })
}

/// Select fit type
fn select_fit_type() -> Result<FitType> {
    let options = vec![
        ("Clearance", FitType::Clearance),
        ("Interference", FitType::Interference),
        ("Transition", FitType::Transition),
    ];

    let choice = Select::new("Fit type:", options.iter().map(|(name, _)| *name).collect())
        .with_starting_cursor(0) // Default to Clearance
        .prompt()?;

    options.iter()
        .find(|(name, _)| *name == choice)
        .map(|(_, ft)| *ft)
        .ok_or_else(|| anyhow::anyhow!("Invalid fit type selected"))
}

/// Select fit type with default
fn select_fit_type_with_default(default: FitType) -> Result<FitType> {
    let options = vec![
        ("Clearance", FitType::Clearance),
        ("Interference", FitType::Interference),
        ("Transition", FitType::Transition),
    ];

    let default_name = match default {
        FitType::Clearance => "Clearance",
        FitType::Interference => "Interference",
        FitType::Transition => "Transition",
    };

    let choice = Select::new("Fit type:", options.iter().map(|(name, _)| *name).collect())
        .with_starting_cursor(options.iter().position(|(name, _)| *name == default_name).unwrap_or(0))
        .prompt()?;

    options.iter()
        .find(|(name, _)| *name == choice)
        .map(|(_, ft)| *ft)
        .ok_or_else(|| anyhow::anyhow!("Invalid fit type selected"))
}

/// Select action for mate management
pub fn select_mate_action() -> Result<String> {
    let actions = vec![
        "Add new mate".to_string(),
        "Edit mate".to_string(),
        "Remove mate".to_string(),
        "List mates".to_string(),
        "Show dependency matrix".to_string(),
        "Back to main menu".to_string(),
    ];

    Select::new("What would you like to do?", actions)
        .prompt()
        .map_err(Into::into)
}