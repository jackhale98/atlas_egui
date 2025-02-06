// src/app.rs
use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;

use crate::state::{AppState, Screen, DialogState};
use crate::ui::{dialog, DialogManager}; // Add DialogManager import

pub struct AtlasApp {
    state: AppState,
    dialog_manager: DialogManager, // Add dialog manager
}

impl AtlasApp {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            dialog_manager: DialogManager::new(), // Initialize dialog manager
        }
    }

    fn show_menu(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    self.new_project();
                    ui.close_menu();
                }
                if ui.button("Open Project...").clicked() {
                    self.open_project();
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    if let Err(e) = self.state.save_project() {
                        self.state.error_message = Some(e.to_string());
                    }
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    self.save_project_as();
                    ui.close_menu();
                }
            });

            ui.separator();

            // Tab selection using buttons
            let tabs = [
                (Screen::Project, "Project"),
                (Screen::Components, "Components"),
                (Screen::Mates, "Mates"),
                (Screen::DependencyMatrix, "Dependencies"),
                (Screen::Analysis, "Analysis"),
            ];

            for (mode, label) in tabs {
                if ui.selectable_label(self.state.current_screen == mode, label).clicked() {
                    self.state.current_screen = mode;
                }
            }
        });
    }

    fn new_project(&mut self) {
        self.state = AppState::new();
    }

    fn open_project(&mut self) {
        let file_dialog = FileDialog::new()
            .add_filter("RON files", &["ron"])
            .set_title("Open Project File");

        if let Some(path) = file_dialog.pick_file() {
            self.load_project(path);
        }
    }

    fn load_project(&mut self, path: PathBuf) {
        let project_dir = path.parent().unwrap().to_path_buf();
        match self.state.file_manager.set_project_dir(project_dir.clone()) {
            Ok(_) => {
                self.state.project_dir = Some(project_dir);
                match self.state.file_manager.load_project(&path) {
                    Ok((project_file, components, mates_file, analyses)) => {
                        self.state.project_file = project_file;
                        self.state.components = components;
                        self.state.mates = mates_file.mates;
                        
                        // Load analyses and their latest results
                        self.state.analyses.clear();
                        self.state.latest_results.clear();
                        
                        for (analysis, results) in analyses {
                            // Store any existing results
                            if let Some(results) = results {
                                self.state.latest_results.insert(analysis.id.clone(), results);
                            }
                            
                            self.state.analyses.push(analysis);
                        }
                        
                        self.state.update_mate_graph();
                        self.state.error_message = None;
                    }
                    Err(e) => {
                        self.state.error_message = Some(format!("Error loading project: {}", e));
                    }
                }
            }
            Err(e) => {
                self.state.error_message = Some(format!("Error setting project directory: {}", e));
            }
        }
    }

    fn save_project_as(&mut self) {
        let file_dialog = FileDialog::new()
            .add_filter("RON files", &["ron"])
            .set_title("Save Project As");

        if let Some(path) = file_dialog.save_file() {
            let project_dir = path.parent().unwrap().to_path_buf();
            if let Ok(_) = self.state.file_manager.set_project_dir(project_dir.clone()) {
                self.state.project_dir = Some(project_dir);
                if let Err(e) = self.state.save_project() {
                    self.state.error_message = Some(e.to_string());
                }
            }
        }
    }
}

impl eframe::App for AtlasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_menu(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.current_screen {
                Screen::Project => {
                    crate::ui::project::show_project_view(ui, &mut self.state);
                },
                Screen::Components => {
                    crate::ui::components::show_components_view(ui, &mut self.state);
                },
                Screen::Mates => {
                    crate::ui::mates::show_mates_view(ui, &mut self.state);
                },
                Screen::DependencyMatrix => {
                    ui.label("Dependencies View - Coming Soon");
                },
                Screen::Analysis => {
                    crate::ui::analysis::show_analysis_view(ui, &mut self.state);
                },
            }
        });

        // Show error modal if needed
        let error_msg = self.state.error_message.clone(); // Clone first
        if let Some(error) = error_msg {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&error);
                    if ui.button("OK").clicked() {
                        self.state.error_message = None;
                    }
                });
        }

        // Handle dialogs using dialog manager
        self.dialog_manager.show(ctx, &mut self.state);
    }
}