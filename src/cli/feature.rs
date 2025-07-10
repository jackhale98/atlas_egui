// src/cli/feature.rs
use anyhow::Result;
use console::style;

use crate::state::AppState;
use crate::cli::FeatureCommands;
use crate::prompts::{fuzzy_select, confirm_action};
use crate::prompts::feature::{prompt_new_feature, prompt_edit_feature};

/// Handle feature subcommands
pub fn handle_feature_command(state: &mut AppState, cmd: FeatureCommands) -> Result<()> {
    match cmd {
        FeatureCommands::Add => add_feature(state),
        FeatureCommands::List => list_features(state),
        FeatureCommands::Edit => edit_feature(state),
        FeatureCommands::Remove => remove_feature(state),
    }
}

/// Add a new feature to a component
fn add_feature(state: &mut AppState) -> Result<()> {
    if state.project_dir.is_none() {
        return Err(anyhow::anyhow!("No project loaded. Use 'atlas new' or 'atlas open' first."));
    }

    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available. Create a component first."));
    }

    let component = fuzzy_select("Select component to add feature to:", &state.components)?;
    let feature = prompt_new_feature()?;

    // Find the component and add the feature
    let component_index = state.components.iter()
        .position(|c| c.name == component.name)
        .ok_or_else(|| anyhow::anyhow!("Component not found"))?;

    // Check for duplicate feature names within the component
    if state.components[component_index].features.iter().any(|f| f.name == feature.name) {
        return Err(anyhow::anyhow!("Feature '{}' already exists in component '{}'", feature.name, component.name));
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Add feature '{}' to component '{}'", feature.name, component.name));

    state.components[component_index].features.push(feature.clone());
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Added feature '{}' to component '{}'", 
             style(&feature.name).green().bold(), 
             style(&component.name).cyan().bold());
    
    println!("   Type: {:?}, Value: {:.3}±{:.3}/{:.3}", 
             feature.feature_type,
             feature.dimension.value,
             feature.dimension.plus_tolerance,
             feature.dimension.minus_tolerance);

    Ok(())
}

/// List all features, optionally filtered by component
fn list_features(state: &AppState) -> Result<()> {
    if state.components.is_empty() {
        println!("📭 No components in project");
        return Ok(());
    }

    let total_features: usize = state.components.iter().map(|c| c.features.len()).sum();
    
    if total_features == 0 {
        println!("📭 No features defined in any component");
        return Ok(());
    }

    println!("🔧 Features ({} total)", total_features);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for component in &state.components {
        if component.features.is_empty() {
            continue;
        }

        println!("\n📦 {} ({} features)", 
                 style(&component.name).cyan().bold(), 
                 component.features.len());
        
        for (i, feature) in component.features.iter().enumerate() {
            println!("  {}. {} ({:?})", 
                     style(i + 1).dim(), 
                     style(&feature.name).green(), 
                     feature.feature_type);
            
            println!("     Value: {:.3} (+{:.3}/-{:.3})", 
                     feature.dimension.value,
                     feature.dimension.plus_tolerance,
                     feature.dimension.minus_tolerance);
            
            if let Some(dist) = feature.distribution {
                println!("     Distribution: {:?}", dist);
            }
        }
    }

    Ok(())
}

/// Edit an existing feature
fn edit_feature(state: &mut AppState) -> Result<()> {
    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available"));
    }

    // Find component with features
    let components_with_features: Vec<_> = state.components.iter()
        .filter(|c| !c.features.is_empty())
        .cloned()
        .collect();

    if components_with_features.is_empty() {
        return Err(anyhow::anyhow!("No features available to edit"));
    }

    let component = fuzzy_select("Select component:", &components_with_features)?;
    let feature = fuzzy_select("Select feature to edit:", &component.features)?;

    // Find the indices
    let component_index = state.components.iter()
        .position(|c| c.name == component.name)
        .ok_or_else(|| anyhow::anyhow!("Component not found"))?;

    let feature_index = component.features.iter()
        .position(|f| f.name == feature.name)
        .ok_or_else(|| anyhow::anyhow!("Feature not found"))?;

    let edited_feature = prompt_edit_feature(&feature)?;

    // Check for name conflicts (excluding the current feature)
    if edited_feature.name != feature.name &&
       state.components[component_index].features.iter().any(|f| f.name == edited_feature.name) {
        return Err(anyhow::anyhow!("Feature '{}' already exists in component '{}'", edited_feature.name, component.name));
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Edit feature '{}' in component '{}'", feature.name, component.name));

    // Update mate relationships if feature name changed
    if edited_feature.name != feature.name {
        for mate in &mut state.mates {
            if mate.component_a == component.name && mate.feature_a == feature.name {
                mate.feature_a = edited_feature.name.clone();
            }
            if mate.component_b == component.name && mate.feature_b == feature.name {
                mate.feature_b = edited_feature.name.clone();
            }
        }
    }

    state.components[component_index].features[feature_index] = edited_feature.clone();
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Updated feature: {}", style(&edited_feature.name).green().bold());

    Ok(())
}

/// Remove a feature from a component
fn remove_feature(state: &mut AppState) -> Result<()> {
    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available"));
    }

    // Find component with features
    let components_with_features: Vec<_> = state.components.iter()
        .filter(|c| !c.features.is_empty())
        .cloned()
        .collect();

    if components_with_features.is_empty() {
        return Err(anyhow::anyhow!("No features available to remove"));
    }

    let component = fuzzy_select("Select component:", &components_with_features)?;
    let feature = fuzzy_select("Select feature to remove:", &component.features)?;

    // Check for mate dependencies
    let dependent_count = state.mates.iter()
        .filter(|m| (m.component_a == component.name && m.feature_a == feature.name) ||
                   (m.component_b == component.name && m.feature_b == feature.name))
        .count();

    if dependent_count > 0 {
        println!("⚠️  Warning: This feature is used in {} mate relationship{}:", 
                 dependent_count,
                 if dependent_count != 1 { "s" } else { "" });
        
        for mate in state.mates.iter()
            .filter(|m| (m.component_a == component.name && m.feature_a == feature.name) ||
                       (m.component_b == component.name && m.feature_b == feature.name)) {
            println!("   {} {} <-> {} {}", 
                     mate.component_a, mate.feature_a,
                     mate.component_b, mate.feature_b);
        }
        
        let should_continue = confirm_action("Remove feature and all its mate relationships?")?;
        if !should_continue {
            println!("❌ Operation cancelled");
            return Ok(());
        }

        // Save state for undo
        state.save_to_undo_stack(format!("Remove feature '{}' from component '{}' with {} mates", feature.name, component.name, dependent_count));

        // Remove dependent mates
        state.mates.retain(|m| !((m.component_a == component.name && m.feature_a == feature.name) ||
                                 (m.component_b == component.name && m.feature_b == feature.name)));
    } else {
        // Save state for undo
        state.save_to_undo_stack(format!("Remove feature '{}' from component '{}'", feature.name, component.name));
    }

    // Find and remove the feature
    let component_index = state.components.iter()
        .position(|c| c.name == component.name)
        .ok_or_else(|| anyhow::anyhow!("Component not found"))?;

    let feature_index = component.features.iter()
        .position(|f| f.name == feature.name)
        .ok_or_else(|| anyhow::anyhow!("Feature not found"))?;

    state.components[component_index].features.remove(feature_index);
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Removed feature: {}", style(&feature.name).red().bold());
    
    if dependent_count > 0 {
        println!("   Also removed {} dependent mate relationship{}", 
                 dependent_count,
                 if dependent_count != 1 { "s" } else { "" });
    }

    Ok(())
}