// src/prompts/component.rs
use anyhow::Result;
use inquire::{Text, Select};

use crate::config::{Component, Feature};

/// Prompt for new component creation
pub fn prompt_new_component() -> Result<Component> {
    let name = Text::new("Component name:")
        .with_help_message("Enter a descriptive name for the component")
        .prompt()?;
    
    let revision = Text::new("Revision:")
        .with_default("A")
        .with_help_message("Component revision (A, B, C, etc.)")
        .prompt()?;
    
    let description = Text::new("Description:")
        .with_help_message("Brief description of the component (optional)")
        .prompt()?;

    let full_name = if revision.trim().is_empty() {
        name
    } else {
        format!("{} Rev {}", name, revision)
    };

    let description = if description.trim().is_empty() {
        None
    } else {
        Some(description)
    };

    Ok(Component {
        name: full_name,
        description,
        features: Vec::new(),
    })
}

/// Prompt for component editing
pub fn prompt_edit_component(component: &Component) -> Result<Component> {
    println!("Editing component: {}", component.name);
    
    let name = Text::new("Component name:")
        .with_default(&component.name)
        .prompt()?;
    
    let current_desc = component.description.as_deref().unwrap_or("");
    let description = Text::new("Description:")
        .with_default(current_desc)
        .prompt()?;

    let description = if description.trim().is_empty() {
        None
    } else {
        Some(description)
    };

    Ok(Component {
        name,
        description,
        features: component.features.clone(), // Keep existing features
    })
}

/// Select action for component management
pub fn select_component_action() -> Result<String> {
    let actions = vec![
        "Add new component".to_string(),
        "Edit component".to_string(),
        "Remove component".to_string(),
        "List components".to_string(),
        "Back to main menu".to_string(),
    ];

    Select::new("What would you like to do?", actions)
        .prompt()
        .map_err(Into::into)
}