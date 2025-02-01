// src/ui/analysis.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Clear},
    Frame,
};
use chrono::DateTime;
use crate::app::App;
use crate::state::ui_state::{DialogMode, AnalysisTab};
use crate::analysis::stackup::{AnalysisMethod, MonteCarloResult, StackupAnalysis, AnalysisResults, StackupContribution};
use crate::state::input_state::{InputMode, EditField, ContributionSelectionState};
use crate::config::Component;

pub fn draw_analysis(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Analysis mode tabs
            Constraint::Min(0),    // Content area
        ])
        .split(area);

    draw_analysis_tabs(frame, app, chunks[0]);
    
    match app.state().ui.analysis_tab {
        AnalysisTab::List => draw_analysis_list(frame, app, chunks[1]),
        AnalysisTab::Details => draw_analysis_details(frame, app, chunks[1]),
        AnalysisTab::Results => draw_analysis_results(frame, app, chunks[1]),
        AnalysisTab::Visualization => draw_visualization(frame, app, chunks[1]),
    }

    // Handle dialogs
    match app.state().ui.dialog_mode {
        DialogMode::AddAnalysis | DialogMode::EditAnalysis => {
            let area = centered_rect(60, 80, frame.size());
            draw_analysis_dialog(frame, app, area);
        },
        DialogMode::AddContribution | DialogMode::EditContribution => {
            draw_contribution_dialog(frame, app);
        },
        _ => {}
    }
}


fn draw_analysis_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    
    let current_analysis = if let Some(idx) = state.ui.analysis_list_state.selected() {
        if let Some(analysis) = state.analysis.analyses.get(idx) {
            format!(" - {}", analysis.name)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let titles = vec!["Analyses", "Details", "Results", "Visualization"];
    let tabs = Tabs::new(titles)
        .block(Block::default()
            .title(format!("Analysis{}", current_analysis))
            .borders(Borders::ALL))
        .select(state.ui.analysis_tab as usize)
        .highlight_style(Style::default().fg(Color::Yellow));
    
    frame.render_widget(tabs, area);
}

fn draw_analysis_list(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    
    let items: Vec<ListItem> = state.analysis.analyses
        .iter()
        .map(|analysis| {
            let methods: String = analysis.methods
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ");

            let summary = format!(
                "{} ({} contributions, methods: {})",
                analysis.name,
                analysis.contributions.len(),
                methods
            );

            ListItem::new(summary)
        })
        .collect();

    let analysis_block = Block::default()
        .title("Analyses (a: add, e: edit, d: delete)")
        .borders(Borders::ALL);

    let analysis_list = List::new(items)
        .block(analysis_block)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    frame.render_stateful_widget(
        analysis_list,
        area,
        &mut state.ui.analysis_list_state.clone(),
    );
}

fn draw_analysis_details(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    
    if let Some(selected) = state.ui.analysis_list_state.selected() {
        if let Some(analysis) = state.analysis.analyses.get(selected) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Name
                    Constraint::Length(3), // Methods
                    Constraint::Min(0),    // Contributions
                ])
                .split(area);

            // Analysis name
            let name_block = Block::default()
                .title("Name")
                .borders(Borders::ALL);
            frame.render_widget(
                Paragraph::new(analysis.name.as_str()).block(name_block),
                chunks[0]
            );

            // Analysis methods
            let methods: String = analysis.methods
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ");
            let methods_block = Block::default()
                .title("Methods")
                .borders(Borders::ALL);
            frame.render_widget(
                Paragraph::new(methods).block(methods_block),
                chunks[1]
            );

            // Contributions list
            let contributions: Vec<ListItem> = state.input.analysis_inputs.contributions
                                                                          .iter()
                                                                          .map(|contrib| {
                                                                              let direction = if contrib.direction > 0.0 { "+" } else { "-" };
                                                                              let half = if contrib.half_count { " (½)" } else { "" };

                                                                              let feature_info = state.project.components
                                                                                                              .iter()
                                                                                                              .find(|c| c.name == contrib.component_id)
                                                                                                              .and_then(|c| c.features.iter().find(|f| f.name == contrib.feature_id));

                                                                              let base_text = format!(
                                                                                  "{}.{} [{}]{}",
                                                                                  contrib.component_id,
                                                                                  contrib.feature_id,
                                                                                  direction,
                                                                                  half,
                                                                              );

                                                                              let feature_text = if let Some(feature) = feature_info {
                                                                                  format!(
                                                                                      "{}\n    Nominal: {:.3} [{:+.3}/{:+.3}]",
                                                                                      base_text,
                                                                                      feature.dimension.value,
                                                                                      feature.dimension.plus_tolerance,
                                                                                      feature.dimension.minus_tolerance
                                                                                  )
                                                                              } else {
                                                                                  base_text
                                                                              };

                                                                              // Add distribution info
                                                                              let full_text = if let Some(dist) = &contrib.distribution {
                                                                                  format!("{}\n    Distribution: {:?}", feature_text, dist.dist_type)
                                                                              } else {
                                                                                  feature_text
                                                                              };

                                                                              ListItem::new(full_text)
                                                                          })
                                                                          .collect();

            let contributions_block = Block::default()
                .title("Contributions (e: edit)")
                .borders(Borders::ALL);

            let contributions_list = List::new(contributions)
                .block(contributions_block)
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("► ");

            frame.render_stateful_widget(
                contributions_list,
                chunks[2],
                &mut state.ui.contribution_list_state.clone(),
            );
        }
    } else {
        let block = Block::default()
            .title("Analysis Details")
            .borders(Borders::ALL);
        frame.render_widget(
            Paragraph::new("Select an analysis to view details")
                .block(block),
            area
        );
    }
}

