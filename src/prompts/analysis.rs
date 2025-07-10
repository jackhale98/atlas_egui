// src/prompts/analysis.rs
use anyhow::Result;
use inquire::{Text, Select, MultiSelect, CustomType};

use crate::analysis::{StackupAnalysis, AnalysisMethod, MonteCarloSettings, Contribution, SpecificationLimits};
use crate::state::AppState;
use crate::prompts::{fuzzy_select, fuzzy_multiselect};

/// Prompt for new analysis creation
pub fn prompt_new_analysis(state: &AppState) -> Result<StackupAnalysis> {
    let name = Text::new("Analysis name:")
        .with_help_message("Enter a descriptive name for the analysis")
        .prompt()?;

    let methods = select_analysis_methods()?;
    let monte_carlo_settings = prompt_monte_carlo_settings()?;
    let contributions = prompt_contributions(state)?;
    let specification_limits = prompt_specification_limits()?;

    Ok(StackupAnalysis {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        methods,
        monte_carlo_settings,
        contributions,
        specification_limits,
    })
}

/// Select analysis methods
fn select_analysis_methods() -> Result<Vec<AnalysisMethod>> {
    let available_methods = vec![
        ("Monte Carlo", AnalysisMethod::MonteCarlo),
        ("Root Sum Square", AnalysisMethod::RootSumSquare),
        ("Worst Case", AnalysisMethod::WorstCase),
    ];

    let method_names: Vec<&str> = available_methods.iter().map(|(name, _)| *name).collect();
    
    let selected_names = MultiSelect::new("Analysis methods:", method_names)
        .with_help_message("Select one or more analysis methods (Space to select, Enter to confirm)")
        .prompt()?;

    let mut methods = Vec::new();
    for name in selected_names {
        if let Some((_, method)) = available_methods.iter().find(|(n, _)| *n == name) {
            methods.push(*method);
        }
    }

    if methods.is_empty() {
        // Default to Monte Carlo if nothing selected
        methods.push(AnalysisMethod::MonteCarlo);
    }

    Ok(methods)
}

/// Prompt for Monte Carlo settings
fn prompt_monte_carlo_settings() -> Result<MonteCarloSettings> {
    let num_samples: u32 = CustomType::new("Number of Monte Carlo samples:")
        .with_default(10000)
        .with_help_message("Higher values give more accurate results but take longer")
        .prompt()?;

    let seed: Option<u64> = CustomType::new("Random seed (optional):")
        .with_help_message("Leave empty for random seed, or enter a number for reproducible results")
        .prompt()
        .ok();

    Ok(MonteCarloSettings {
        num_samples,
        seed,
    })
}

/// Prompt for analysis contributions
fn prompt_contributions(state: &AppState) -> Result<Vec<Contribution>> {
    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available for analysis"));
    }

    let mut contributions: Vec<Contribution> = Vec::new();

    println!("Select features that contribute to this stackup:");
    
    loop {
        // Show current contributions
        if !contributions.is_empty() {
            println!("\nCurrent contributions:");
            for (i, contrib) in contributions.iter().enumerate() {
                let direction_symbol = if contrib.direction > 0.0 { "+" } else { "-" };
                println!("  {}. {} {} {} {:.1}", 
                         i + 1, 
                         contrib.component_id,
                         contrib.feature_id,
                         direction_symbol,
                         contrib.direction);
            }
            println!();
        }

        let actions = vec![
            "Add contribution".to_string(),
            "Remove contribution".to_string(),
            "Finish".to_string(),
        ];

        let action = Select::new("What would you like to do?", actions).prompt()?;

        match action.as_str() {
            "Add contribution" => {
                if let Ok(contribution) = prompt_single_contribution(state) {
                    contributions.push(contribution);
                }
            },
            "Remove contribution" => {
                if !contributions.is_empty() {
                    let contrib_descriptions: Vec<String> = contributions.iter()
                        .map(|c| format!("{} {} ({:.1})", c.component_id, c.feature_id, c.direction))
                        .collect();
                    
                    let selected = Select::new("Select contribution to remove:", contrib_descriptions)
                        .prompt()?;
                    
                    contributions.retain(|c| 
                        format!("{} {} ({:.1})", c.component_id, c.feature_id, c.direction) != selected
                    );
                }
            },
            "Finish" => break,
            _ => unreachable!(),
        }
    }

    if contributions.is_empty() {
        return Err(anyhow::anyhow!("At least one contribution is required for analysis"));
    }

    Ok(contributions)
}

/// Prompt for a single contribution
fn prompt_single_contribution(state: &AppState) -> Result<Contribution> {
    // Select component
    let component = fuzzy_select("Select component:", &state.components)?;
    
    if component.features.is_empty() {
        return Err(anyhow::anyhow!("Selected component has no features"));
    }

    // Select feature
    let feature = fuzzy_select("Select feature:", &component.features)?;

    // Direction
    let direction_options = vec![
        ("Positive (+1.0)", 1.0),
        ("Negative (-1.0)", -1.0),
        ("Custom", 0.0), // Will be replaced
    ];

    let direction_choice = Select::new("Direction in stackup:", 
                                     direction_options.iter().map(|(name, _)| *name).collect())
        .prompt()?;

    let direction = if direction_choice == "Custom" {
        CustomType::new("Direction multiplier:")
            .with_help_message("Positive values add to the stackup, negative values subtract")
            .prompt()?
    } else {
        direction_options.iter()
            .find(|(name, _)| *name == direction_choice)
            .map(|(_, dir)| *dir)
            .unwrap_or(1.0)
    };

    // Half count option
    let half_count = inquire::Confirm::new("Use half count?")
        .with_default(false)
        .with_help_message("Half count is used for features that contribute only half their tolerance")
        .prompt()?;

    Ok(Contribution {
        component_id: component.name,
        feature_id: feature.name,
        direction,
        half_count,
    })
}

/// Prompt for specification limits
fn prompt_specification_limits() -> Result<Option<SpecificationLimits>> {
    let has_spec_limits = inquire::Confirm::new("Define specification limits for process capability analysis?")
        .with_default(false)
        .with_help_message("Specification limits are required to calculate Cp, Cpk, Pp, and Ppk")
        .prompt()?;

    if !has_spec_limits {
        return Ok(None);
    }

    let lower_spec_limit: Option<f64> = CustomType::new("Lower specification limit (optional):")
        .with_help_message("Leave empty if no lower limit")
        .prompt()
        .ok();

    let upper_spec_limit: Option<f64> = CustomType::new("Upper specification limit (optional):")
        .with_help_message("Leave empty if no upper limit")
        .prompt()
        .ok();

    let target: Option<f64> = CustomType::new("Target value (optional):")
        .with_help_message("Leave empty if no target value")
        .prompt()
        .ok();

    if lower_spec_limit.is_none() && upper_spec_limit.is_none() {
        println!("⚠️  Warning: No specification limits defined. Process capability indices will not be calculated.");
        return Ok(None);
    }

    Ok(Some(SpecificationLimits {
        lower_spec_limit,
        upper_spec_limit,
        target,
    }))
}

/// Select action for analysis management
pub fn select_analysis_action() -> Result<String> {
    let actions = vec![
        "New analysis".to_string(),
        "Run analysis".to_string(),
        "View results".to_string(),
        "Export results".to_string(),
        "List analyses".to_string(),
        "Back to main menu".to_string(),
    ];

    Select::new("What would you like to do?", actions)
        .prompt()
        .map_err(Into::into)
}