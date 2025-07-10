// src/visualization/plots.rs
use anyhow::Result;

use crate::analysis::AnalysisResults;

/// Create a histogram plot description (simplified)
pub fn create_histogram_plot(data: &[f64], title: &str) -> Result<String> {
    if data.is_empty() {
        return Ok("No data to plot".to_string());
    }

    let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max_val - min_val).abs() < f64::EPSILON {
        return Ok("All values are the same".to_string());
    }

    // Create histogram bins
    let num_bins = 30;
    let bin_width = (max_val - min_val) / num_bins as f64;
    let mut bins = vec![0usize; num_bins];
    
    for &value in data {
        let bin_index = ((value - min_val) / bin_width).floor() as usize;
        let bin_index = bin_index.min(num_bins - 1);
        bins[bin_index] += 1;
    }

    Ok(format!("Histogram for '{}': {} bins, range {:.3} to {:.3}", title, num_bins, min_val, max_val))
}

/// Create a normal distribution overlay
pub fn create_normal_overlay(results: &AnalysisResults) -> Result<Vec<(f64, f64)>> {
    let mean = results.mean;
    let std_dev = results.std_dev;
    
    let min_x = mean - 4.0 * std_dev;
    let max_x = mean + 4.0 * std_dev;
    let num_points = 100;
    let step = (max_x - min_x) / num_points as f64;
    
    let mut points = Vec::new();
    
    for i in 0..=num_points {
        let x = min_x + i as f64 * step;
        let y = normal_pdf(x, mean, std_dev);
        points.push((x, y));
    }
    
    Ok(points)
}

/// Normal probability density function
fn normal_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let coefficient = 1.0 / (std_dev * (2.0 * std::f64::consts::PI).sqrt());
    let exponent = -0.5 * ((x - mean) / std_dev).powi(2);
    coefficient * exponent.exp()
}

/// Create a scatter plot description
pub fn create_scatter_plot(x_data: &[f64], y_data: &[f64], title: &str) -> Result<String> {
    if x_data.len() != y_data.len() {
        return Err(anyhow::anyhow!("X and Y data must have the same length"));
    }

    if x_data.is_empty() {
        return Ok("No data to plot".to_string());
    }

    Ok(format!("Scatter plot for '{}' with {} points", title, x_data.len()))
}

/// Create a line plot description
pub fn create_line_plot(data: &[(f64, f64)], title: &str) -> Result<String> {
    if data.is_empty() {
        return Ok("No data to plot".to_string());
    }

    Ok(format!("Line plot for '{}' with {} points", title, data.len()))
}

/// Create a box plot representation (simplified)
pub fn create_box_plot_data(data: &[f64]) -> Result<BoxPlotData> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("No data for box plot"));
    }

    let mut sorted_data = data.to_vec();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted_data.len();
    let q1_index = n / 4;
    let median_index = n / 2;
    let q3_index = 3 * n / 4;

    let q1 = sorted_data[q1_index];
    let median = sorted_data[median_index];
    let q3 = sorted_data[q3_index];
    let iqr = q3 - q1;

    let min = sorted_data[0];
    let max = sorted_data[n - 1];

    // Calculate outliers (values beyond 1.5 * IQR from quartiles)
    let lower_fence = q1 - 1.5 * iqr;
    let upper_fence = q3 + 1.5 * iqr;

    let outliers: Vec<f64> = sorted_data.iter()
        .filter(|&&x| x < lower_fence || x > upper_fence)
        .copied()
        .collect();

    Ok(BoxPlotData {
        min,
        q1,
        median,
        q3,
        max,
        outliers,
    })
}

/// Box plot data structure
#[derive(Debug)]
pub struct BoxPlotData {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub outliers: Vec<f64>,
}