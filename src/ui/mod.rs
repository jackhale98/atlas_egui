// src/ui/mod.rs
pub mod dialog;
pub mod dialog_widgets;
pub mod project;
pub mod components;
pub mod mates;
pub mod analysis;
pub mod dependency_matrix;
pub mod git_control;

// Re-export dialog manager
pub use dialog::DialogManager;