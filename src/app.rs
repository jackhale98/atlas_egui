// src/app.rs
use crate::input::InputHandler;
use crate::state::AppState;
use crate::state::ui_state::ScreenMode;
use crate::ui::dialog::*;
use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;


#[derive(Default)]
pub enum AppMessage {
    #[default]
    None,
    SwitchTab(ScreenMode),
}


pub struct App {
    pub state: AppState,
    pub message: AppMessage,
    input_handler: InputHandler,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            message: AppMessage::default(),
            input_handler: InputHandler::new(),
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn switch_tab(&mut self, mode: ScreenMode) {
        self.message = AppMessage::SwitchTab(mode);
    }
}

pub struct AtlasApp {
    app: App,
    current_tab: ScreenMode,
    error_message: Option<String>,
    show_error: bool,
    dialog_state: DialogState,
}

impl AtlasApp {
    pub fn new() -> Self {
        Self {
            app: App::new(),
            current_tab: ScreenMode::Project,
            error_message: None,
            show_error: false,
            dialog_state: DialogState::None,
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
                    self.save_project();
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    self.save_project_as();
                    ui.close_menu();
                }
            });

            // Tab selection using buttons
            ui.separator();
            let tabs = [
                (ScreenMode::Project, "Project"),
                (ScreenMode::Components, "Components"),
                (ScreenMode::Mates, "Mates"),
                (ScreenMode::DependencyMatrix, "Dependencies"),
                (ScreenMode::Analysis, "Analysis"),
            ];

            for (mode, label) in tabs {
                if ui.selectable_label(self.current_tab == mode, label).clicked() {
                    self.current_tab = mode;
                    self.app.state.ui.current_screen = mode;
                }
            }
        });
    }

    fn new_project(&mut self) {
        self.app = App::new();
        self.error_message = None;
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
        match self.app.state.file_manager.set_project_dir(project_dir.clone()) {
            Ok(_) => {
                self.app.state.project.project_dir = Some(project_dir);
                match self.app.state.file_manager.load_project(&path) {
                    Ok((project_file, components, mates_file, analyses)) => {
                        self.app.state.project.project_file = project_file;
                        self.app.state.project.components = components;
                        self.app.state.mates.mates = mates_file.mates;
                        self.app.state.analysis.analyses = analyses
                            .into_iter()
                            .map(|(analysis, _)| analysis)
                            .collect();
                        self.app.state.mates.update_dependency_graph(&self.app.state.project.components);
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error loading project: {}", e));
                        self.show_error = true;
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Error setting project directory: {}", e));
                self.show_error = true;
            }
        }
    }

    fn save_project(&mut self) {
        if self.app.state.project.project_dir.is_none() {
            self.save_project_as();
            return;
        }

        self.save_project_to_current_location();
    }

    fn save_project_as(&mut self) {
        let file_dialog = FileDialog::new()
            .add_filter("RON files", &["ron"])
            .set_title("Save Project As");

        if let Some(path) = file_dialog.save_file() {
            let project_dir = path.parent().unwrap().to_path_buf();
            match self.app.state.file_manager.set_project_dir(project_dir.clone()) {
                Ok(_) => {
                    self.app.state.project.project_dir = Some(project_dir);
                    self.save_project_to_current_location();
                }
                Err(e) => {
                    self.error_message = Some(format!("Error setting project directory: {}", e));
                    self.show_error = true;
                }
            }
        }
    }

    fn save_project_to_current_location(&mut self) {
        if let Err(e) = self.app.state.file_manager.save_project(
            &self.app.state.project.project_file,
            &self.app.state.project.components,
            &self.app.state.analysis.analyses,
        ) {
            self.error_message = Some(format!("Error saving project: {}", e));
            self.show_error = true;
        }
    }

    fn show_error_modal(&mut self, ctx: &egui::Context) {
        if self.show_error {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(error) = &self.error_message {
                        ui.label(error);
                    }
                    if ui.button("OK").clicked() {
                        self.show_error = false;
                    }
                });
        }
    }
}

impl eframe::App for AtlasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // At the start of update, process any pending messages
        match &self.app.message {
            AppMessage::SwitchTab(mode) => {
                self.current_tab = *mode;
                self.app.state.ui.current_screen = *mode;
                self.app.message = AppMessage::None;
            }
            AppMessage::None => {}
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_menu(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                ScreenMode::Project => {
                    crate::ui::project::draw_project_view(ui, &mut self.app);
                },
                ScreenMode::Components => {
                    crate::ui::components::draw_components_view(ui, &mut self.app, &mut self.dialog_state);
                },
                ScreenMode::Mates => {
                    crate::ui::mates::draw_mates_view(ui, &mut self.app, &mut self.dialog_state);
                },
                ScreenMode::DependencyMatrix => {
                    ui.label("Dependencies View - Coming Soon");
                },
                ScreenMode::Analysis => {
                    ui.label("Analysis View - Coming Soon");
                },
            }
        });

        self.show_error_modal(ctx);
        crate::ui::components::show_component_edit_dialog(ctx, &mut self.dialog_state, &mut self.app);
        crate::ui::components::show_feature_edit_dialog(ctx, &mut self.dialog_state, &mut self.app);
        crate::ui::mates::show_mate_edit_dialog(ctx, &mut self.dialog_state, &mut self.app);
    }
}