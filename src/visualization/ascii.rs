// src/visualization/ascii.rs
use anyhow::Result;
use textplots::{Chart, Plot, Shape};
use console::style;
use std::collections::HashMap;

use crate::state::AppState;
use crate::analysis::{AnalysisResults, SensitivityAnalysis};

/// Render dependency matrix as ASCII table
pub fn render_dependency_matrix(state: &AppState) -> Result<String> {
    let mut output = String::new();
    
    if state.mates.is_empty() {
        output.push_str("📭 No mate relationships defined\n");
        return Ok(output);
    }

    // Collect all features
    let mut features: Vec<(String, String)> = Vec::new(); // (component, feature)
    for component in &state.components {
        for feature in &component.features {
            features.push((component.name.clone(), feature.name.clone()));
        }
    }

    if features.is_empty() {
        output.push_str("📭 No features defined\n");
        return Ok(output);
    }

    // Create dependency matrix
    let mut matrix = vec![vec![' '; features.len()]; features.len()];
    
    for mate in &state.mates {
        if let (Some(i), Some(j)) = (
            features.iter().position(|(c, f)| c == &mate.component_a && f == &mate.feature_a),
            features.iter().position(|(c, f)| c == &mate.component_b && f == &mate.feature_b)
        ) {
            let symbol = match mate.fit_type {
                crate::config::mate::FitType::Clearance => 'C',
                crate::config::mate::FitType::Interference => 'I',
                crate::config::mate::FitType::Transition => 'T',
            };
            matrix[i][j] = symbol;
            matrix[j][i] = symbol; // Symmetric
        }
    }

    // Render the matrix
    output.push_str(&format!("🔗 Dependency Matrix ({} features)\n", features.len()));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Header row
    output.push_str("         ");
    for (i, (comp, feat)) in features.iter().enumerate() {
        output.push_str(&format!("{:>3}", i + 1));
    }
    output.push('\n');
    
    // Matrix rows
    for (i, (_comp, _feat)) in features.iter().enumerate() {
        output.push_str(&format!("{:>3}. ", i + 1));
        
        for j in 0..features.len() {
            let symbol = if matrix[i][j] != ' ' { 
                matrix[i][j].to_string() 
            } else { 
                "·".to_string() 
            };
            output.push_str(&format!("{:>3}", symbol));
        }
        output.push('\n');
    }
    
    // Legend
    output.push('\n');
    output.push_str("Legend:\n");
    output.push_str("  Features: ");
    for (i, (comp, feat)) in features.iter().enumerate() {
        output.push_str(&format!("{}:{}.{} ", i + 1, comp, feat));
        if (i + 1) % 3 == 0 {
            output.push_str("\n           ");
        }
    }
    output.push_str("\n  Symbols: C=Clearance, I=Interference, T=Transition, ·=No relationship\n");

    Ok(output)
}

/// Render analysis results as ASCII text with charts
pub fn render_analysis_results(results: &AnalysisResults, analysis_name: &str) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("📊 Analysis Results: {}\n", style(analysis_name).cyan().bold()));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Summary statistics
    output.push_str(&format!("Mean: {:.6}\n", results.mean));
    output.push_str(&format!("Std Dev: {:.6}\n", results.std_dev));
    output.push_str(&format!("Min: {:.6}\n", results.min));
    output.push_str(&format!("Max: {:.6}\n", results.max));
    output.push_str(&format!("Range: {:.6}\n", results.max - results.min));
    
    // Process capability indices
    if let Some(cp) = results.cp {
        output.push_str(&format!("Cp: {:.3}\n", cp));
    }
    if let Some(cpk) = results.cpk {
        output.push_str(&format!("Cpk: {:.3}\n", cpk));
    }
    if let Some(pp) = results.pp {
        output.push_str(&format!("Pp: {:.3}\n", pp));
    }
    if let Some(ppk) = results.ppk {
        output.push_str(&format!("Ppk: {:.3}\n", ppk));
    }
    
    // Specification limits
    if let Some(spec) = &results.specification_limits {
        output.push_str("\nSpecification Limits:\n");
        if let Some(lsl) = spec.lower_spec_limit {
            output.push_str(&format!("  LSL: {:.6}\n", lsl));
        }
        if let Some(usl) = spec.upper_spec_limit {
            output.push_str(&format!("  USL: {:.6}\n", usl));
        }
        if let Some(target) = spec.target {
            output.push_str(&format!("  Target: {:.6}\n", target));
        }
    }
    
    // Percentiles
    if !results.percentiles.is_empty() {
        output.push_str("\nPercentiles:\n");
        for (p, value) in &results.percentiles {
            output.push_str(&format!("  {:.1}%: {:.6}\n", p, value));
        }
    }
    
    // Sensitivity analysis
    if let Some(sensitivity) = &results.sensitivity_analysis {
        output.push_str(&render_sensitivity_analysis(sensitivity)?);
    }
    
    // Histogram
    if !results.histogram_data.is_empty() {
        output.push_str(&render_histogram(&results.histogram_data, "Distribution")?);
    }
    
    Ok(output)
}