fn draw_analysis_results(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    
    if let Some(selected) = state.ui.analysis_list_state.selected() {
        if let Some(analysis) = state.analysis.analyses.get(selected) {
            if let Some(results) = state.analysis.latest_results.get(&analysis.id) {
                // Split screen horizontally
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Ratio(1, 2),  // Left side: Main results
                        Constraint::Ratio(1, 2),  // Right side: Sensitivity
                    ])
                    .split(area);

                // Left side results
                let left_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // Timestamp
                        Constraint::Length(3),  // Nominal value
                        Constraint::Length(8),  // Worst case results
                        Constraint::Length(8),  // RSS results
                        Constraint::Min(0),     // Monte Carlo results
                    ])
                    .split(chunks[0]);

                // Draw timestamp
                let timestamp = DateTime::parse_from_rfc3339(&results.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|_| "Invalid timestamp".to_string());
                
                let timestamp_block = Block::default()
                    .title("Last Run")
                    .borders(Borders::ALL);
                frame.render_widget(
                    Paragraph::new(timestamp).block(timestamp_block),
                    left_chunks[0]
                );

                // Nominal value
                let nominal_block = Block::default()
                    .title("Nominal Value")
                    .borders(Borders::ALL);
                frame.render_widget(
                    Paragraph::new(format!("{:.6}", results.nominal))
                        .block(nominal_block),
                    left_chunks[1]
                );

                // Draw worst case results
                if let Some(wc) = &results.worst_case {
                    let wc_text = format!(
                        "Min: {:.6}\nMax: {:.6}\nRange: {:.6}",
                        wc.min,
                        wc.max,
                        wc.max - wc.min
                    );
                    let wc_block = Block::default()
                        .title("Worst Case Analysis")
                        .borders(Borders::ALL);
                    frame.render_widget(
                        Paragraph::new(wc_text).block(wc_block),
                        left_chunks[2]
                    );
                }

                // Draw RSS results
                if let Some(rss) = &results.rss {
                    let rss_text = format!(
                        "Mean: {:.6}\nMin (3σ): {:.6}\nMax (3σ): {:.6}\nStd Dev: {:.6}",
                        results.nominal,
                        rss.min,
                        rss.max,
                        rss.std_dev
                    );
                    let rss_block = Block::default()
                        .title("RSS Analysis")
                        .borders(Borders::ALL);
                    frame.render_widget(
                        Paragraph::new(rss_text).block(rss_block),
                        left_chunks[3]
                    );
                }

                // Draw Monte Carlo results
                if let Some(mc) = &results.monte_carlo {
                    let mut mc_text = format!(
                        "Statistical Summary:\n\
                         Mean: {:.6}\n\
                         Std Dev: {:.6}\n\
                         Min: {:.6}\n\
                         Max: {:.6}\n\n\
                         Confidence Intervals:\n",
                        mc.mean,
                        mc.std_dev,
                        mc.min,
                        mc.max,
                    );
                
                    // Sort intervals by confidence level
                    let mut intervals = mc.confidence_intervals.clone();
                    intervals.sort_by(|a, b| a.confidence_level.partial_cmp(&b.confidence_level).unwrap_or(std::cmp::Ordering::Equal));
                
                    for interval in intervals {
                        mc_text.push_str(&format!(
                            "{:.1}%: [{:.6}, {:.6}]\n",
                            interval.confidence_level * 100.0,
                            interval.lower_bound,
                            interval.upper_bound,
                        ));
                    }
                
                    let mc_block = Block::default()
                        .title("Monte Carlo Analysis")
                        .borders(Borders::ALL);
                    frame.render_widget(
                        Paragraph::new(mc_text).block(mc_block),
                        left_chunks[4]
                    );
                }

                // Right side: Sensitivity Analysis
                let mut sensitivity_text = String::new();
                
                if let Some(wc) = &results.worst_case {
                    sensitivity_text.push_str("Worst Case Sensitivities:\n");
                    for sens in &wc.sensitivity {
                        sensitivity_text.push_str(&format!(
                            "   {}.{}: {:.1}% contribution\n",
                            sens.component_id,
                            sens.feature_id,
                            sens.contribution_percent
                        ));
                    }
                    sensitivity_text.push('\n');
                }

                if let Some(rss) = &results.rss {
                    sensitivity_text.push_str("RSS Sensitivities:\n");
                    for sens in &rss.sensitivity {
                        sensitivity_text.push_str(&format!(
                            "   {}.{}: {:.1}% contribution\n",
                            sens.component_id,
                            sens.feature_id,
                            sens.contribution_percent
                        ));
                    }
                    sensitivity_text.push('\n');
                }

                if let Some(mc) = &results.monte_carlo {
                    sensitivity_text.push_str("Monte Carlo Sensitivities:\n");
                    for sens in &mc.sensitivity {
                        sensitivity_text.push_str(&format!(
                            "   {}.{}: {:.1}% contribution (correlation: {:.3})\n",
                            sens.component_id,
                            sens.feature_id,
                            sens.contribution_percent,
                            sens.correlation.unwrap_or(0.0)
                        ));
                    }
                }

                let sensitivity_block = Block::default()
                    .title("Sensitivity Analysis")
                    .borders(Borders::ALL);
                frame.render_widget(
                    Paragraph::new(sensitivity_text).block(sensitivity_block),
                    chunks[1]
                );
            }
        }
    }
}


fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_analysis_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let area = centered_rect(60, 80, frame.size());
    let state = app.state();

    let title = match state.ui.dialog_mode {
        DialogMode::AddAnalysis => "Add Analysis",
        DialogMode::EditAnalysis => "Edit Analysis",
        _ => unreachable!(),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Name
            Constraint::Length(6),  // Analysis methods
            //Constraint::Length(3),  // Monte Carlo settings header
            Constraint::Length(5),  // Monte Carlo settings
            Constraint::Min(0),     // Contributions section
            //Constraint::Length(3),  // Instructions
        ])
        .split(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Name input
    let name_style = match state.input.mode {
        InputMode::Editing(EditField::AnalysisName) => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };
    let name_block = Block::default()
        .title("Name (n: edit)")
        .borders(Borders::ALL);
    let name_widget = Paragraph::new(state.input.analysis_inputs.name.value())
        .style(name_style)
        .block(name_block);
    frame.render_widget(name_widget, chunks[0]);

    // Analysis methods
    let methods_block = Block::default()
        .title("Analysis Methods (space: toggle, J/K: select method)")
        .borders(Borders::ALL);
    
    let method_items: Vec<ListItem> = [
        AnalysisMethod::WorstCase,
        AnalysisMethod::Rss,
        AnalysisMethod::MonteCarlo,
    ].iter().map(|method| {
        let prefix = if state.input.analysis_inputs.methods.contains(method) {
            "[x] "
        } else {
            "[ ] "
        };
        ListItem::new(format!("{}{:?}", prefix, method))
    }).collect();

    let methods_list = List::new(method_items)
        .block(methods_block)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    frame.render_stateful_widget(
        methods_list,
        chunks[1],
        &mut state.ui.method_list_state.clone(),
    );

    let mc_settings = &state.input.analysis_inputs.monte_carlo_settings;
    let mc_content = vec![
        Line::from(highlight_line(
            format!("Iterations: {} (i)", mc_settings.iterations),
            EditField::MonteCarloIterations,
            state.input.mode
        )),
        Line::from(highlight_line(
            format!("Confidence Level: {:.2}% (f)", mc_settings.confidence * 100.0), // Display as percentage
            EditField::MonteCarloConfidence,
            state.input.mode
        )),
        Line::from(highlight_line(
            format!("Seed: {} (x)", mc_settings.seed.map_or("None".to_string(), |s| s.to_string())),
            EditField::MonteCarloSeed,
            state.input.mode
        )),
    ];

    let mc_settings_block = Block::default()
        .title("Monte Carlo Settings")
        .borders(Borders::ALL);
    frame.render_widget(Paragraph::new(mc_content).block(mc_settings_block), chunks[2]);

    // Contributions header
    let contrib_header = Paragraph::new("")
        .block(Block::default()
            .title("Contributions (c: add, e: edit, d: delete)")
            .borders(Borders::ALL));
    frame.render_widget(contrib_header, chunks[3]);

    // Contributions section
    let contributions_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Min(0),     // List
        ])
        .split(chunks[3]);

    frame.render_widget(
        Paragraph::new("Contributions (c: add, e: edit, d: delete, j/k: select contribution)"),
        contributions_chunks[0]
    );

    let contributions: Vec<ListItem> = state.input.analysis_inputs.contributions
        .iter()
        .map(|contrib| {
            let direction = if contrib.direction > 0.0 { "+" } else { "-" };
            let half = if contrib.half_count { " (½)" } else { "" };
            
            // Get the actual feature information
            let feature_info = state.project.components
                .iter()
                .find(|c| c.name == contrib.component_id)
                .and_then(|c| c.features.iter().find(|f| f.name == contrib.feature_id));

            let base_text = format!(
                "{}.{} [{}]{}", 
                contrib.component_id,
                contrib.feature_id,
                direction,
                half,
            );

            let feature_text = if let Some(feature) = feature_info {
                format!(
                    "{}\n    Nominal: {:.3} [{:+.3}/{:+.3}]",
                    base_text,
                    feature.dimension.value,
                    feature.dimension.plus_tolerance,
                    feature.dimension.minus_tolerance
                )
            } else {
                base_text
            };

            // Add distribution info if present
            let full_text = if let Some(dist) = &contrib.distribution {
                format!("{}\n    Distribution: {:?}", feature_text, dist.dist_type)
            } else {
                feature_text
            };

            ListItem::new(full_text)
        })
        .collect();

    let contributions_list = List::new(contributions)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    frame.render_stateful_widget(
        contributions_list,
        contributions_chunks[1],
        &mut state.ui.contribution_list_state.clone(),
    );



    // Instructions
    //let instructions = vec![
    //    Line::from(vec![
    //        Span::raw("J/K: select method, j/k: select contribution, space: toggle method, "),
    //        Span::styled("i/f/x", Style::default().add_modifier(Modifier::BOLD)),
    //        Span::raw(": MC settings, "),
    //        Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
    //        Span::raw(": add contribution, "),
    //        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
    //        Span::raw("/"),
    //        Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
    //        Span::raw(": edit/delete contribution"),
    //    ]),
    //];
    //frame.render_widget(Paragraph::new(instructions), chunks[4]);
}

