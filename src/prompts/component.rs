// src/prompts/component.rs
use anyhow::Result;
use inquire::{Text, Select};

use crate::config::{Component, Feature};
use crate::prompts::navigation::{prompt_text, prompt_text_with_default, show_cancellation_message};

/// Prompt for new component creation
pub fn prompt_new_component() -> Result<Option<Component>> {
    let name = match prompt_text("Component name:")? {
        Some(name) => name,
        None => {
            show_cancellation_message("Component creation");
            return Ok(None);
        }
    };
    
    let revision = match prompt_text_with_default("Revision:", "A")? {
        Some(revision) => revision,
        None => {
            show_cancellation_message("Component creation");
            return Ok(None);
        }
    };
    
    let description = match prompt_text("Description (optional):")? {
        Some(description) => description,
        None => {
            show_cancellation_message("Component creation");
            return Ok(None);
        }
    };

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

    Ok(Some(Component {
        name: full_name,
        description,
        features: Vec::new(),
    }))
}

/// Prompt for component editing
pub fn prompt_edit_component(component: &Component) -> Result<Option<Component>> {
    println!("Editing component: {}", component.name);
    
    let name = match prompt_text_with_default("Component name:", &component.name)? {
        Some(name) => name,
        None => {
            show_cancellation_message("Component editing");
            return Ok(None);
        }
    };
    
    let current_desc = component.description.as_deref().unwrap_or("");
    let description = match prompt_text_with_default("Description:", current_desc)? {
        Some(description) => description,
        None => {
            show_cancellation_message("Component editing");
            return Ok(None);
        }
    };

    let description = if description.trim().is_empty() {
        None
    } else {
        Some(description)
    };

    Ok(Some(Component {
        name,
        description,
        features: component.features.clone(), // Keep existing features
    }))
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