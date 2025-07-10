// src/cli/mate.rs
use anyhow::Result;
use console::style;

use crate::state::AppState;
use crate::cli::MateCommands;
use crate::prompts::{fuzzy_select, confirm_action};
use crate::prompts::mate::{prompt_new_mate, prompt_edit_mate};

/// Handle mate relationship subcommands
pub fn handle_mate_command(state: &mut AppState, cmd: MateCommands) -> Result<()> {
    match cmd {
        MateCommands::Add => add_mate(state),
        MateCommands::List => list_mates(state),
        MateCommands::Edit => edit_mate(state),
        MateCommands::Remove => remove_mate(state),
    }
}

/// Add a new mate relationship
fn add_mate(state: &mut AppState) -> Result<()> {
    if state.project_dir.is_none() {
        return Err(anyhow::anyhow!("No project loaded. Use 'atlas new' or 'atlas open' first."));
    }

    let mate = prompt_new_mate(state)?;

    // Check for duplicate relationships
    let duplicate = state.mates.iter().any(|m| {
        (m.component_a == mate.component_a && m.feature_a == mate.feature_a &&
         m.component_b == mate.component_b && m.feature_b == mate.feature_b) ||
        (m.component_a == mate.component_b && m.feature_a == mate.feature_b &&
         m.component_b == mate.component_a && m.feature_b == mate.feature_a)
    });

    if duplicate {
        return Err(anyhow::anyhow!("Mate relationship already exists between these features"));
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Add mate {} {} <-> {} {}", mate.component_a, mate.feature_a, mate.component_b, mate.feature_b));

    state.mates.push(mate.clone());
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Added mate relationship:");
    println!("   {} {} <-> {} {}", 
             style(&mate.component_a).cyan(), 
             style(&mate.feature_a).green(),
             style(&mate.component_b).cyan(), 
             style(&mate.feature_b).green());
    println!("   Fit type: {:?}", mate.fit_type);

    Ok(())
}

/// List all mate relationships
fn list_mates(state: &AppState) -> Result<()> {
    if state.mates.is_empty() {
        println!("📭 No mate relationships defined");
        return Ok(());
    }

    println!("🔗 Mate Relationships ({} total)", state.mates.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (i, mate) in state.mates.iter().enumerate() {
        let fit_symbol = match mate.fit_type {
            crate::config::mate::FitType::Clearance => "🔵",
            crate::config::mate::FitType::Interference => "🔴", 
            crate::config::mate::FitType::Transition => "🟡",
        };

        println!("{}. {} {} {} <-> {} {} {}", 
                 style(i + 1).dim(),
                 style(&mate.component_a).cyan().bold(),
                 style(&mate.feature_a).green(),
                 fit_symbol,
                 style(&mate.component_b).cyan().bold(),
                 style(&mate.feature_b).green(),
                 style(format!("({:?})", mate.fit_type)).dim());
    }

    println!("\nLegend: 🔵 Clearance, 🔴 Interference, 🟡 Transition");

    Ok(())
}

/// Edit an existing mate relationship
fn edit_mate(state: &mut AppState) -> Result<()> {
    if state.mates.is_empty() {
        return Err(anyhow::anyhow!("No mate relationships available to edit"));
    }

    // Create a searchable representation of mates
    let mate_descriptions: Vec<String> = state.mates.iter()
        .map(|m| format!("{} {} <-> {} {} ({:?})", 
                         m.component_a, m.feature_a,
                         m.component_b, m.feature_b,
                         m.fit_type))
        .collect();

    let selected = inquire::Select::new("Select mate to edit:", mate_descriptions)
        .prompt()?;

    // Find the selected mate index
    let mate_index = state.mates.iter()
        .position(|m| format!("{} {} <-> {} {} ({:?})", 
                             m.component_a, m.feature_a,
                             m.component_b, m.feature_b,
                             m.fit_type) == selected)
        .ok_or_else(|| anyhow::anyhow!("Mate not found"))?;

    let current_mate = &state.mates[mate_index];
    let edited_mate = prompt_edit_mate(current_mate, state)?;

    // Check for duplicate relationships (excluding the current one)
    let duplicate = state.mates.iter().enumerate().any(|(i, m)| {
        i != mate_index && (
            (m.component_a == edited_mate.component_a && m.feature_a == edited_mate.feature_a &&
             m.component_b == edited_mate.component_b && m.feature_b == edited_mate.feature_b) ||
            (m.component_a == edited_mate.component_b && m.feature_a == edited_mate.feature_b &&
             m.component_b == edited_mate.component_a && m.feature_b == edited_mate.feature_a)
        )
    });

    if duplicate {
        return Err(anyhow::anyhow!("Mate relationship already exists between these features"));
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Edit mate {} {} <-> {} {}", current_mate.component_a, current_mate.feature_a, current_mate.component_b, current_mate.feature_b));

    state.mates[mate_index] = edited_mate.clone();
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Updated mate relationship:");
    println!("   {} {} <-> {} {}", 
             style(&edited_mate.component_a).cyan(), 
             style(&edited_mate.feature_a).green(),
             style(&edited_mate.component_b).cyan(), 
             style(&edited_mate.feature_b).green());
    println!("   Fit type: {:?}", edited_mate.fit_type);

    Ok(())
}

/// Remove a mate relationship
fn remove_mate(state: &mut AppState) -> Result<()> {
    if state.mates.is_empty() {
        return Err(anyhow::anyhow!("No mate relationships available to remove"));
    }

    // Create a searchable representation of mates
    let mate_descriptions: Vec<String> = state.mates.iter()
        .map(|m| format!("{} {} <-> {} {} ({:?})", 
                         m.component_a, m.feature_a,
                         m.component_b, m.feature_b,
                         m.fit_type))
        .collect();

    let selected = inquire::Select::new("Select mate to remove:", mate_descriptions)
        .prompt()?;

    // Find the selected mate index
    let mate_index = state.mates.iter()
        .position(|m| format!("{} {} <-> {} {} ({:?})", 
                             m.component_a, m.feature_a,
                             m.component_b, m.feature_b,
                             m.fit_type) == selected)
        .ok_or_else(|| anyhow::anyhow!("Mate not found"))?;

    let mate = &state.mates[mate_index];
    
    println!("Removing mate relationship:");
    println!("   {} {} <-> {} {}", 
             mate.component_a, mate.feature_a,
             mate.component_b, mate.feature_b);

    let should_continue = confirm_action("Are you sure you want to remove this mate relationship?")?;
    if !should_continue {
        println!("❌ Operation cancelled");
        return Ok(());
    }

    // Save state for undo
    state.save_to_undo_stack(format!("Remove mate {} {} <-> {} {}", mate.component_a, mate.feature_a, mate.component_b, mate.feature_b));

    let removed_mate = state.mates.remove(mate_index);
    state.update_dependencies();

    // Autosave project
    if let Err(e) = state.save_project() {
        eprintln!("⚠️  Warning: Failed to save project: {}", e);
    }

    println!("✅ Removed mate relationship: {} {} <-> {} {}", 
             style(&removed_mate.component_a).red(),
             style(&removed_mate.feature_a).red(),
             style(&removed_mate.component_b).red(),
             style(&removed_mate.feature_b).red());

    Ok(())
}