fn highlight_line(text: String, field: EditField, current_mode: InputMode) -> Line<'static> {
    match current_mode {
        InputMode::Editing(edit_field) if edit_field == field =>
            Line::from(vec![Span::styled(text, Style::default().fg(Color::Yellow))]),
        _ => Line::from(text),
    }
}

// Add the contribution dialog
pub fn draw_contribution_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.size());
    let state = app.state();

    let block = Block::default()
        .title("Add Contribution")
        .borders(Borders::ALL);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Status
            Constraint::Min(10),    // Selection List
            Constraint::Length(3),  // Direction/Half Count
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Status
    let status_text = match state.input.contribution_selection_state {
        ContributionSelectionState::SelectingComponent => "Select Component",
        ContributionSelectionState::SelectingFeature => "Select Feature",
    };
    let status = Block::default()
        .title("Status")
        .borders(Borders::ALL);
    frame.render_widget(Paragraph::new(status_text).block(status), chunks[0]);

    // Selection list
    match state.input.contribution_selection_state {
        ContributionSelectionState::SelectingComponent => {
            let items: Vec<ListItem> = state.project.components
                .iter()
                .map(|c| ListItem::new(c.name.clone()))
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Components").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("► ");

            frame.render_stateful_widget(list, chunks[1], &mut state.ui.component_list_state.clone());
        },
        ContributionSelectionState::SelectingFeature => {
            if let Some(comp_idx) = state.ui.component_list_state.selected() {
                if let Some(component) = state.project.components.get(comp_idx) {
                    let items: Vec<ListItem> = component.features
                        .iter()
                        .map(|f| {
                            ListItem::new(format!(
                                "{} ({:.3} [{:+.3}/{:+.3}])",
                                f.name,
                                f.dimension.value,
                                f.dimension.plus_tolerance,
                                f.dimension.minus_tolerance
                            ))
                        })
                        .collect();

                    let list = List::new(items)
                        .block(Block::default().title("Features").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::DarkGray))
                        .highlight_symbol("► ");

                    frame.render_stateful_widget(list, chunks[1], &mut state.ui.feature_list_state.clone());
                }
            }
        }
    }

    // Direction and half count
    let options = format!(
        "Direction: {} (d) | Half Count: {} (h) | Distribution: {:?}",
        if state.input.contribution_inputs.direction > 0.0 { "+" } else { "-" },
        if state.input.contribution_inputs.half_count { "Yes" } else { "No" },
        state.input.contribution_inputs.distribution_type  // Reference the distribution_type directly
    );
    let options_block = Block::default()
        .title("Options")
        .borders(Borders::ALL);
    frame.render_widget(Paragraph::new(options).block(options_block), chunks[2]);

    // Instructions
    let instructions = vec![
        Line::from(vec![
            Span::raw("Use "),
            Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to navigate, "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to select, "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to cancel"),
        ]),
    ];
    frame.render_widget(Paragraph::new(instructions), chunks[3]);
}

