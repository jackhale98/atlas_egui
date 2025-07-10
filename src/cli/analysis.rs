// src/cli/analysis.rs
use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::state::AppState;
use crate::cli::AnalysisCommands;
use crate::prompts::{fuzzy_select, confirm_action};
use crate::analysis::{StackupAnalysis, run_monte_carlo_analysis, AnalysisResults};

/// Handle analysis subcommands
pub fn handle_analysis_command(state: &mut AppState, cmd: AnalysisCommands) -> Result<()> {
    match cmd {
        AnalysisCommands::New => new_analysis(state),
        AnalysisCommands::Run => run_analysis(state),
        AnalysisCommands::List => list_analyses(state),
        AnalysisCommands::Results => show_results(state),
        AnalysisCommands::Export => export_results(state),
    }
}

/// Create a new analysis
fn new_analysis(state: &mut AppState) -> Result<()> {
    if state.project_dir.is_none() {
        return Err(anyhow::anyhow!("No project loaded. Use 'atlas new' or 'atlas open' first."));
    }

    if state.components.is_empty() {
        return Err(anyhow::anyhow!("No components available. Create components first."));
    }

    let analysis = crate::prompts::analysis::prompt_new_analysis(state)?;

    // Check for duplicate names
    if state.analyses.iter().any(|a| a.name == analysis.name) {
        return Err(anyhow::anyhow!("Analysis with name '{}' already exists", analysis.name));
    }

    state.analyses.push(analysis.clone());

    println!("✅ Created analysis: {}", style(&analysis.name).cyan().bold());
    println!("   Methods: {:?}", analysis.methods);
    println!("   Contributions: {}", analysis.contributions.len());

    Ok(())
}

/// Run an existing analysis
fn run_analysis(state: &mut AppState) -> Result<()> {
    if state.analyses.is_empty() {
        return Err(anyhow::anyhow!("No analyses available to run"));
    }

    // Create searchable representations
    let analysis_descriptions: Vec<String> = state.analyses.iter()
        .map(|a| format!("{} ({} contributions)", a.name, a.contributions.len()))
        .collect();

    let selected = inquire::Select::new("Select analysis to run:", analysis_descriptions)
        .prompt()?;

    // Find the selected analysis
    let analysis = state.analyses.iter()
        .find(|a| format!("{} ({} contributions)", a.name, a.contributions.len()) == selected)
        .ok_or_else(|| anyhow::anyhow!("Analysis not found"))?;

    println!("🚀 Running analysis: {}", style(&analysis.name).cyan().bold());

    // Show progress bar
    let pb = ProgressBar::new(analysis.monte_carlo_settings.num_samples as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} samples ({eta})")?
        .progress_chars("#>-"));

    // Run the analysis
    let results = run_monte_carlo_analysis(analysis, state, Some(&pb))?;

    pb.finish_with_message("✅ Analysis complete");

    // Store results
    state.latest_results.insert(analysis.id.clone(), results.clone());

    // Display summary
    println!("\n📊 Analysis Results Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Mean: {:.6}", results.mean);
    println!("Std Dev: {:.6}", results.std_dev);
    println!("Min: {:.6}", results.min);
    println!("Max: {:.6}", results.max);
    
    if let Some(cp) = results.cp {
        println!("Cp: {:.3}", cp);
    }
    if let Some(cpk) = results.cpk {
        println!("Cpk: {:.3}", cpk);
    }

    Ok(())
}

/// List all analyses
fn list_analyses(state: &AppState) -> Result<()> {
    if state.analyses.is_empty() {
        println!("📭 No analyses defined");
        return Ok(());
    }

    println!("🔬 Analyses ({} total)", state.analyses.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (i, analysis) in state.analyses.iter().enumerate() {
        let has_results = state.latest_results.contains_key(&analysis.id);
        let status = if has_results { "✅" } else { "⏳" };
        
        println!("{}. {} {} {}", 
                 style(i + 1).dim(),
                 status,
                 style(&analysis.name).cyan().bold(),
                 style(format!("({} contributions)", analysis.contributions.len())).dim());
        
        println!("   Methods: {:?}", analysis.methods);
        println!("   Monte Carlo: {} samples", analysis.monte_carlo_settings.num_samples);
        
        if has_results {
            if let Some(results) = state.latest_results.get(&analysis.id) {
                println!("   Last result: μ={:.3}, σ={:.3}", results.mean, results.std_dev);
            }
        }
    }

    println!("\nLegend: ✅ Has results, ⏳ Not run yet");

    Ok(())
}

/// Show analysis results
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

    let selected = inquire::Select::new("Select analysis to view results:", analysis_descriptions)
        .prompt()?;

    let analysis = analyses_with_results.iter()
        .find(|a| format!("{} ({} contributions)", a.name, a.contributions.len()) == selected)
        .ok_or_else(|| anyhow::anyhow!("Analysis not found"))?;

    if let Some(results) = state.latest_results.get(&analysis.id) {
        // Use ASCII visualization to show results
        let visualization = crate::visualization::ascii::render_analysis_results(results, &analysis.name)?;
        println!("{}", visualization);
    }

    Ok(())
}

/// Export analysis results
fn export_results(state: &AppState) -> Result<()> {
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

    // Select export format
    let formats = vec!["CSV", "JSON", "ASCII Report"];
    let format = inquire::Select::new("Export format:", formats).prompt()?;

    // Get export path
    let default_filename = format!("{}_results.{}", 
                                   analysis.name.replace(' ', "_").to_lowercase(),
                                   match format {
                                       "CSV" => "csv",
                                       "JSON" => "json", 
                                       "ASCII Report" => "txt",
                                       _ => "txt",
                                   });

    let export_path = inquire::Text::new("Export path:")
        .with_default(&default_filename)
        .prompt()?;

    if let Some(results) = state.latest_results.get(&analysis.id) {
        match format {
            "CSV" => export_csv(results, &export_path)?,
            "JSON" => export_json(results, &export_path)?,
            "ASCII Report" => {
                let report = crate::visualization::ascii::render_analysis_results(results, &analysis.name)?;
                std::fs::write(&export_path, report)?;
            },
            _ => unreachable!(),
        }

        println!("✅ Exported results to: {}", style(&export_path).green().bold());
    }

    Ok(())
}

/// Export results as CSV
fn export_csv(results: &crate::analysis::AnalysisResults, path: &str) -> Result<()> {
    use std::fs::File;
    use csv::Writer;

    let mut wtr = Writer::from_writer(File::create(path)?);

    // Write headers
    wtr.write_record(&["Sample", "Value"])?;

    // Write data points
    for (i, &value) in results.histogram_data.iter().enumerate() {
        wtr.write_record(&[i.to_string(), value.to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}

/// Export results as JSON
fn export_json(results: &crate::analysis::AnalysisResults, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(path, json)?;
    Ok(())
}