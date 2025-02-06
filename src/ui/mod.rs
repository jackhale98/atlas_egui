// src/ui/mod.rs
pub mod dialog;
pub mod dialog_widgets;
pub mod project;
pub mod components;
pub mod mates;
pub mod analysis;

// Re-export dialog manager
pub use dialog::DialogManager;