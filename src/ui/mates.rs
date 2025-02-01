// src/ui/mates.rs
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};
use crate::app::App;
use crate::state::ui_state::DialogMode;
use crate::config::mate::FitValidation;
use crate::config::Feature;
use crate::state::AppState;
use crate::state::mate_state::{get_component_by_name, MateFilter};
use crate::state::input_state::MateSelectionState;
use crate::config::Component;


pub fn draw_mates(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    draw_mate_list(frame, app, chunks[0]);
    draw_mate_details(frame, app, chunks[1]);

    if matches!(app.state().ui.dialog_mode, DialogMode::AddMate | DialogMode::EditMate) {
        draw_mate_dialog(frame, app);
    }
}

// src/ui/mates.rs
fn draw_mate_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.size());
    let state = app.state();

    let title = match state.ui.dialog_mode {
        DialogMode::AddMate => "Add Mate",
        DialogMode::EditMate => "Edit Mate",
        _ => unreachable!(),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Current Selection Status
            Constraint::Min(10),    // Selection List
            Constraint::Length(3),  // Current Selections / Fit Type
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Show current selection state
    let status_text = match state.input.mate_selection_state {
        MateSelectionState::SelectingComponentA => "Select First Component",
        MateSelectionState::SelectingFeatureA => "Select First Feature",
        MateSelectionState::SelectingComponentB => "Select Second Component",
        MateSelectionState::SelectingFeatureB => "Select Second Feature",
    };

    let status_block = Block::default()
        .title("Status")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(status_text).block(status_block),
        chunks[0]
    );

    // Draw the appropriate selection list
    match state.input.mate_selection_state {
        MateSelectionState::SelectingComponentA | MateSelectionState::SelectingComponentB => {
            draw_component_selection_list(frame, app, chunks[1]);
        },
        MateSelectionState::SelectingFeatureA => {
            if let Some(comp_a) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_a.value()) {
                draw_feature_selection_list(frame, comp_a, app, chunks[1]);
            }
        },
        MateSelectionState::SelectingFeatureB => {
            if let Some(comp_b) = get_component_by_name(&state.project.components, &state.input.mate_inputs.component_b.value()) {
                draw_feature_selection_list(frame, comp_b, app, chunks[1]);
            }
        },
    }

    // Show current selections and fit type
    let selections = format!(
        "A: {}.{} ↔ B: {}.{} | Fit Type: {:?} (press 't' to toggle)",
        state.input.mate_inputs.component_a.value(),
        state.input.mate_inputs.feature_a.value(),
        state.input.mate_inputs.component_b.value(),
        state.input.mate_inputs.feature_b.value(),
        state.input.mate_inputs.fit_type,
    );

    let preview_block = Block::default()
        .title("Current Selections")
        .borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(selections).block(preview_block),
        chunks[2]
    );

    // Instructions
    let instructions = vec![
        Line::from(vec![
            Span::raw("Use "),
            Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to navigate, "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to select, "),
            Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to toggle fit type, "),
            Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to save, "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to cancel"),
        ]),
    ];
    frame.render_widget(Paragraph::new(instructions), chunks[3]);
}


fn draw_mate_details(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    let mate_block = Block::default()
        .title("Mate Details")
        .borders(Borders::ALL);

    if let Some(selected) = state.ui.mate_list_state.selected() {
        if let Some(mate) = state.mates.mates.get(selected) {
            // Find the relevant features
            let feature_a = find_feature(state, &mate.component_a, &mate.feature_a);
            let feature_b = find_feature(state, &mate.component_b, &mate.feature_b);

            if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(4),  // Component/Feature A info
                        Constraint::Length(4),  // Component/Feature B info
                        Constraint::Length(3),  // Fit Type
                        Constraint::Length(3),  // Nominal Fit
                        Constraint::Length(3),  // Min Fit
                        Constraint::Length(3),  // Max Fit
                        Constraint::Length(3),  // Validation Status
                    ])
                    .split(area);

                frame.render_widget(mate_block, area);

                // Feature A info
                let comp_a = format!(
                    "{} - {} ({:?})\nNominal: {:.3} [{:+.3}/{:+.3}]",
                    mate.component_a,
                    mate.feature_a,
                    feat_a.feature_type,
                    feat_a.dimension.value,
                    feat_a.dimension.plus_tolerance,
                    feat_a.dimension.minus_tolerance
                );
                let comp_a_widget = Paragraph::new(comp_a)
                    .block(Block::default().title("Component/Feature A").borders(Borders::ALL));
                frame.render_widget(comp_a_widget, chunks[0]);

                // Feature B info
                let comp_b = format!(
                    "{} - {} ({:?})\nNominal: {:.3} [{:+.3}/{:+.3}]",
                    mate.component_b,
                    mate.feature_b,
                    feat_b.feature_type,
                    feat_b.dimension.value,
                    feat_b.dimension.plus_tolerance,
                    feat_b.dimension.minus_tolerance
                );
                let comp_b_widget = Paragraph::new(comp_b)
                    .block(Block::default().title("Component/Feature B").borders(Borders::ALL));
                frame.render_widget(comp_b_widget, chunks[1]);

                // Fit Type
                let fit_type = Paragraph::new(format!("{:?}", mate.fit_type))
                    .block(Block::default().title("Fit Type").borders(Borders::ALL));
                frame.render_widget(fit_type, chunks[2]);

                // Calculate and display fits
                let nominal_fit = mate.calculate_nominal_fit(&feat_a, &feat_b);
                let min_fit = mate.calculate_min_fit(&feat_a, &feat_b);
                let max_fit = mate.calculate_max_fit(&feat_a, &feat_b);

                let nominal = Paragraph::new(format!("{:.3}", nominal_fit))
                    .block(Block::default().title("Nominal Fit").borders(Borders::ALL));
                frame.render_widget(nominal, chunks[3]);

                let min = Paragraph::new(format!("{:.3}", min_fit))
                    .block(Block::default().title("Minimum Fit").borders(Borders::ALL));
                frame.render_widget(min, chunks[4]);

                let max = Paragraph::new(format!("{:.3}", max_fit))
                    .block(Block::default().title("Maximum Fit").borders(Borders::ALL));
                frame.render_widget(max, chunks[5]);
                let validation = mate.validate(&feat_a, &feat_b);
                let validation_style = if !validation.is_valid {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                };

                let validation_block = Block::default()
                    .title("Validation")
                    .borders(Borders::ALL);

                let validation_text = if let Some(error) = validation.error_message {
                    format!("Invalid: {}", error)
                } else {
                    "Valid fit".to_string()
                };

                let validation_widget = Paragraph::new(validation_text)
                    .style(validation_style)
                    .block(validation_block);
                frame.render_widget(validation_widget, chunks[6]); // Add a new chunk for validation
            } else {
                // One or both features not found
                frame.render_widget(
                    Paragraph::new("One or both features not found in components")
                        .block(mate_block),
                    area,
                );
            }
        }
    } else {
        frame.render_widget(
            Paragraph::new("Select a mate to view details")
                .block(mate_block),
            area,
        );
    }
}

