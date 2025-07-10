// src/main.rs
use clap::Parser;
use anyhow::Result;

mod analysis;
mod cli;
mod config;
mod file;
mod state;
mod prompts;
mod visualization;

use cli::{Cli, Commands};
use state::AppState;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut app_state = AppState::new();

    // Auto-detect project file in current directory unless specific command is given
    let should_auto_detect = matches!(cli.command, 
        Commands::Component(_) | Commands::Feature(_) | Commands::Mate(_) | 
        Commands::Analysis(_) | Commands::Visualize(_) | Commands::Interactive
    );

    if should_auto_detect {
        if let Some(project_path) = find_project_file()? {
            println!("📂 Auto-detected project: {}", project_path.display());
            cli::project::open_project(&mut app_state, project_path)?;
        }
    }

    match cli.command {
        Commands::New { name } => {
            cli::project::new_project(&mut app_state, name.as_deref())?;
        }
        Commands::Open { path } => {
            cli::project::open_project(&mut app_state, path)?;
        }
        Commands::Component(cmd) => {
            cli::component::handle_component_command(&mut app_state, cmd)?;
        }
        Commands::Feature(cmd) => {
            cli::feature::handle_feature_command(&mut app_state, cmd)?;
        }
        Commands::Mate(cmd) => {
            cli::mate::handle_mate_command(&mut app_state, cmd)?;
        }
        Commands::Analysis(cmd) => {
            cli::analysis::handle_analysis_command(&mut app_state, cmd)?;
        }
        Commands::Visualize(cmd) => {
            cli::visualize::handle_visualize_command(&mut app_state, cmd)?;
        }
        Commands::Interactive => {
            cli::interactive::run_interactive_mode(&mut app_state)?;
        }
        Commands::Undo => {
            handle_undo(&mut app_state)?;
        }
        Commands::Redo => {
            handle_redo(&mut app_state)?;
        }
    }

    Ok(())
}

/// Find project file in current working directory
fn find_project_file() -> Result<Option<PathBuf>> {
    use walkdir::WalkDir;
    
    let current_dir = std::env::current_dir()?;
    
    // First, look for files explicitly named "project.ron"
    for entry in WalkDir::new(&current_dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(file_name) = entry.path().file_name() {
                if file_name == "project.ron" {
                    return Ok(Some(entry.path().to_path_buf()));
                }
            }
        }
    }
    
    // If no project.ron found, try to identify project files by parsing them
    for entry in WalkDir::new(&current_dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "ron" {
                    // Skip known non-project files
                    if let Some(file_name) = entry.path().file_name() {
                        let file_name_str = file_name.to_string_lossy();
                        if file_name_str == "mates.ron" || file_name_str.starts_with("components/") {
                            continue;
                        }
                    }
                    
                    // Try to parse as a project file to verify it's actually a project
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if content.contains("component_references") && content.contains("name") {
                            return Ok(Some(entry.path().to_path_buf()));
                        }
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Handle undo command
fn handle_undo(app_state: &mut AppState) -> Result<()> {
    if app_state.project_dir.is_none() {
        println!("❌ No project loaded. Use 'atlas new' or 'atlas open' first.");
        return Ok(());
    }

    match app_state.undo()? {
        Some(description) => {
            println!("↶ Undone: {}", description);
            if app_state.can_redo() {
                println!("💡 Use 'atlas redo' to restore this change");
            }
        }
        None => {
            println!("❌ Nothing to undo");
        }
    }
    
    Ok(())
}

/// Handle redo command
fn handle_redo(app_state: &mut AppState) -> Result<()> {
    if app_state.project_dir.is_none() {
        println!("❌ No project loaded. Use 'atlas new' or 'atlas open' first.");
        return Ok(());
    }

    match app_state.redo()? {
        Some(description) => {
            println!("↷ Redone: {}", description);
            if app_state.can_undo() {
                println!("💡 Use 'atlas undo' to undo this change");
            }
        }
        None => {
            println!("❌ Nothing to redo");
        }
    }
    
    Ok(())
}