fn draw_histogram(frame: &mut Frame, mc_results: &MonteCarloResult, area: Rect) {
    let block = Block::default()
        .title("Monte Carlo Distribution (red line = mean)")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if mc_results.histogram.is_empty() {
        return;
    }

    let max_freq = mc_results.histogram.iter()
        .map(|(_, freq)| *freq)
        .max()
        .unwrap_or(1);

    // Calculate available height and width
    let height = inner.height.saturating_sub(4) as usize; // Leave room for x-axis labels
    let x_margin = 6; // Margin for y-axis labels
    let usable_width = inner.width.saturating_sub(2 * x_margin) as usize; // Full width minus margins
    
    // Scale frequencies to available height and distribute bars across full width
    let bar_positions: Vec<(u16, f64, usize)> = mc_results.histogram.iter()
        .enumerate()
        .map(|(i, (val, freq))| {
            let scaled_height = ((*freq as f64 / max_freq as f64) * height as f64) as usize;
            let x_pos = inner.left() + x_margin + (i * usable_width / mc_results.histogram.len()) as u16;
            (x_pos, *val, scaled_height.min(height))
        })
        .collect();

    // Draw histogram bars and x-axis labels
    let label_interval = (bar_positions.len() / 6).max(1); // Show ~6 labels across the axis
    for (i, (x_pos, val, bar_height)) in bar_positions.iter().enumerate() {
        // Draw vertical bar
        for y in 0..*bar_height {
            let y_pos = inner.bottom().saturating_sub(4) - y as u16;
            
            if y_pos >= inner.top() && *x_pos < inner.right() {
                frame.buffer_mut().get_mut(*x_pos, y_pos)
                    .set_char('█');
            }
        }

        // Draw x-axis labels at intervals
        if i % label_interval == 0 {
            let label = format!("{:.3}", val);
            frame.buffer_mut().set_stringn(
                x_pos.saturating_sub(3),
                inner.bottom().saturating_sub(2),
                label,
                6,
                Style::default(),
            );
        }
    }

    // Draw mean line with correct positioning
    let mean_x = inner.left() + x_margin + 
        (((mc_results.mean - mc_results.min) / (mc_results.max - mc_results.min) * usable_width as f64) as u16);
    
    for y in 0..height {
        let y_pos = inner.bottom().saturating_sub(4) - y as u16;
        if y_pos >= inner.top() && mean_x < inner.right() {
            frame.buffer_mut().get_mut(mean_x, y_pos)
                .set_char('│')
                .set_style(Style::default().fg(Color::Red));
        }
    }

    // Draw mean value label
    let mean_label = format!("μ={:.3}", mc_results.mean);
    frame.buffer_mut().set_stringn(
        mean_x.saturating_sub(3),
        inner.bottom().saturating_sub(1),
        mean_label,
        8,
        Style::default().fg(Color::Red),
    );

    // Add y-axis labels
    let y_label_interval = height / 4;
    for i in 0..=4 {
        let freq_label = format!("{:>4}", (max_freq as f64 * i as f64 / 4.0) as usize);
        frame.buffer_mut().set_stringn(
            inner.left() + 1,
            inner.bottom().saturating_sub(4) - (i * y_label_interval) as u16,
            freq_label,
            4,
            Style::default(),
        );
    }
}

