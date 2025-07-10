// src/visualization/svg.rs
use anyhow::Result;
use svg::Document;
use svg::node::element::{Rectangle, Text, Line, Circle, Group};

use crate::state::AppState;
use crate::analysis::AnalysisResults;

/// Render dependency matrix as SVG
pub fn render_dependency_matrix(state: &AppState) -> Result<String> {
    if state.mates.is_empty() {
        return Ok(create_empty_message("No mate relationships defined"));
    }

    // Collect all features
    let mut features: Vec<(String, String)> = Vec::new(); // (component, feature)
    for component in &state.components {
        for feature in &component.features {
            features.push((component.name.clone(), feature.name.clone()));
        }
    }

    if features.is_empty() {
        return Ok(create_empty_message("No features defined"));
    }

    let cell_size = 30;
    let margin = 80;
    let text_height = 15;
    
    let matrix_size = features.len();
    let width = margin * 2 + matrix_size * cell_size;
    let height = margin * 2 + matrix_size * cell_size + text_height;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height);

    // Background
    document = document.add(Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "white"));

    // Title
    document = document.add(Text::new("Dependency Matrix")
        .set("x", width / 2)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("font-size", "20")
        .set("font-weight", "bold")
        .set("fill", "black"));

    // Create dependency matrix
    let mut matrix = vec![vec![None; features.len()]; features.len()];
    
    for mate in &state.mates {
        if let (Some(i), Some(j)) = (
            features.iter().position(|(c, f)| c == &mate.component_a && f == &mate.feature_a),
            features.iter().position(|(c, f)| c == &mate.component_b && f == &mate.feature_b)
        ) {
            let color = match mate.fit_type {
                crate::config::mate::FitType::Clearance => "#4CAF50", // Green
                crate::config::mate::FitType::Interference => "#F44336", // Red
                crate::config::mate::FitType::Transition => "#FF9800", // Orange
            };
            matrix[i][j] = Some(color);
            matrix[j][i] = Some(color); // Symmetric
        }
    }

    // Draw matrix grid and relationships
    for i in 0..matrix_size {
        for j in 0..matrix_size {
            let x = margin + j * cell_size;
            let y = margin + i * cell_size + text_height;

            // Cell background
            let fill_color = if i == j {
                "#E0E0E0" // Diagonal
            } else if let Some(color) = matrix[i][j] {
                color
            } else {
                "white"
            };

            document = document.add(Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", cell_size)
                .set("height", cell_size)
                .set("fill", fill_color)
                .set("stroke", "#CCCCCC")
                .set("stroke-width", 1));

            // Feature index
            if i == j {
                document = document.add(Text::new(format!("{}", i + 1))
                    .set("x", x + cell_size / 2)
                    .set("y", y + cell_size / 2 + 5)
                    .set("text-anchor", "middle")
                    .set("font-size", "12")
                    .set("fill", "black"));
            }
        }
    }

    // Legend
    let legend_y = height - 60;
    let legend_items = vec![
        ("Clearance", "#4CAF50"),
        ("Interference", "#F44336"),
        ("Transition", "#FF9800"),
    ];

    for (i, (label, color)) in legend_items.iter().enumerate() {
        let x = 20 + i * 120;
        
        document = document.add(Rectangle::new()
            .set("x", x)
            .set("y", legend_y)
            .set("width", 15)
            .set("height", 15)
            .set("fill", *color));

        document = document.add(Text::new(*label)
            .set("x", x + 20)
            .set("y", legend_y + 12)
            .set("font-size", "12")
            .set("fill", "black"));
    }

    // Feature labels (simplified - show first few)
    let max_labels = std::cmp::min(features.len(), 10);
    for (i, (comp, feat)) in features.iter().take(max_labels).enumerate() {
        let label = format!("{}. {}.{}", i + 1, 
                           comp.chars().take(8).collect::<String>(),
                           feat.chars().take(8).collect::<String>());
        
        document = document.add(Text::new(label)
            .set("x", 10)
            .set("y", margin + i * cell_size + text_height + cell_size / 2 + 5)
            .set("font-size", "10")
            .set("fill", "black"));
    }

    Ok(document.to_string())
}

/// Render analysis results as SVG
pub fn render_analysis_results(results: &AnalysisResults, analysis_name: &str) -> Result<String> {
    let width = 800;
    let height = 600;
    let margin = 60;
    let chart_width = width - 2 * margin;
    let chart_height = height - 2 * margin - 100; // Leave space for stats

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height);

    // Background
    document = document.add(Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "white"));

    // Title
    document = document.add(Text::new(format!("Analysis Results: {}", analysis_name))
        .set("x", width / 2)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("font-size", "18")
        .set("font-weight", "bold")
        .set("fill", "black"));

    // Statistics box
    let stats_y = 50;
    let stats = vec![
        format!("Mean: {:.6}", results.mean),
        format!("Std Dev: {:.6}", results.std_dev),
        format!("Min: {:.6}", results.min),
        format!("Max: {:.6}", results.max),
    ];

    for (i, stat) in stats.iter().enumerate() {
        document = document.add(Text::new(stat)
            .set("x", 20)
            .set("y", stats_y + i * 20)
            .set("font-size", "12")
            .set("fill", "black"));
    }

    // Histogram if data available
    if !results.histogram_data.is_empty() {
        document = add_histogram_to_svg(document, &results.histogram_data, margin, 150, chart_width, chart_height)?;
    }

    Ok(document.to_string())
}

