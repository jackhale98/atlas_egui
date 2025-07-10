// src/cli/mod.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod project;
pub mod component;
pub mod feature;
pub mod mate;
pub mod analysis;
pub mod visualize;
pub mod interactive;

#[derive(Parser)]
#[command(name = "atlas")]
#[command(about = "A CLI tool for tolerance stack-up analysis")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    New {
        /// Project name (optional - will prompt if not provided)
        name: Option<String>,
    },
    /// Open an existing project
    Open {
        /// Path to project file
        path: PathBuf,
    },
    /// Component management commands
    #[command(subcommand)]
    Component(ComponentCommands),
    /// Feature management commands  
    #[command(subcommand)]
    Feature(FeatureCommands),
    /// Mate relationship commands
    #[command(subcommand)]
    Mate(MateCommands),
    /// Analysis commands
    #[command(subcommand)]
    Analysis(AnalysisCommands),
    /// Visualization commands
    #[command(subcommand)]
    Visualize(VisualizeCommands),
    /// Start interactive mode
    Interactive,
    
    /// Undo last action
    Undo,
    
    /// Redo last undone action
    Redo,
}

#[derive(Subcommand)]
pub enum ComponentCommands {
    /// Add a new component
    Add,
    /// List all components
    List,
    /// Edit an existing component
    Edit,
    /// Remove a component
    Remove,
}

#[derive(Subcommand)]
pub enum FeatureCommands {
    /// Add a new feature to a component
    Add,
    /// List features for a component
    List,
    /// Edit an existing feature
    Edit,
    /// Remove a feature
    Remove,
}

#[derive(Subcommand)]
pub enum MateCommands {
    /// Add a new mate relationship
    Add,
    /// List all mate relationships
    List,
    /// Edit an existing mate relationship
    Edit,
    /// Remove a mate relationship
    Remove,
}

#[derive(Subcommand)]
pub enum AnalysisCommands {
    /// Create a new analysis
    New,
    /// Run an existing analysis
    Run,
    /// List all analyses
    List,
    /// Show analysis results
    Results,
    /// Export analysis results
    Export,
}

#[derive(Subcommand)]
pub enum VisualizeCommands {
    /// Show dependency matrix
    Dependencies,
    /// Show analysis results
    Results,
    /// Export visualization to SVG
    Export,
}