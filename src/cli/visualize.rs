// src/cli/visualize.rs
use anyhow::Result;
use console::style;

use crate::state::AppState;
use crate::cli::VisualizeCommands;
use crate::visualization::{AsciiVisualizer, SvgVisualizer, Visualizer, export_to_file, get_file_extension};
use crate::analysis::StackupAnalysis;

/// Handle visualization subcommands
pub fn handle_visualize_command(state: &mut AppState, cmd: VisualizeCommands) -> Result<()> {
    match cmd {
        VisualizeCommands::Dependencies => show_dependencies(state),
        VisualizeCommands::Results => show_results(state),
        VisualizeCommands::Export => export_visualization(state),
    }
}

/// Show dependency matrix
fn show_dependencies(state: &AppState) -> Result<()> {
    if state.mates.is_empty() {
        println!("📭 No mate relationships defined");
        return Ok(());
    }

    let visualizer = AsciiVisualizer;
    let output = visualizer.render_dependency_matrix(state)?;
    println!("{}", output);

    Ok(())
}

/// Show analysis results visualization
fn show_results(state: &AppState) -> Result<()> {
    if state.analyses.is_empty() {
        return Err(anyhow::anyhow!("No analyses available"));
    }

    // Filter analyses that have results
    let analyses_with_results: Vec<_> = state.analyses.iter()
        .filter(|a| state.latest_results.contains_key(&a.id))
        .collect();

    if analyses_with_results.is_empty() {
        return Err(anyhow::anyhow!("No analysis results available. Run an analysis first."));
    }

    let analysis_descriptions: Vec<String> = analyses_with_results.iter()
        .map(|a| format!("{} ({} contributions)", a.name, a.contributions.len()))
        .collect();

    let selected = inquire::Select::new("Select analysis to visualize:", analysis_descriptions)
        .prompt()?;

    let analysis = analyses_with_results.iter()
        .find(|a| format!("{} ({} contributions)", a.name, a.contributions.len()) == selected)
        .ok_or_else(|| anyhow::anyhow!("Analysis not found"))?;

    if let Some(results) = state.latest_results.get(&analysis.id) {
        let visualizer = AsciiVisualizer;
        let output = visualizer.render_analysis_results(results, &analysis.name)?;
        println!("{}", output);

        // Show contribution breakdown if available
        show_contribution_breakdown(analysis)?;
    }

    Ok(())
}

/// Export visualization to file
fn export_visualization(state: &AppState) -> Result<()> {
    let export_types = vec![
        "Dependency Matrix",
        "Analysis Results",
    ];

    let export_type = inquire::Select::new("What would you like to export?", export_types)
        .prompt()?;

    let formats = vec!["ASCII (Text)", "SVG"];
    let format = inquire::Select::new("Export format:", formats).prompt()?;

    match export_type {
        "Dependency Matrix" => export_dependency_matrix(state, &format)?,
        "Analysis Results" => export_analysis_results(state, &format)?,
        _ => unreachable!(),
    }

    Ok(())
}

/// Export dependency matrix
fn export_dependency_matrix(state: &AppState, format: &str) -> Result<()> {
    if state.mates.is_empty() {
        return Err(anyhow::anyhow!("No mate relationships to export"));
    }

    let output = match format {
        "ASCII (Text)" => {
            let visualizer = AsciiVisualizer;
            visualizer.render_dependency_matrix(state)?
        },
        "SVG" => {
            let visualizer = SvgVisualizer;
            visualizer.render_dependency_matrix(state)?
        },
        _ => unreachable!(),
    };

    let extension = get_file_extension(format);
    let default_filename = format!("dependency_matrix.{}", extension);
    
    let export_path = inquire::Text::new("Export path:")
        .with_default(&default_filename)
        .prompt()?;

    export_to_file(&output, &export_path)?;

    println!("✅ Exported dependency matrix to: {}", style(&export_path).green().bold());

    Ok(())
}

/// Export analysis results
fn export_analysis_results(state: &AppState, format: &str) -> Result<()> {
    if state.analyses.is_empty() {
        return Err(anyhow::anyhow!("No analyses available"));
    }

    // Filter analyses that have results
    let analyses_with_results: Vec<_> = state.analyses.iter()
        .filter(|a| state.latest_results.contains_key(&a.id))
        .collect();

    if analyses_with_results.is_empty() {
        return Err(anyhow::anyhow!("No analysis results available. Run an analysis first."));
    }

    let analysis_descriptions: Vec<String> = analyses_with_results.iter()
        .map(|a| format!("{} ({} contributions)", a.name, a.contributions.len()))
        .collect();

    let selected = inquire::Select::new("Select analysis to export:", analysis_descriptions)
        .prompt()?;

    let analysis = analyses_with_results.iter()
        .find(|a| format!("{} ({} contributions)", a.name, a.contributions.len()) == selected)
        .ok_or_else(|| anyhow::anyhow!("Analysis not found"))?;

    if let Some(results) = state.latest_results.get(&analysis.id) {
        let output = match format {
            "ASCII (Text)" => {
                let visualizer = AsciiVisualizer;
                visualizer.render_analysis_results(results, &analysis.name)?
            },
            "SVG" => {
                let visualizer = SvgVisualizer;
                visualizer.render_analysis_results(results, &analysis.name)?
            },
            _ => unreachable!(),
        };

        let extension = get_file_extension(format);
        let safe_name = analysis.name.replace(' ', "_").to_lowercase();
        let default_filename = format!("{}_results.{}", safe_name, extension);
        
        let export_path = inquire::Text::new("Export path:")
            .with_default(&default_filename)
            .prompt()?;

        export_to_file(&output, &export_path)?;

        println!("✅ Exported analysis results to: {}", style(&export_path).green().bold());
    }

    Ok(())
}

/// Show contribution breakdown for an analysis
fn show_contribution_breakdown(analysis: &StackupAnalysis) -> Result<()> {
    if analysis.contributions.is_empty() {
        return Ok(());
    }

    println!("\n🧮 Contribution Breakdown");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (i, contribution) in analysis.contributions.iter().enumerate() {
        let direction_symbol = if contribution.direction > 0.0 { "+" } else { "-" };
        let half_indicator = if contribution.half_count { " (Half)" } else { "" };
        
        println!("{}. {} {} {} {:.1}{}", 
                 style(i + 1).dim(),
                 style(&contribution.component_id).cyan(),
                 style(&contribution.feature_id).green(),
                 style(direction_symbol).bold(),
                 contribution.direction.abs(),
                 style(half_indicator).dim());
    }

    Ok(())
}