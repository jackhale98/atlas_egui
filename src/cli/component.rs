// src/cli/component.rs
use anyhow::Result;
use console::style;

use crate::state::AppState;
use crate::cli::ComponentCommands;
use crate::prompts::{fuzzy_select, confirm_action};
use crate::prompts::component::{prompt_new_component, prompt_edit_component};

/// Handle component subcommands
pub fn handle_component_command(state: &mut AppState, cmd: ComponentCommands) -> Result<()> {
    match cmd {
        ComponentCommands::Add => add_component(state),
        ComponentCommands::List => list_components(state),
        ComponentCommands::Edit => edit_component(state),
        ComponentCommands::Remove => remove_component(state),
    }
}

/// Add a new component
fn add_component(state: &mut AppState) -> Result<()> {
    if state.project_dir.is_none() {
        return Err(anyhow::anyhow!("No project loaded. Use 'atlas new' or 'atlas open' first."));
    }

    let component = match prompt_new_component()? {
        Some(component) => component,
        None => return Ok(()), // User cancelled
    };
    
    // Check for duplicate names
    if state.components.iter().any(|c| c.name == component.name) {
        return Err(anyhow::anyhow!("Component with name '{}' already exists", component.name));
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Add component '{}'", component.name));

    state.components.push(component.clone());
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Added component: {}", style(&component.name).cyan().bold());
    
    if let Some(desc) = &component.description {
        println!("   Description: {}", desc);
    }

    Ok(())
}

/// List all components
fn list_components(state: &AppState) -> Result<()> {
    if state.components.is_empty() {
        println!("📭 No components in project");
        return Ok(());
    }

    println!("📦 Components ({} total)", state.components.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (i, component) in state.components.iter().enumerate() {
        println!("{}. {}", 
                 style(i + 1).dim(), 
                 style(&component.name).cyan().bold());
        
        if let Some(desc) = &component.description {
            println!("   {}", style(desc).dim());
        }
        
        let feature_count = component.features.len();
        if feature_count > 0 {
            println!("   {} feature{}", feature_count, if feature_count != 1 { "s" } else { "" });
        } else {
            println!("   {}", style("No features").yellow());
        }
        
        if i < state.components.len() - 1 {
            println!();
        }
    }

    Ok(())
}

/// Edit an existing component
fn edit_component(state: &mut AppState) -> Result<()> {
    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available to edit"));
    }

    let component = fuzzy_select("Select component to edit:", &state.components)?;
    
    // Find the index of the selected component
    let index = state.components.iter()
        .position(|c| c.name == component.name)
        .ok_or_else(|| anyhow::anyhow!("Component not found"))?;

    let edited_component = match prompt_edit_component(&component)? {
        Some(component) => component,
        None => return Ok(()), // User cancelled
    };

    // Check for name conflicts (excluding the current component)
    if edited_component.name != component.name && 
       state.components.iter().any(|c| c.name == edited_component.name) {
        return Err(anyhow::anyhow!("Component with name '{}' already exists", edited_component.name));
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Edit component '{}'", component.name));

    // Update mate relationships if component name changed
    if edited_component.name != component.name {
        for mate in &mut state.mates {
            if mate.component_a == component.name {
                mate.component_a = edited_component.name.clone();
            }
            if mate.component_b == component.name {
                mate.component_b = edited_component.name.clone();
            }
        }
    }

    state.components[index] = edited_component.clone();
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Updated component: {}", style(&edited_component.name).cyan().bold());

    Ok(())
}

/// Remove a component
fn remove_component(state: &mut AppState) -> Result<()> {
    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available to remove"));
    }

    let component = fuzzy_select("Select component to remove:", &state.components)?;

    // Check for mate dependencies
    let dependent_count = state.mates.iter()
        .filter(|m| m.component_a == component.name || m.component_b == component.name)
        .count();

    if dependent_count > 0 {
        println!("⚠️  Warning: This component is used in {} mate relationship{}:", 
                 dependent_count,
                 if dependent_count != 1 { "s" } else { "" });
        
        for mate in state.mates.iter()
            .filter(|m| m.component_a == component.name || m.component_b == component.name) {
            println!("   {} {} <-> {} {}", 
                     mate.component_a, mate.feature_a,
                     mate.component_b, mate.feature_b);
        }
        
        let should_continue = confirm_action("Remove component and all its mate relationships?")?;
        if !should_continue {
            println!("❌ Operation cancelled");
            return Ok(());
        }

        // Save state for undo
        state.save_to_undo_stack(format!("Remove component '{}' with {} mates", component.name, dependent_count));

        // Remove dependent mates
        state.mates.retain(|m| m.component_a != component.name && m.component_b != component.name);
    } else {
        // Save state for undo
        state.save_to_undo_stack(format!("Remove component '{}'", component.name));
    }

    // Find and remove the component
    let index = state.components.iter()
        .position(|c| c.name == component.name)
        .ok_or_else(|| anyhow::anyhow!("Component not found"))?;

    state.components.remove(index);
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Removed component: {}", style(&component.name).red().bold());
    
    if dependent_count > 0 {
        println!("   Also removed {} dependent mate relationship{}", 
                 dependent_count,
                 if dependent_count != 1 { "s" } else { "" });
    }

    Ok(())
}