/// Render histogram using textplots
pub fn render_histogram(data: &[f64], title: &str) -> Result<String> {
    if data.is_empty() {
        return Ok("📭 No data to plot\n".to_string());
    }

    let mut output = String::new();
    output.push_str(&format!("\n📈 {}\n", title));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create histogram bins
    let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max_val - min_val).abs() < f64::EPSILON {
        output.push_str("All values are the same\n");
        return Ok(output);
    }

    let num_bins = 20;
    let bin_width = (max_val - min_val) / num_bins as f64;
    let mut bins = vec![0usize; num_bins];
    
    for &value in data {
        let bin_index = ((value - min_val) / bin_width).floor() as usize;
        let bin_index = bin_index.min(num_bins - 1);
        bins[bin_index] += 1;
    }
    
    // Convert to plot points
    let points: Vec<(f32, f32)> = bins.iter().enumerate()
        .map(|(i, &count)| {
            let x = min_val + (i as f64 + 0.5) * bin_width;
            (x as f32, count as f32)
        })
        .collect();

    // Create the plot
    let chart_output = Chart::new(120, 30, min_val as f32, max_val as f32)
        .lineplot(&Shape::Bars(&points))
        .to_string();
    
    output.push_str(&chart_output);
    output.push('\n');
    
    // Add statistics below the chart
    output.push_str(&format!("Samples: {}, Min: {:.3}, Max: {:.3}, Range: {:.3}\n", 
                             data.len(), min_val, max_val, max_val - min_val));

    Ok(output)
}

/// Render a simple bar chart for component contributions
pub fn render_contribution_chart(contributions: &[(String, f64)], title: &str) -> Result<String> {
    let mut output = String::new();
    
    if contributions.is_empty() {
        return Ok("📭 No contributions to display\n".to_string());
    }
    
    output.push_str(&format!("\n📊 {}\n", title));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let max_value = contributions.iter()
        .map(|(_, v)| v.abs())
        .fold(0.0f64, f64::max);
    
    if max_value == 0.0 {
        output.push_str("All contributions are zero\n");
        return Ok(output);
    }
    
    let bar_width = 50;
    
    for (name, value) in contributions {
        let normalized = (value.abs() / max_value * bar_width as f64) as usize;
        let bar = "█".repeat(normalized);
        let sign = if *value >= 0.0 { "+" } else { "-" };
        
        output.push_str(&format!("{:>20}: {:>8.3} {} {}\n", 
                                name, value, sign, bar));
    }
    
    Ok(output)
}

/// Render sensitivity analysis as ASCII chart
pub fn render_sensitivity_analysis(sensitivity: &SensitivityAnalysis) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("\n🎯 Sensitivity Analysis\n"));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    if sensitivity.contributions.is_empty() {
        output.push_str("No sensitivity data available\n");
        return Ok(output);
    }
    
    output.push_str(&format!("Total Variance: {:.6}\n", sensitivity.total_variance));
    output.push_str(&format!("Total Std Dev: {:.6}\n\n", sensitivity.total_variance.sqrt()));
    
    output.push_str("Variance Contributions (sorted by impact):\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let max_name_length = sensitivity.contributions.iter()
        .map(|c| format!("{}.{}", c.component_name, c.feature_name).len())
        .max()
        .unwrap_or(20);
    
    let max_percentage = sensitivity.contributions.iter()
        .map(|c| c.percentage)
        .fold(0.0f64, f64::max);
    
    for (i, contrib) in sensitivity.contributions.iter().enumerate() {
        let feature_name = format!("{}.{}", contrib.component_name, contrib.feature_name);
        let bar_width = if max_percentage > 0.0 {
            ((contrib.percentage / max_percentage) * 30.0) as usize
        } else {
            0
        };
        
        let bar = "█".repeat(bar_width);
        let percentage_color = if contrib.percentage > 50.0 {
            style(format!("{:>6.1}%", contrib.percentage)).red().bold()
        } else if contrib.percentage > 25.0 {
            style(format!("{:>6.1}%", contrib.percentage)).yellow().bold()
        } else {
            style(format!("{:>6.1}%", contrib.percentage)).green()
        };
        
        output.push_str(&format!(
            "{:>2}. {:width$} {:>10.6} {} {}\n",
            i + 1,
            feature_name,
            contrib.std_dev_contribution,
            percentage_color,
            bar,
            width = max_name_length
        ));
    }
    
    // Add interpretation guide
    output.push_str("\nInterpretation:\n");
    output.push_str("  High contributors (>50%): Focus areas for tolerance tightening\n");
    output.push_str("  Medium contributors (25-50%): Secondary optimization targets\n");
    output.push_str("  Low contributors (<25%): Minimal impact on variation\n");
    
    Ok(output)
}