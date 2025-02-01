// src/ui/dialog.rs
use crate::config::FeatureType;
use crate::analysis::DistributionType;
use crate::config::mate::FitType;

#[derive(Default)]
pub struct ComponentEditData {
    pub name: String,
    pub revision: String,
    pub description: String,
    pub is_editing: bool,
    pub component_index: Option<usize>,
}

#[derive(Default)]
pub struct FeatureEditData {
    pub name: String,
    pub feature_type: FeatureType,
    pub value: String,
    pub plus_tolerance: String,
    pub minus_tolerance: String,
    pub distribution: DistributionType,
    pub is_editing: bool,
    pub feature_index: Option<usize>,
    pub component_index: Option<usize>,
}

#[derive(Default)]
pub struct MateEditData {
    pub component_a: String,
    pub feature_a: String,
    pub component_b: String,
    pub feature_b: String,
    pub fit_type: FitType,
    pub is_editing: bool,
    pub mate_index: Option<usize>,
}

#[derive(Default)]
pub enum DialogState {
    #[default]
    None,
    ComponentEdit(ComponentEditData),
    FeatureEdit(FeatureEditData),
    MateEdit(MateEditData),
}