fn draw_waterfall(frame: &mut Frame, analysis: &StackupAnalysis, components: &[Component], results: &AnalysisResults, area: Rect) {
    let block = Block::default()
        .title("Contribution Waterfall")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if analysis.contributions.is_empty() {
        return;
    }

    let mut running_total = 0.0;
    let mut y_position = inner.top();
    let bar_width = inner.width.saturating_sub(30) as usize;

    // Get feature values and find the max value for scaling
    let mut contributions_with_values: Vec<(f64, &StackupContribution)> = Vec::new();
    let mut max_value = 0.0f64;

    // First pass - collect values and find max
    for contribution in &analysis.contributions {
        let value = contribution.direction * 
            (if let Some(feat) = analysis.get_feature(components, contribution) {
                feat.dimension.value * if contribution.half_count { 0.5 } else { 1.0 }
            } else {
                0.0
            });
        
        if value.abs() > max_value {
            max_value = value.abs();
        }
        
        contributions_with_values.push((value, contribution));
    }

    // Draw starting point
    let label = format!("Start: {:.3}", 0.0);
    frame.buffer_mut().set_stringn(
        inner.left(),
        y_position,
        label,
        inner.width as usize,
        Style::default(),
    );
    y_position += 1;

    // Draw each contribution
    for (value, contribution) in contributions_with_values {
        if y_position >= inner.bottom() {
            break;
        }

        running_total += value;
        
        // Draw contribution label with nominal value
        let label = format!("{}.{}: {:.3}", 
            contribution.component_id,
            contribution.feature_id,
            value
        );
        frame.buffer_mut().set_stringn(
            inner.left(),
            y_position,
            label,
            30,
            Style::default(),
        );

        // Scale bar length relative to max absolute value
        let scaled_length = if max_value > 0.0 {
            ((value.abs() / max_value) * bar_width as f64) as usize
        } else {
            0
        };
        
        let bar_char = '█';
        let bar_style = if value >= 0.0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        // Draw the bar
        for x in 0..scaled_length {
            let x_pos = inner.left() + 30 + x as u16;
            if x_pos < inner.right() {
                frame.buffer_mut().get_mut(x_pos, y_position)
                    .set_char(bar_char)
                    .set_style(bar_style);
            }
        }

        // Draw running total
        let total_label = format!("= {:.3}", running_total);
        frame.buffer_mut().set_stringn(
            inner.right().saturating_sub(15),
            y_position,
            total_label,
            15,
            Style::default().fg(Color::Yellow),
        );

        y_position += 1;
    }
}

// Update the visualization tab drawing:
fn draw_visualization(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    
    if let Some(selected) = state.ui.analysis_list_state.selected() {
        if let Some(analysis) = state.analysis.analyses.get(selected) {
            if let Some(results) = state.analysis.latest_results.get(&analysis.id) {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Ratio(1, 2),
                        Constraint::Ratio(1, 2),
                    ])
                    .split(area);

                if let Some(mc_results) = &results.monte_carlo {
                    draw_histogram(frame, mc_results, chunks[0]);
                }

                draw_waterfall(frame, analysis, &state.project.components, results, chunks[1]);
            } else {
                let block = Block::default()
                    .title("Analysis Visualization")
                    .borders(Borders::ALL);
                frame.render_widget(
                    Paragraph::new("Run analysis to see visualizations")
                        .block(block),
                    area
                );
            }
        }
    } else {
        let block = Block::default()
            .title("Analysis Visualization")
            .borders(Borders::ALL);
        frame.render_widget(
            Paragraph::new("Select an analysis to view visualizations")
                .block(block),
            area
        );
    }
}
