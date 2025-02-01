// src/state/ui_state.rs
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenMode {
    Project,
    Components,
    Mates,
    DependencyMatrix,
    Analysis,
}

#[derive(Debug, PartialEq)]
pub enum DialogMode {
    None,
    AddComponent,
    EditComponent,
    AddFeature,
    EditFeature,
    DeleteConfirm(DeletionTarget),
    AddMate,
    EditMate,
    AddAnalysis, 
    EditAnalysis,
    AddContribution,
    EditContribution,
}

#[derive(Debug, PartialEq)]
pub enum DeletionTarget {
    Component,
    Feature,
}

#[derive(Debug)]
pub struct UiState {
    pub current_screen: ScreenMode,
    pub dialog_mode: DialogMode,
    pub error_message: Option<String>,
    pub dialog_error: Option<String>,
    pub component_list_state: ListState,
    pub feature_list_state: ListState,
    pub mate_list_state: ListState,
    pub selected_mate: Option<usize>,
    pub analysis_tab: AnalysisTab,
    pub analysis_list_state: ListState,
    pub contribution_list_state: ListState,
    pub method_list_state: ListState,
    pub distribution_type_list_state: ListState,
    pub parameter_descriptions: Vec<String>,
    pub selected_analysis_index: Option<usize>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_screen: ScreenMode::Project,
            dialog_mode: DialogMode::None,
            error_message: None,
            dialog_error: None,
            component_list_state: ListState::default(),
            feature_list_state: ListState::default(),
            mate_list_state: ListState::default(),
            selected_mate: None,
            analysis_tab: AnalysisTab::List,
            analysis_list_state: ListState::default(),
            contribution_list_state: ListState::default(),
            method_list_state: ListState::default(),
            distribution_type_list_state: ListState::default(),
            parameter_descriptions: Vec::new(),
            selected_analysis_index: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisTab {
    List,
    Details,
    Results,
    Visualization,
}