// src/cli/project.rs
use anyhow::Result;
use std::path::PathBuf;
use inquire::{Text, Confirm};

use crate::state::AppState;
use crate::config::ProjectFile;

/// Create a new project
pub fn new_project(state: &mut AppState, name: Option<&str>) -> Result<()> {
    let project_name = match name {
        Some(n) => n.to_string(),
        None => Text::new("Project name:")
            .with_help_message("Enter a name for your new project")
            .prompt()?,
    };

    let description = Text::new("Project description (optional):")
        .prompt()
        .ok();

    let description = if description.as_ref().map_or(true, |s| s.trim().is_empty()) {
        None
    } else {
        description
    };

    // Create new project file
    state.project_file = ProjectFile {
        name: project_name.clone(),
        description,
        version: "1.0.0".to_string(),
        units: crate::config::Units::Metric,
        component_references: Vec::new(),
        analyses: Vec::new(),
    };

    // Clear existing data
    state.components.clear();
    state.mates.clear();
    state.analyses.clear();
    state.latest_results.clear();

    // Prompt for save location
    let save_path = Text::new("Save location:")
        .with_default(&format!("./{}.ron", project_name.replace(' ', "_").to_lowercase()))
        .with_help_message("Path where the project file will be saved")
        .prompt()?;

    let project_path = PathBuf::from(save_path);
    let project_dir = project_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid project path"))?
        .to_path_buf();

    state.file_manager.set_project_dir(project_dir.clone())?;
    state.project_dir = Some(project_dir);

    // Save the new project
    state.save_project()?;

    println!("✅ Created new project: {}", project_name);
    println!("📁 Saved at: {}", project_path.display());

    Ok(())
}

/// Open an existing project
pub fn open_project(state: &mut AppState, path: PathBuf) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Project file does not exist: {}", path.display()));
    }

    let project_dir = path.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid project path"))?
        .to_path_buf();

    // Set up file manager
    state.file_manager.set_project_dir(project_dir.clone())?;
    state.project_dir = Some(project_dir);

    // Load project data
    match state.file_manager.load_project(&path) {
        Ok((project_file, components, mates_file)) => {
            state.project_file = project_file;
            state.components = components;
            state.mates = mates_file.mates;
            
            // Clear analyses since we're not loading them from files anymore
            state.analyses.clear();
            state.latest_results.clear();
            
            // Update dependency graph
            state.update_mate_graph();
            
            println!("✅ Opened project: {}", state.project_file.name);
            println!("📊 Components: {}, Mates: {}, Analyses: {}", 
                     state.components.len(), 
                     state.mates.len(), 
                     state.analyses.len());

            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!("Error loading project: {}", e))
        }
    }
}

/// Show project information
pub fn show_project_info(state: &AppState) -> Result<()> {
    if state.project_dir.is_none() {
        println!("❌ No project currently loaded");
        return Ok(());
    }

    println!("📋 Project Information");
    println!("━━━━━━━━━━━━━━━━━━━━");
    println!("Name: {}", state.project_file.name);
    
    if let Some(desc) = &state.project_file.description {
        println!("Description: {}", desc);
    }
    
    println!("Version: {}", state.project_file.version);
    println!("Units: {:?}", state.project_file.units);
    
    if let Some(dir) = &state.project_dir {
        println!("Directory: {}", dir.display());
    }
    
    println!("\n📊 Statistics");
    println!("━━━━━━━━━━━━━━");
    println!("Components: {}", state.components.len());
    println!("Total Features: {}", state.components.iter().map(|c| c.features.len()).sum::<usize>());
    println!("Mate Relationships: {}", state.mates.len());
    println!("Analyses: {}", state.analyses.len());

    Ok(())
}

/// Save current project
pub fn save_project(state: &mut AppState) -> Result<()> {
    if state.project_dir.is_none() {
        return Err(anyhow::anyhow!("No project directory set. Use 'new' or 'open' first."));
    }

    state.save_project()?;
    println!("✅ Project saved successfully");
    
    Ok(())
}

/// Close current project
pub fn close_project(state: &mut AppState) -> Result<()> {
    if state.project_dir.is_none() {
        println!("No project currently loaded");
        return Ok(());
    }

    let should_save = Confirm::new("Save project before closing?")
        .with_default(true)
        .prompt()?;

    if should_save {
        save_project(state)?;
    }

    // Clear all project data
    *state = AppState::new();
    
    println!("✅ Project closed");
    Ok(())
}