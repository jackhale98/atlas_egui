// src/state/input_state.rs
use tui_input::Input;
use crate::analysis::stackup::{
    AnalysisMethod,
    MonteCarloSettings,
    StackupContribution,
    DistributionType
};
use crate::config::feature::FeatureType;
use crate::config::mate::FitType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing(EditField),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditField {
    ProjectName,
    Description,
    Units,
    ComponentName,
    ComponentDescription,
    FeatureName,
    FeatureValue,
    FeatureTolerance,
    MateComponentA,
    MateFeatureA,
    MateComponentB,
    MateFeatureB,
    MateFitType,
    AnalysisName,
    MonteCarloIterations,
    MonteCarloConfidence,
    MonteCarloSeed,
    DistributionParam1,
    DistributionParam2,
    DistributionParam3,
}

#[derive(Debug)]
pub struct MateInputs {
    pub component_a: Input,
    pub feature_a: Input,
    pub component_b: Input,
    pub feature_b: Input,
    pub fit_type: FitType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContributionSelectionState {
    SelectingComponent,
    SelectingFeature,
}

impl Default for MateInputs {
    fn default() -> Self {
        Self {
            component_a: Input::default(),
            feature_a: Input::default(),
            component_b: Input::default(),
            feature_b: Input::default(),
            fit_type: FitType::Clearance,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToleranceField {
    Plus,
    Minus,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MateSelectionState {
    SelectingComponentA,
    SelectingFeatureA,
    SelectingComponentB,
    SelectingFeatureB,
}

#[derive(Debug)]
pub struct InputState {
    pub mode: InputMode,
    pub project_inputs: ProjectInputs,
    pub component_inputs: ComponentInputs,
    pub feature_inputs: FeatureInputs,
    pub tolerance_focus: ToleranceField,
    pub mate_inputs: MateInputs,
    pub mate_selection_state: MateSelectionState,
    pub analysis_inputs: AnalysisInputs,
    pub contribution_inputs: ContributionInputs,
    pub contribution_selection_state: ContributionSelectionState,
}

#[derive(Debug)]
pub struct ProjectInputs {
    pub name: Input,
    pub description: Input,
}

#[derive(Debug)]
pub struct ComponentInputs {
    pub name: Input,
    pub description: Input,
}

#[derive(Debug)]
pub struct FeatureInputs {
    pub name: Input,
    pub feature_type: FeatureType,
    pub value: Input,
    pub plus_tolerance: Input,
    pub minus_tolerance: Input,
    pub distribution: Option<DistributionType>,
}

impl Default for FeatureInputs {
    fn default() -> Self {
        Self {
            name: Input::default(),
            feature_type: FeatureType::External,
            value: Input::default(),
            plus_tolerance: Input::default(),
            minus_tolerance: Input::default(),
            distribution: Some(DistributionType::Normal),
        }
    }
}


impl Default for InputState {
    fn default() -> Self {
        Self {
            mode: InputMode::Normal,
            project_inputs: ProjectInputs {
                name: Input::default(),
                description: Input::default(),
            },
            component_inputs: ComponentInputs {
                name: Input::default(),
                description: Input::default(),
            },
            feature_inputs: FeatureInputs {
                name: Input::default(),
                feature_type: FeatureType::External,
                value: Input::default(),
                plus_tolerance: Input::default(),
                minus_tolerance: Input::default(),
                distribution: Some(DistributionType::Normal),
            },
            tolerance_focus: ToleranceField::Plus,
            mate_inputs: MateInputs::default(),
            mate_selection_state: MateSelectionState::SelectingComponentA,
            analysis_inputs: AnalysisInputs::default(),
            contribution_inputs: ContributionInputs::default(),
            contribution_selection_state: ContributionSelectionState::SelectingComponent,
        }
    }
}


#[derive(Debug)]
pub struct AnalysisInputs {
    pub name: Input,
    pub methods: Vec<AnalysisMethod>,
    pub monte_carlo_settings: MonteCarloSettings,
    pub contributions: Vec<StackupContribution>,
    pub id: Option<String>,
}

#[derive(Debug)]
pub struct ContributionInputs {
    pub selected_component: String,
    pub selected_feature: String,
    pub direction: f64,
    pub half_count: bool,
    pub distribution_type: DistributionType,
}

impl Default for AnalysisInputs {
    fn default() -> Self {
        Self {
            name: Input::default(),
            methods: vec![AnalysisMethod::WorstCase],
            monte_carlo_settings: MonteCarloSettings {
                iterations: 10000,
                confidence: 0.9995,
                seed: None,
            },
            contributions: Vec::new(),
            id: None,
        }
    }
}

impl Default for ContributionInputs {
    fn default() -> Self {
        Self {
            selected_component: String::new(),
            selected_feature: String::new(),
            direction: 1.0,
            half_count: false,
            distribution_type: DistributionType::Normal,
        }
    }
}
