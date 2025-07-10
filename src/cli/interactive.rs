// src/cli/interactive.rs
use anyhow::Result;
use inquire::Select;
use console::{style, Term};

use crate::state::AppState;
use crate::cli::{project, component, feature, mate, analysis, visualize};
use crate::prompts::navigation::{MenuResult, show_main_menu, show_submenu, confirm_save_before_exit, prompt_text};

/// Run interactive mode with a menu-driven interface
pub fn run_interactive_mode(state: &mut AppState) -> Result<()> {
    let term = Term::stdout();
    
    loop {
        // Clear screen and show header
        term.clear_screen()?;
        show_header(state)?;
        
        let action_result = select_main_action()?;
        
        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "Project Management" => handle_project_menu(state)?,
                    "Component Management" => handle_component_menu(state)?,
                    "Feature Management" => handle_feature_menu(state)?,
                    "Mate Relationships" => handle_mate_menu(state)?,
                    "Analysis" => handle_analysis_menu(state)?,
                    "Visualization" => handle_visualization_menu(state)?,
                    "Save Project" => {
                        if let Err(e) = project::save_project(state) {
                            println!("❌ Error saving project: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Exit" => {
                        if state.project_dir.is_some() {
                            // Ask if user wants to save before exiting
                            match confirm_save_before_exit()? {
                                Some(true) => {
                                    if let Err(e) = project::save_project(state) {
                                        println!("❌ Error saving project: {}", e);
                                        pause_for_input()?;
                                        continue; // Don't exit if save failed
                                    }
                                },
                                Some(false) => {
                                    // Don't save, just exit
                                },
                                None => {
                                    // User cancelled exit
                                    continue;
                                }
                            }
                            project::close_project(state)?;
                        }
                        break;
                    },
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => {
                // This shouldn't happen in main menu, but if it does, treat as exit
                if state.project_dir.is_some() {
                    match confirm_save_before_exit()? {
                        Some(true) => {
                            if let Err(e) = project::save_project(state) {
                                println!("❌ Error saving project: {}", e);
                                pause_for_input()?;
                                continue;
                            }
                        },
                        Some(false) => {},
                        None => continue,
                    }
                    project::close_project(state)?;
                }
                break;
            },
            MenuResult::Exit => {
                if state.project_dir.is_some() {
                    match confirm_save_before_exit()? {
                        Some(true) => {
                            if let Err(e) = project::save_project(state) {
                                println!("❌ Error saving project: {}", e);
                                pause_for_input()?;
                                continue;
                            }
                        },
                        Some(false) => {},
                        None => continue,
                    }
                    project::close_project(state)?;
                }
                break;
            }
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}

/// Show application header with current project status
fn show_header(state: &AppState) -> Result<()> {
    println!("{}", style("┌─────────────────────────────────────────┐").cyan());
    println!("{}", style("│              ATLAS CLI                 │").cyan().bold());
    println!("{}", style("│      Tolerance Stack-up Analysis       │").cyan());
    println!("{}", style("└─────────────────────────────────────────┘").cyan());
    println!();

    if let Some(_) = &state.project_dir {
        println!("📂 Current Project: {}", style(&state.project_file.name).green().bold());
        println!("📊 Stats: {} components, {} mates, {} analyses", 
                 state.components.len(), 
                 state.mates.len(), 
                 state.analyses.len());
    } else {
        println!("{}", style("📭 No project loaded").yellow());
    }
    
    println!();
    Ok(())
}

/// Select main menu action
fn select_main_action() -> Result<MenuResult<String>> {
    let actions = vec![
        "Project Management".to_string(),
        "Component Management".to_string(),
        "Feature Management".to_string(),
        "Mate Relationships".to_string(),
        "Analysis".to_string(),
        "Visualization".to_string(),
        "Save Project".to_string(),
        "Exit".to_string(),
    ];

    show_main_menu("What would you like to do?", actions)
}

/// Handle project management submenu
fn handle_project_menu(state: &mut AppState) -> Result<()> {
    loop {
        let actions = vec![
            "New Project".to_string(),
            "Open Project".to_string(),
            "Project Info".to_string(),
            "Close Project".to_string(),
            "Back to Main Menu".to_string(),
        ];

        let action_result = show_submenu("Project Management:", actions)?;

        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "New Project" => {
                        if let Err(e) = project::new_project(state, None) {
                            println!("❌ Error creating project: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Open Project" => {
                        match prompt_text("Project file path:")? {
                            Some(path) => {
                                if let Err(e) = project::open_project(state, path.into()) {
                                    println!("❌ Error opening project: {}", e);
                                    pause_for_input()?;
                                }
                            },
                            None => {
                                println!("❌ Project opening cancelled");
                            }
                        }
                    },
                    "Project Info" => {
                        project::show_project_info(state)?;
                        pause_for_input()?;
                    },
                    "Close Project" => {
                        project::close_project(state)?;
                        pause_for_input()?;
                    },
                    "Back to Main Menu" => break,
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => break,
            MenuResult::Exit => return Ok(()),
        }
    }
    
    Ok(())
}

/// Handle component management submenu
fn handle_component_menu(state: &mut AppState) -> Result<()> {
    loop {
        let actions = vec![
            "Add Component".to_string(),
            "List Components".to_string(),
            "Edit Component".to_string(),
            "Remove Component".to_string(),
            "Back to Main Menu".to_string(),
        ];

        let action_result = show_submenu("Component Management:", actions)?;

        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "Add Component" => {
                        if let Err(e) = component::handle_component_command(state, crate::cli::ComponentCommands::Add) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "List Components" => {
                        component::handle_component_command(state, crate::cli::ComponentCommands::List)?;
                        pause_for_input()?;
                    },
                    "Edit Component" => {
                        if let Err(e) = component::handle_component_command(state, crate::cli::ComponentCommands::Edit) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Remove Component" => {
                        if let Err(e) = component::handle_component_command(state, crate::cli::ComponentCommands::Remove) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Back to Main Menu" => break,
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => break,
            MenuResult::Exit => return Ok(()),
        }
    }
    
    Ok(())
}

/// Handle feature management submenu
fn handle_feature_menu(state: &mut AppState) -> Result<()> {
    loop {
        let actions = vec![
            "Add Feature".to_string(),
            "List Features".to_string(),
            "Edit Feature".to_string(),
            "Remove Feature".to_string(),
            "Back to Main Menu".to_string(),
        ];

        let action_result = show_submenu("Feature Management:", actions)?;

        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "Add Feature" => {
                        if let Err(e) = feature::handle_feature_command(state, crate::cli::FeatureCommands::Add) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "List Features" => {
                        if let Err(e) = feature::handle_feature_command(state, crate::cli::FeatureCommands::List) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Edit Feature" => {
                        if let Err(e) = feature::handle_feature_command(state, crate::cli::FeatureCommands::Edit) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Remove Feature" => {
                        if let Err(e) = feature::handle_feature_command(state, crate::cli::FeatureCommands::Remove) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Back to Main Menu" => break,
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => break,
            MenuResult::Exit => return Ok(()),
        }
    }
    
    Ok(())
}

/// Handle mate relationships submenu
fn handle_mate_menu(state: &mut AppState) -> Result<()> {
    loop {
        let actions = vec![
            "Add Mate".to_string(),
            "List Mates".to_string(),
            "Edit Mate".to_string(),
            "Remove Mate".to_string(),
            "Show Dependencies".to_string(),
            "Back to Main Menu".to_string(),
        ];

        let action_result = show_submenu("Mate Relationships:", actions)?;

        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "Add Mate" => {
                        if let Err(e) = mate::handle_mate_command(state, crate::cli::MateCommands::Add) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "List Mates" => {
                        mate::handle_mate_command(state, crate::cli::MateCommands::List)?;
                        pause_for_input()?;
                    },
                    "Edit Mate" => {
                        if let Err(e) = mate::handle_mate_command(state, crate::cli::MateCommands::Edit) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Remove Mate" => {
                        if let Err(e) = mate::handle_mate_command(state, crate::cli::MateCommands::Remove) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Show Dependencies" => {
                        visualize::handle_visualize_command(state, crate::cli::VisualizeCommands::Dependencies)?;
                        pause_for_input()?;
                    },
                    "Back to Main Menu" => break,
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => break,
            MenuResult::Exit => return Ok(()),
        }
    }
    
    Ok(())
}

/// Handle analysis submenu
fn handle_analysis_menu(state: &mut AppState) -> Result<()> {
    loop {
        let actions = vec![
            "New Analysis".to_string(),
            "Run Analysis".to_string(),
            "List Analyses".to_string(),
            "Show Results".to_string(),
            "Export Results".to_string(),
            "Back to Main Menu".to_string(),
        ];

        let action_result = show_submenu("Analysis:", actions)?;

        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "New Analysis" => {
                        if let Err(e) = analysis::handle_analysis_command(state, crate::cli::AnalysisCommands::New) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Run Analysis" => {
                        if let Err(e) = analysis::handle_analysis_command(state, crate::cli::AnalysisCommands::Run) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "List Analyses" => {
                        analysis::handle_analysis_command(state, crate::cli::AnalysisCommands::List)?;
                        pause_for_input()?;
                    },
                    "Show Results" => {
                        if let Err(e) = analysis::handle_analysis_command(state, crate::cli::AnalysisCommands::Results) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Export Results" => {
                        if let Err(e) = analysis::handle_analysis_command(state, crate::cli::AnalysisCommands::Export) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Back to Main Menu" => break,
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => break,
            MenuResult::Exit => return Ok(()),
        }
    }
    
    Ok(())
}

/// Handle visualization submenu
fn handle_visualization_menu(state: &mut AppState) -> Result<()> {
    loop {
        let actions = vec![
            "Show Dependencies".to_string(),
            "Show Analysis Results".to_string(),
            "Export to SVG".to_string(),
            "Back to Main Menu".to_string(),
        ];

        let action_result = show_submenu("Visualization:", actions)?;

        match action_result {
            MenuResult::Selection(action) => {
                match action.as_str() {
                    "Show Dependencies" => {
                        visualize::handle_visualize_command(state, crate::cli::VisualizeCommands::Dependencies)?;
                        pause_for_input()?;
                    },
                    "Show Analysis Results" => {
                        if let Err(e) = visualize::handle_visualize_command(state, crate::cli::VisualizeCommands::Results) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Export to SVG" => {
                        if let Err(e) = visualize::handle_visualize_command(state, crate::cli::VisualizeCommands::Export) {
                            println!("❌ Error: {}", e);
                            pause_for_input()?;
                        }
                    },
                    "Back to Main Menu" => break,
                    _ => unreachable!(),
                }
            },
            MenuResult::GoBack => break,
            MenuResult::Exit => return Ok(()),
        }
    }
    
    Ok(())
}

/// Pause and wait for user input
fn pause_for_input() -> Result<()> {
    println!("\nPress Enter to continue...");
    let _ = prompt_text("");
    Ok(())
}