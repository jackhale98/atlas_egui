// src/ui/dialog.rs
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
}