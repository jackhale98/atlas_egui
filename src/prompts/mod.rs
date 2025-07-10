// src/prompts/mod.rs
use anyhow::Result;
use inquire::{Text, Select, MultiSelect, Confirm};

use crate::config::{Component, Feature};
use crate::analysis::{StackupAnalysis, AnalysisMethod, MonteCarloSettings, DistributionType};
use crate::state::AppState;

pub mod component;
pub mod feature;
pub mod mate;
pub mod analysis;
pub mod utils;
pub mod navigation;

// Re-export main functions
pub use component::prompt_new_component;
pub use feature::prompt_new_feature;
pub use mate::prompt_new_mate;
pub use analysis::prompt_new_analysis;

/// Trait for fuzzy searchable items
pub trait Searchable {
    fn search_text(&self) -> String;
    fn display_text(&self) -> String;
}

impl Searchable for Component {
    fn search_text(&self) -> String {
        format!("{} {}", self.name, self.description.as_deref().unwrap_or(""))
    }
    
    fn display_text(&self) -> String {
        match &self.description {
            Some(desc) => format!("{} - {}", self.name, desc),
            None => self.name.clone(),
        }
    }
}

impl Searchable for Feature {
    fn search_text(&self) -> String {
        format!("{} {:?}", self.name, self.feature_type)
    }
    
    fn display_text(&self) -> String {
        format!("{} ({:?}) - {:.3}±{:.3}/{:.3}", 
                self.name, 
                self.feature_type,
                self.dimension.value,
                self.dimension.plus_tolerance,
                self.dimension.minus_tolerance)
    }
}

/// Enhanced select with fuzzy search capabilities
pub fn fuzzy_select<T: Searchable + Clone>(
    message: &str,
    items: &[T],
) -> Result<T> {
    if items.is_empty() {
        return Err(anyhow::anyhow!("No items available to select"));
    }

    let display_items: Vec<String> = items.iter()
        .map(|item| item.display_text())
        .collect();

    let selected = Select::new(message, display_items)
        .with_help_message("Type to search, use ↑↓ to navigate")
        .prompt()?;

    // Find the selected item by matching display text
    for item in items {
        if item.display_text() == selected {
            return Ok(item.clone());
        }
    }

    Err(anyhow::anyhow!("Selection not found"))
}

/// Multi-select with fuzzy search
pub fn fuzzy_multiselect<T: Searchable + Clone>(
    message: &str,
    items: &[T],
) -> Result<Vec<T>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let display_items: Vec<String> = items.iter()
        .map(|item| item.display_text())
        .collect();

    let selected_displays = MultiSelect::new(message, display_items)
        .with_help_message("Space to select, Enter to confirm")
        .prompt()?;

    let mut selected_items = Vec::new();
    for display in selected_displays {
        for item in items {
            if item.display_text() == display {
                selected_items.push(item.clone());
                break;
            }
        }
    }

    Ok(selected_items)
}

/// Get component and feature pair for mate relationships
pub fn select_component_feature_pair(
    state: &AppState,
    message: &str,
) -> Result<(String, String)> {
    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available"));
    }

    let component = fuzzy_select("Select component:", &state.components)?;
    
    if component.features.is_empty() {
        return Err(anyhow::anyhow!("Selected component has no features"));
    }

    let feature = fuzzy_select("Select feature:", &component.features)?;

    Ok((component.name, feature.name))
}

/// Confirm action with user
pub fn confirm_action(message: &str) -> Result<bool> {
    Ok(Confirm::new(message)
        .with_default(false)
        .prompt()?)
}

/// Prompt for file path
pub fn prompt_file_path(message: &str, default: Option<&str>) -> Result<String> {
    let mut prompt = Text::new(message);
    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }
    Ok(prompt.prompt()?)
}