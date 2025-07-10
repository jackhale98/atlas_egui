// src/prompts/utils.rs
use anyhow::Result;
use inquire::{Select, Text};
use console::style;

/// Display a formatted table of data
pub fn display_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    // Calculate column widths
    let mut widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
    
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    print!("│");
    for (i, header) in headers.iter().enumerate() {
        print!(" {:width$} │", style(header).bold(), width = widths[i]);
    }
    println!();

    // Print separator
    print!("├");
    for (i, &width) in widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < widths.len() - 1 {
            print!("┼");
        }
    }
    println!("┤");

    // Print rows
    for row in rows {
        print!("│");
        for (i, cell) in row.iter().enumerate() {
            let width = widths.get(i).unwrap_or(&10);
            print!(" {:width$} │", cell, width = width);
        }
        println!();
    }
}

/// Display a progress indicator
pub fn show_progress(current: usize, total: usize, message: &str) {
    let percentage = (current as f64 / total as f64 * 100.0) as usize;
    let bar_length = 30;
    let filled = (percentage * bar_length) / 100;
    let empty = bar_length - filled;
    
    print!("\r{} [{}{}] {}% ({}/{})", 
           message,
           "█".repeat(filled),
           "░".repeat(empty),
           percentage,
           current,
           total);
    
    if current == total {
        println!(); // New line when complete
    }
}

/// Format a floating point number for display
pub fn format_number(value: f64, precision: usize) -> String {
    format!("{:.precision$}", value, precision = precision)
}

/// Format a tolerance for display
pub fn format_tolerance(nominal: f64, plus: f64, minus: f64) -> String {
    if (plus - minus).abs() < f64::EPSILON {
        format!("{:.3} ± {:.3}", nominal, plus)
    } else {
        format!("{:.3} +{:.3}/-{:.3}", nominal, plus, minus)
    }
}

/// Create a simple menu selection
pub fn simple_menu(title: &str, options: &[&str]) -> Result<usize> {
    println!("\n{}", style(title).cyan().bold());
    println!("{}", "─".repeat(title.len()));
    
    for (i, option) in options.iter().enumerate() {
        println!("{}. {}", i + 1, option);
    }
    
    loop {
        let input = Text::new("Select option (number):")
            .prompt()?;
        
        if let Ok(choice) = input.parse::<usize>() {
            if choice > 0 && choice <= options.len() {
                return Ok(choice - 1);
            }
        }
        
        println!("❌ Invalid selection. Please enter a number between 1 and {}", options.len());
    }
}

/// Display a warning message
pub fn show_warning(message: &str) {
    println!("{} {}", style("⚠️ ").yellow(), style(message).yellow());
}

/// Display an error message
pub fn show_error(message: &str) {
    println!("{} {}", style("❌").red(), style(message).red());
}

/// Display a success message
pub fn show_success(message: &str) {
    println!("{} {}", style("✅").green(), style(message).green());
}

/// Display an info message
pub fn show_info(message: &str) {
    println!("{} {}", style("ℹ️ ").blue(), style(message).blue());
}

/// Format bytes for display
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Create a confirmation prompt with custom message
pub fn confirm_with_default(message: &str, default: bool) -> Result<bool> {
    inquire::Confirm::new(message)
        .with_default(default)
        .prompt()
        .map_err(Into::into)
}

/// Prompt for numeric input with validation
pub fn prompt_number<T>(message: &str, default: Option<T>) -> Result<T> 
where 
    T: std::str::FromStr + std::fmt::Display + Clone,
    T::Err: std::fmt::Display,
{
    let mut prompt = inquire::CustomType::new(message);
    
    if let Some(def) = default {
        prompt = prompt.with_default(def);
    }
    
    prompt.prompt().map_err(Into::into)
}