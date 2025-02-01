// src/ui/dialog.rs
use crate::config::FeatureType;
use crate::analysis::DistributionType;


#[derive(Default)]
pub struct ComponentEditData {
    pub name: String,
    pub revision: String,
    pub description: String,
    pub is_editing: bool,
    pub component_index: Option<usize>,
}

#[derive(Default)]
pub enum DialogState {
    #[default]
    None,
    ComponentEdit(ComponentEditData),
    FeatureEdit(FeatureEditData),
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