fn draw_component_selection_list(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    let items: Vec<ListItem> = state.project.components
        .iter()
        .map(|component| {
            ListItem::new(format!("{} ({} features)", component.name, component.features.len()))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Components").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    frame.render_stateful_widget(
        list,
        area,
        &mut state.ui.component_list_state.clone(),
    );
}

fn draw_feature_selection_list(frame: &mut Frame, component: &Component, app: &App, area: Rect) {
    let state = app.state();  // Get state reference from app
    let items: Vec<ListItem> = component.features
        .iter()
        .map(|feature| {
            ListItem::new(format!(
                "{} ({:?}) - {:.3} [{:+.3}/{:+.3}]",
                feature.name,
                feature.feature_type,
                feature.dimension.value,
                feature.dimension.plus_tolerance,
                feature.dimension.minus_tolerance
            ))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Features").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    frame.render_stateful_widget(
        list,
        area,
        &mut app.state().ui.feature_list_state.clone(),  // Use app.state()
    );
}


// Helper function to find a feature in the project state
fn find_feature<'a>(
    state: &'a AppState,
    component_name: &str,
    feature_name: &str
) -> Option<&'a Feature> {
    state.project.components
        .iter()
        .find(|c| c.name == component_name)?
        .features
        .iter()
        .find(|f| f.name == feature_name)
}

fn draw_mate_list(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.state();
    let mates = state.mates.filtered_mates();

    let title = match &state.mates.filter {
        Some(MateFilter::Component(comp)) => {
            format!("Mates for component {} (m: add, e: edit, d: delete, c: clear filter)", comp)
        },
        Some(MateFilter::Feature(comp, feat)) => {
            format!("Mates for {}.{} (m: add, e: edit, d: delete, c: clear filter)", comp, feat)
        },
        None => "Mates (m: add, e: edit, d: delete)".to_string(),
    };

    let items: Vec<ListItem> = mates
        .iter()
        .map(|mate| {
            let feature_a = find_feature(state, &mate.component_a, &mate.feature_a);
            let feature_b = find_feature(state, &mate.component_b, &mate.feature_b);

            let validation = if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                mate.validate(feat_a, feat_b)
            } else {
                FitValidation {
                    is_valid: false,
                    nominal_fit: 0.0,
                    min_fit: 0.0,
                    max_fit: 0.0,
                    error_message: Some("Missing features".to_string())
                }
            };

            let style = if !validation.is_valid {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };

            let text = format!(
                "{} ({}) ↔ {} ({}) - {:?}",
                mate.component_a, mate.feature_a,
                mate.component_b, mate.feature_b,
                mate.fit_type
            );

            if let Some(error) = validation.error_message {
                ListItem::new(vec![
                    Line::from(text),
                    Line::from(format!("  Error: {}", error))
                ]).style(style)
            } else {
                ListItem::new(text).style(style)
            }
        })
        .collect();

    let mates_block = Block::default()
        .title(title)
        .borders(Borders::ALL);

    let mates_list = List::new(items)
        .block(mates_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(
        mates_list,
        area,
        &mut state.ui.mate_list_state.clone(),
    );
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
