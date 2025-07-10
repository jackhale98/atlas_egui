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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut app_state = AppState::new();

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
    }

    Ok(())
}