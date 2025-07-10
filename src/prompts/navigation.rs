// src/prompts/navigation.rs
use anyhow::Result;
use inquire::{InquireError, Text, Select, Confirm, CustomType, validator::CustomTypeValidator};

/// Custom result type for menu navigation
#[derive(Debug)]
pub enum MenuResult<T> {
    Selection(T),
    GoBack,
    Exit,
}

/// Helper function for text input that handles Esc gracefully
pub fn prompt_text(prompt: &str) -> Result<Option<String>> {
    match Text::new(prompt).prompt() {
        Ok(text) => Ok(Some(text)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for text input with default value that handles Esc gracefully
pub fn prompt_text_with_default(prompt: &str, default: &str) -> Result<Option<String>> {
    match Text::new(prompt).with_default(default).prompt() {
        Ok(text) => Ok(Some(text)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for confirmation prompts that handles Esc gracefully
pub fn prompt_confirm(prompt: &str, default: bool) -> Result<Option<bool>> {
    match Confirm::new(prompt).with_default(default).prompt() {
        Ok(confirmed) => Ok(Some(confirmed)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for select prompts that handles Esc gracefully
pub fn prompt_select<T: Clone + std::fmt::Display>(prompt: &str, choices: Vec<T>) -> Result<Option<T>> {
    match Select::new(prompt, choices.clone()).prompt() {
        Ok(selection) => Ok(Some(selection)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for custom type prompts that handles Esc gracefully
pub fn prompt_custom_type<T: Clone + std::str::FromStr + std::fmt::Display>(
    prompt: &str,
    validator: impl CustomTypeValidator<T> + Clone + 'static
) -> Result<Option<T>>
where
    <T as std::str::FromStr>::Err: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    match CustomType::<T>::new(prompt).with_validator(validator.clone()).prompt() {
        Ok(value) => Ok(Some(value)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for custom type prompts with default value that handles Esc gracefully
pub fn prompt_custom_type_with_default<T: Clone + std::str::FromStr + std::fmt::Display>(
    prompt: &str,
    default: T,
    validator: impl CustomTypeValidator<T> + Clone + 'static
) -> Result<Option<T>>
where
    <T as std::str::FromStr>::Err: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    match CustomType::<T>::new(prompt).with_default(default).with_validator(validator.clone()).prompt() {
        Ok(value) => Ok(Some(value)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Main menu function that handles Esc with exit confirmation
pub fn show_main_menu<T: Clone>(prompt: &str, choices: Vec<T>) -> Result<MenuResult<T>>
where T: std::fmt::Display,
{
    let select = Select::new(prompt, choices.clone());

    match select.prompt() {
        Ok(choice) => Ok(MenuResult::Selection(choice)),
        Err(InquireError::OperationInterrupted) => {
            // ESC was pressed - ask for exit confirmation in main menu
            if confirm_exit()? {
                Ok(MenuResult::Exit)
            } else {
                show_main_menu(prompt, choices) // Try again
            }
        },
        Err(InquireError::OperationCanceled) => {
            // Also treat as exit request for main menu
            if confirm_exit()? {
                Ok(MenuResult::Exit)
            } else {
                show_main_menu(prompt, choices) // Try again
            }
        },
        Err(e) => Err(e.into()),
    }
}

/// Submenu function that handles Esc as "go back" without exit confirmation
pub fn show_submenu<T: Clone>(prompt: &str, choices: Vec<T>) -> Result<MenuResult<T>>
where T: std::fmt::Display,
{
    let select = Select::new(prompt, choices.clone());

    match select.prompt() {
        Ok(choice) => Ok(MenuResult::Selection(choice)),
        Err(InquireError::OperationInterrupted) => {
            // ESC was pressed - go back to parent menu
            Ok(MenuResult::GoBack)
        },
        Err(InquireError::OperationCanceled) => {
            // Also treat as go back - some terminals send this for ESC
            Ok(MenuResult::GoBack)
        },
        Err(e) => Err(e.into()),
    }
}

/// Confirms if the user really wants to exit
pub fn confirm_exit() -> Result<bool> {
    match Confirm::new("Are you sure you want to exit Atlas?")
        .with_default(false)
        .prompt()
    {
        Ok(confirmed) => Ok(confirmed),
        Err(InquireError::OperationInterrupted) => Ok(false), // ESC means "no, don't exit"
        Err(InquireError::OperationCanceled) => Ok(true),     // Ctrl+C means "yes, exit"
        Err(e) => Err(e.into()),
    }
}

/// Confirms if the user wants to save before exiting
pub fn confirm_save_before_exit() -> Result<Option<bool>> {
    match Confirm::new("Save project before exiting?")
        .with_default(true)
        .prompt()
    {
        Ok(save) => Ok(Some(save)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC means cancel exit
        Err(InquireError::OperationCanceled) => Ok(Some(false)), // Ctrl+C means don't save, just exit
        Err(e) => Err(e.into()),
    }
}

/// Show cancellation message consistently
pub fn show_cancellation_message(operation: &str) {
    println!("❌ {} cancelled", operation);
}

/// Show operation success message
pub fn show_success_message(operation: &str) {
    println!("✅ {} completed successfully", operation);
}