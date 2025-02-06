// src/ui/dialog.rs
use eframe::egui;
use crate::state::{AppState, DialogState};
use crate::ui::dialog_widgets::{
    ComponentDialog,
    FeatureDialog,
    MateDialog,
    AnalysisDialog,
    ContributionDialog,
};
use crate::config::Component;
use crate::config::mate::Mate;
use crate::analysis::stackup::{StackupAnalysis, StackupContribution};

pub struct DialogManager {
    component_dialog: ComponentDialog,
    feature_dialog: FeatureDialog,
    mate_dialog: MateDialog,
    analysis_dialog: AnalysisDialog,
    contribution_dialog: ContributionDialog,
    current_state: DialogState,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            component_dialog: ComponentDialog::new(),
            feature_dialog: FeatureDialog::new(),
            mate_dialog: MateDialog::new(),
            analysis_dialog: AnalysisDialog::new(),
            contribution_dialog: ContributionDialog::new(),
            current_state: DialogState::None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState) {
        // Check if dialog state has changed
        if !std::mem::discriminant(&self.current_state).eq(&std::mem::discriminant(&state.current_dialog)) {
            // Dialog state has changed, open appropriate dialog
            match &state.current_dialog {
                DialogState::NewComponent { .. } => {
                    self.component_dialog.open(None);
                },
                DialogState::EditComponent { index, .. } => {
                    if let Some(component) = state.components.get(*index) {
                        self.component_dialog.open(Some(component));
                    }
                },
                DialogState::NewFeature { .. } => {
                    self.feature_dialog.open(None);
                },
                DialogState::EditFeature { component_index, feature_index, .. } => {
                    if let Some(component) = state.components.get(*component_index) {
                        if let Some(feature) = component.features.get(*feature_index) {
                            self.feature_dialog.open(Some(feature));
                        }
                    }
                },
                DialogState::NewMate { .. } => {
                    self.mate_dialog.open(None);
                },
                DialogState::EditMate { index, .. } => {
                    if let Some(mate) = state.mates.get(*index) {
                        self.mate_dialog.open(Some(mate));
                    }
                },
                DialogState::NewAnalysis { .. } => {
                    self.analysis_dialog.open(None);
                },
                DialogState::EditAnalysis { index, .. } => {
                    if let Some(analysis) = state.analyses.get(*index) {
                        self.analysis_dialog.open(Some(analysis));
                    }
                },
                DialogState::NewContribution { .. } => {
                    self.contribution_dialog.open(None);
                },
                DialogState::EditContribution { analysis_index, contribution_index, .. } => {
                    if let Some(analysis) = state.analyses.get(*analysis_index) {
                        if let Some(contribution_idx) = contribution_index {
                            if let Some(contribution) = analysis.contributions.get(*contribution_idx) {
                                self.contribution_dialog.open(Some(contribution));
                            }
                        }
                    }
                },
                DialogState::None => {},
            }
            self.current_state = state.current_dialog.clone();
        }

        // Handle the current dialog state
        match &state.current_dialog {
            DialogState::None => {},
            
            DialogState::NewComponent { .. } | DialogState::EditComponent { .. } => {
                let edit_index = if let DialogState::EditComponent { index, .. } = state.current_dialog {
                    Some(index)
                } else {
                    None
                };

                if let Some(changed) = self.component_dialog.show(
                    ctx,
                    edit_index,
                    &mut state.components,
                    || { state.current_dialog = DialogState::None }
                ) {
                    if changed {
                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                    }
                }
            },

            DialogState::NewFeature { component_index, .. } | DialogState::EditFeature { component_index, .. } => {
                let feature_index = if let DialogState::EditFeature { feature_index, .. } = state.current_dialog {
                    Some(feature_index)
                } else {
                    None
                };

                if let Some(changed) = self.feature_dialog.show(
                    ctx,
                    *component_index,
                    feature_index,
                    &mut state.components,
                    || { state.current_dialog = DialogState::None }
                ) {
                    if changed {
                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                    }
                }
            },

            DialogState::NewMate { .. } | DialogState::EditMate { .. } => {
                let edit_index = if let DialogState::EditMate { index, .. } = state.current_dialog {
                    Some(index)
                } else {
                    None
                };

                if let Some(changed) = self.mate_dialog.show(
                    ctx,
                    edit_index,
                    &state.components,
                    &mut state.mates,
                    || { state.current_dialog = DialogState::None }
                ) {
                    if changed {
                        state.update_mate_graph();
                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                    }
                }
            },

            DialogState::NewAnalysis { .. } | DialogState::EditAnalysis { .. } => {
                let edit_index = if let DialogState::EditAnalysis { index, .. } = state.current_dialog {
                    Some(index)
                } else {
                    None
                };

                if let Some(changed) = self.analysis_dialog.show(
                    ctx,
                    edit_index,
                    &mut state.analyses,
                    || { state.current_dialog = DialogState::None }
                ) {
                    if changed {
                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                    }
                }
            },

            DialogState::NewContribution { analysis_index, .. } | 
            DialogState::EditContribution { analysis_index, .. } => {
                let contribution_index = if let DialogState::EditContribution { contribution_index, .. } = state.current_dialog {
                    contribution_index
                } else {
                    None
                };

                if let Some(changed) = self.contribution_dialog.show(
                    ctx,
                    *analysis_index,
                    contribution_index,
                    &state.components,
                    &mut state.analyses,
                    || { state.current_dialog = DialogState::None }
                ) {
                    if changed {
                        if let Err(e) = state.save_project() {
                            state.error_message = Some(e.to_string());
                        }
                    }
                }
            },
        }
    }
}