/// Render histogram as SVG
pub fn render_histogram(data: &[f64], title: &str) -> Result<String> {
    if data.is_empty() {
        return Ok(create_empty_message("No data to plot"));
    }

    let width = 800;
    let height = 400;
    let margin = 60;
    let chart_width = width - 2 * margin;
    let chart_height = height - 2 * margin;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height);

    // Background
    document = document.add(Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "white"));

    // Title
    document = document.add(Text::new(title)
        .set("x", width / 2)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("font-size", "16")
        .set("font-weight", "bold")
        .set("fill", "black"));

    document = add_histogram_to_svg(document, data, margin, margin, chart_width, chart_height)?;

    Ok(document.to_string())
}

/// Add histogram bars to SVG document
fn add_histogram_to_svg(
    mut document: Document, 
    data: &[f64], 
    x_offset: i32, 
    y_offset: i32, 
    chart_width: i32, 
    chart_height: i32
) -> Result<Document> {
    let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max_val - min_val).abs() < f64::EPSILON {
        return Ok(document.add(Text::new("All values are the same")
            .set("x", x_offset + chart_width / 2)
            .set("y", y_offset + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", "14")
            .set("fill", "gray")));
    }

    let num_bins = 30;
    let bin_width = (max_val - min_val) / num_bins as f64;
    let mut bins = vec![0usize; num_bins];
    
    for &value in data {
        let bin_index = ((value - min_val) / bin_width).floor() as usize;
        let bin_index = bin_index.min(num_bins - 1);
        bins[bin_index] += 1;
    }

    let max_count = *bins.iter().max().unwrap_or(&1);
    let bar_width = chart_width as f64 / num_bins as f64;

    // Draw axes
    // X-axis
    document = document.add(Line::new()
        .set("x1", x_offset)
        .set("y1", y_offset + chart_height)
        .set("x2", x_offset + chart_width)
        .set("y2", y_offset + chart_height)
        .set("stroke", "black")
        .set("stroke-width", 2));

    // Y-axis
    document = document.add(Line::new()
        .set("x1", x_offset)
        .set("y1", y_offset)
        .set("x2", x_offset)
        .set("y2", y_offset + chart_height)
        .set("stroke", "black")
        .set("stroke-width", 2));

    // Draw histogram bars
    for (i, &count) in bins.iter().enumerate() {
        if count > 0 {
            let bar_height = (count as f64 / max_count as f64) * chart_height as f64;
            let x = x_offset as f64 + i as f64 * bar_width;
            let y = y_offset as f64 + chart_height as f64 - bar_height;

            document = document.add(Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", bar_width - 1.0) // Small gap between bars
                .set("height", bar_height)
                .set("fill", "#4CAF50")
                .set("stroke", "#2E7D32")
                .set("stroke-width", 1));
        }
    }

    // Add axis labels
    document = document.add(Text::new("Value")
        .set("x", x_offset + chart_width / 2)
        .set("y", y_offset + chart_height + 40)
        .set("text-anchor", "middle")
        .set("font-size", "12")
        .set("fill", "black"));

    document = document.add(Text::new("Frequency")
        .set("x", 20)
        .set("y", y_offset + chart_height / 2)
        .set("text-anchor", "middle")
        .set("font-size", "12")
        .set("fill", "black")
        .set("transform", format!("rotate(-90, 20, {})", y_offset + chart_height / 2)));

    // Add scale labels
    let num_labels = 5;
    for i in 0..=num_labels {
        let value = min_val + (max_val - min_val) * i as f64 / num_labels as f64;
        let x = x_offset as f64 + (chart_width as f64 * i as f64 / num_labels as f64);
        
        document = document.add(Text::new(format!("{:.2}", value))
            .set("x", x)
            .set("y", y_offset + chart_height + 20)
            .set("text-anchor", "middle")
            .set("font-size", "10")
            .set("fill", "black"));
    }

    Ok(document)
}

/// Create empty message SVG
fn create_empty_message(message: &str) -> String {
    let width = 400;
    let height = 200;

    Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .add(Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "white"))
        .add(Text::new(message)
            .set("x", width / 2)
            .set("y", height / 2)
            .set("text-anchor", "middle")
            .set("font-size", "16")
            .set("fill", "gray"))
        .to_string()
}