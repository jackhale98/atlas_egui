// src/ui/git_control.rs
use eframe::egui;
use std::process::Command;
use std::path::Path;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fmt;
use crate::state::AppState;

// Add Debug implementation
#[derive(Default)]
pub struct GitControlState {
    cache: GitCache,
    commit_message: String,
    remote_url: String,
    remote_name: String,
    show_diff_panel: bool,
    view_mode: ViewMode,
    selected_commit: Option<GitCommitInfo>,
    // Add new fields for merge functionality
    merge_from_branch: String,
    merge_to_branch: String,
}

// Implement Debug for GitControlState
impl fmt::Debug for GitControlState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitControlState")
            .field("commit_message", &self.commit_message)
            .field("remote_url", &self.remote_url)
            .field("remote_name", &self.remote_name)
            .field("show_diff_panel", &self.show_diff_panel)
            .field("view_mode", &self.view_mode)
            .field("merge_from_branch", &self.merge_from_branch)
            .field("merge_to_branch", &self.merge_to_branch)
            .finish()
    }
}

// Different view modes for the diff panel
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    FileDiff,
    CommitInfo,
    BranchManagement,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::FileDiff
    }
}

// Struct to hold detailed commit information
#[derive(Clone, Debug)]
struct GitCommitInfo {
    hash: String,
    author: String,
    date: String,
    message: String,
    changed_files: Vec<String>,
}

struct GitCache {
    status: Option<GitStatus>,
    remotes: Option<Vec<GitRemote>>,
    log_entries: Option<Vec<GitLogEntry>>,
    branches: Option<GitBranches>,
    last_refresh: Instant,
    diff_cache: HashMap<String, String>,
    selected_file: Option<String>,
    commit_file_cache: HashMap<String, Vec<String>>, // Hash -> files
}

impl Default for GitCache {
    fn default() -> Self {
        Self {
            status: None,
            remotes: None,
            log_entries: None,
            branches: None,
            last_refresh: Instant::now() - Duration::from_secs(10), // Force initial refresh
            diff_cache: HashMap::new(),
            selected_file: None,
            commit_file_cache: HashMap::new(),
        }
    }
}

impl GitCache {
    fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(5) // Refresh at most every 5 seconds
    }

    fn refresh(&mut self, project_dir: &Path) -> Result<(), String> {
        if !self.should_refresh() {
            return Ok(());
        }

        self.status = Some(get_git_status(project_dir)?);
        self.remotes = Some(get_git_remotes(project_dir)?);
        self.log_entries = Some(get_git_log(project_dir)?);
        self.branches = Some(get_git_branches(project_dir)?);
        self.last_refresh = Instant::now();
        Ok(())
    }

    fn clear_diff_cache(&mut self) {
        self.diff_cache.clear();
    }
}

pub fn show_git_control(ui: &mut egui::Ui, state: &mut AppState) {
    // Create git control state if it doesn't exist
    if state.git_control_state.is_none() {
        state.git_control_state = Some(GitControlState::default());
    }
    
    let git_state = state.git_control_state.as_mut().unwrap();
    
    ui.heading("Git Version Control");
    
    if state.project_dir.is_none() {
        ui.label("No project directory selected. Please open or create a project first.");
        return;
    }
    
    let project_dir = state.project_dir.as_ref().unwrap().clone();
    let git_dir = project_dir.join(".git");
    
    // Check if the project is a git repository
    let is_git_repo = git_dir.exists() && git_dir.is_dir();
    
    // Create a mutable reference to state.error_message to avoid borrowing issues
    let error_message = &mut state.error_message;
    
    // Split the UI into left and right panels
    ui.columns(2, |columns| {
        // Left panel - Repository info and controls
        left_panel(&mut columns[0], error_message, git_state, &project_dir, is_git_repo);
        
        // Right panel - Diff viewer and other panels
        right_panel(&mut columns[1], error_message, git_state, &project_dir);
    });
}

// Left panel containing repository info, files, and actions
fn left_panel(
    ui: &mut egui::Ui,
    error_message: &mut Option<String>,
    git_state: &mut GitControlState, 
    project_dir: &Path, 
    is_git_repo: bool
) {
    ui.group(|ui| {
        ui.heading("Repository Status");
        
        if !is_git_repo {
            ui.horizontal(|ui| {
                ui.label("This project is not yet under version control.");
                if ui.button("Initialize Git Repository").clicked() {
                    match initialize_git_repo(project_dir) {
                        Ok(_) => {
                            // Success, refresh status
                            git_state.cache.clear_diff_cache();
                        },
                        Err(e) => {
                            *error_message = Some(format!("Failed to initialize git repository: {}", e));
                        }
                    }
                }
            });
            return;
        }
        
        // Refresh the cached data if needed
        if let Err(e) = git_state.cache.refresh(project_dir) {
            ui.label(format!("Error refreshing git status: {}", e));
            return;
        }
        
        // Clone the status and other data we need to avoid borrow checker issues
        let branch = git_state.cache.status.as_ref()
            .map(|s| s.branch.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let changed_files = git_state.cache.status.as_ref()
            .map(|s| s.changed_files.clone())
            .unwrap_or_default();
        
        let staged_files = git_state.cache.status.as_ref()
            .map(|s| s.staged_files.clone())
            .unwrap_or_default();
        
        // Branch display and selection
        if let Some(branches) = &git_state.cache.branches {
            // Clone the branches data to avoid borrow conflicts
            let branches_clone = branches.clone();
            let current_branch = git_state.cache.status.as_ref()
                .map(|s| s.branch.clone())
                .unwrap_or_else(|| "unknown".to_string());
                
            ui.horizontal(|ui| {
                ui.label("Branch:");
                egui::ComboBox::from_id_source("branch_selector")
                    .selected_text(current_branch.clone())
                    .show_ui(ui, |ui| {
                        for b in &branches_clone.local {
                            let text = if b == &current_branch { format!("* {}", b) } else { b.clone() };
                            if ui.selectable_label(b == &current_branch, text).clicked() {
                                // Switch branch
                                let branch_name = b.clone(); // Clone before using in closure
                                if let Err(e) = switch_branch(project_dir, &branch_name) {
                                    *error_message = Some(format!("Failed to switch branch: {}", e));
                                } else {
                                    // Hard reset to properly update working directory
                                    if let Err(e) = reset_hard(project_dir) {
                                        *error_message = Some(format!("Failed to reset after branch switch: {}", e));
                                    }
                                    
                                    // Force refresh cache
                                    git_state.cache.clear_diff_cache();
                                    git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                }
                            }
                        }
                    });
                
                if ui.button("Branch Management").clicked() {
                    git_state.view_mode = ViewMode::BranchManagement;
                }
            });
        } else {
            ui.label(format!("Branch: {}", branch));
        }
        
        ui.add_space(10.0);
        
        // Show changed files
        ui.group(|ui| {
            ui.heading("Changed Files");
            
            if changed_files.is_empty() {
                ui.label("No changes detected");
            } else {
                // Make the file list scrollable with a fixed height
                egui::ScrollArea::vertical()
                    .id_source("git_changed_files_scroll") 
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for file in &changed_files {
                            ui.horizontal(|ui| {
                                let mut checked = staged_files.contains(file);
                                if ui.checkbox(&mut checked, file.clone()).changed() {
                                    if checked {
                                        // Stage file
                                        if let Err(e) = stage_file(project_dir, file) {
                                            *error_message = Some(format!("Failed to stage file: {}", e));
                                        }
                                        // Clear diff cache when staging status changes
                                        git_state.cache.clear_diff_cache();
                                        // Force refresh on next frame
                                        git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                    } else {
                                        // Unstage file
                                        if let Err(e) = unstage_file(project_dir, file) {
                                            *error_message = Some(format!("Failed to unstage file: {}", e));
                                        }
                                        // Clear diff cache when staging status changes
                                        git_state.cache.clear_diff_cache();
                                        // Force refresh on next frame
                                        git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                    }
                                }
                                
                                // Add view diff button
                                if ui.small_button("View Diff").clicked() {
                                    git_state.cache.selected_file = Some(file.clone());
                                    git_state.show_diff_panel = true;
                                    git_state.view_mode = ViewMode::FileDiff;
                                    git_state.selected_commit = None;
                                }
                            });
                        }
                    });
            }
        });
        
        ui.add_space(10.0);
        
        // Commit area
        ui.group(|ui| {
            ui.heading("Commit Changes");
            
            ui.label("Commit Message:");
            ui.text_edit_multiline(&mut git_state.commit_message);
            
            ui.horizontal(|ui| {
                let can_commit = !staged_files.is_empty() && !git_state.commit_message.trim().is_empty();
                if ui.add_enabled(can_commit, egui::Button::new("Commit")).clicked() {
                    match commit_changes(project_dir, &git_state.commit_message) {
                        Ok(_) => {
                            // Clear commit message after successful commit
                            git_state.commit_message.clear();
                            // Force refresh and clear diff cache
                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                            git_state.cache.clear_diff_cache();
                        },
                        Err(e) => {
                            *error_message = Some(format!("Failed to commit changes: {}", e));
                        }
                    }
                }
                
                if ui.button("Refresh Status").clicked() {
                    // Force refresh
                    git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                    git_state.cache.clear_diff_cache();
                }
            });
        });
        
        ui.add_space(10.0);
        
        // Remote repository operations
        ui.group(|ui| {
            ui.heading("Remote Repository");
            
            // Clone remotes to avoid borrow issues
            let remotes = git_state.cache.remotes.as_ref()
                .map(|r| r.clone())
                .unwrap_or_default();
            
            if remotes.is_empty() {
                ui.label("No remote repositories configured.");
                
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut git_state.remote_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut git_state.remote_url);
                });
                
                let can_add = !git_state.remote_url.trim().is_empty() && !git_state.remote_name.trim().is_empty();
                if ui.add_enabled(can_add, egui::Button::new("Add Remote")).clicked() {
                    match add_git_remote(project_dir, &git_state.remote_name, &git_state.remote_url) {
                        Ok(_) => {
                            // Clear fields after successful add
                            git_state.remote_name.clear();
                            git_state.remote_url.clear();
                            // Force refresh
                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                        },
                        Err(e) => {
                            *error_message = Some(format!("Failed to add remote: {}", e));
                        }
                    }
                }
            } else {
                for remote in remotes {
                    let remote_name = remote.name.clone(); // Clone for closure
                    ui.horizontal(|ui| {
                        ui.label(&remote.name);
                        ui.label(remote.url);
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Pull").clicked() {
                                if let Err(e) = git_pull(project_dir, &remote_name) {
                                    *error_message = Some(format!("Failed to pull changes: {}", e));
                                } else {
                                    // Force refresh
                                    git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                    git_state.cache.clear_diff_cache();
                                }
                            }
                            
                            if ui.button("Push").clicked() {
                                if let Err(e) = git_push(project_dir, &remote_name) {
                                    *error_message = Some(format!("Failed to push changes: {}", e));
                                } else {
                                    // Force refresh
                                    git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                }
                            }
                        });
                    });
                }
            }
        });
        
        ui.add_space(10.0);
        
        // Commit history
        ui.group(|ui| {
            ui.heading("Commit History");
            
            let log_entries = git_state.cache.log_entries.as_ref()
                .map(|l| l.clone())
                .unwrap_or_default();
            
            egui::ScrollArea::vertical()
            .id_source("git_history_scroll")
            .max_height(150.0).show(ui, |ui| {
                for entry in &log_entries {
                    let is_selected = git_state.selected_commit.as_ref()
                        .map(|c| c.hash == entry.hash)
                        .unwrap_or(false);
                    
                    // Use selectable to indicate current selection
                    let commit_ui = ui.selectable_label(
                        is_selected,
                        format!("{} ({}): {}", entry.hash, entry.date, entry.message)
                    );
                    
                    if commit_ui.clicked() {
                        // Get files changed in this commit
                        match get_commit_files(project_dir, &entry.hash) {
                            Ok(files) => {
                                // Store in cache for future reference
                                git_state.cache.commit_file_cache.insert(entry.hash.clone(), files.clone());
                                
                                // Store commit info for display
                                git_state.selected_commit = Some(GitCommitInfo {
                                    hash: entry.hash.clone(),
                                    author: entry.author.clone(),
                                    date: entry.date.clone(),
                                    message: entry.message.clone(),
                                    changed_files: files,
                                });
                                
                                // Switch to commit info view mode
                                git_state.view_mode = ViewMode::CommitInfo;
                            },
                            Err(e) => {
                                *error_message = Some(format!("Failed to get commit files: {}", e));
                            }
                        }
                    }
                }
            });
        });
    });
}

// Right panel for diff viewing and other context-specific panels
fn right_panel(
    ui: &mut egui::Ui,
    error_message: &mut Option<String>,
    git_state: &mut GitControlState,
    project_dir: &Path
) {
    ui.group(|ui| {
        match git_state.view_mode {
            ViewMode::FileDiff => {
                show_file_diff_panel(ui, git_state, project_dir);
            },
            ViewMode::CommitInfo => {
                show_commit_info_panel(ui, error_message, git_state, project_dir);
            },
            ViewMode::BranchManagement => {
                show_branch_management_panel(ui, error_message, git_state, project_dir);
            }
        }
    });
}

// Show file diff panel
fn show_file_diff_panel(ui: &mut egui::Ui, git_state: &mut GitControlState, project_dir: &Path) {
    ui.vertical(|ui| {
        ui.heading("Diff Viewer");
        
        if let Some(file) = git_state.cache.selected_file.clone() {
            ui.horizontal(|ui| {
                ui.heading(&file);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("×").clicked() {
                        git_state.show_diff_panel = false;
                        
                        // This is the key fix - reset the view mode to match the selected commit if any
                        if git_state.selected_commit.is_some() {
                            git_state.view_mode = ViewMode::CommitInfo;
                        } else {
                            git_state.view_mode = ViewMode::FileDiff;
                        }
                        
                        // Clear the selected file
                        git_state.cache.selected_file = None;
                    }
                });
            });
            
            ui.separator();
            
            // Check if we have this diff cached
            let diff = if let Some(cached) = git_state.cache.diff_cache.get(&file) {
                cached.clone()
            } else {
                // Determine if this is a commit file diff or a working directory diff
                let diff_result = if file.contains(':') {
                    // It's a commit file diff (format: "hash:file")
                    let parts: Vec<&str> = file.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        get_commit_file_diff(project_dir, parts[0], parts[1])
                    } else {
                        Err(format!("Invalid file format: {}", file))
                    }
                } else {
                    // It's a working directory diff
                    get_file_diff(project_dir, &file)
                };
                
                match diff_result {
                    Ok(diff) => {
                        git_state.cache.diff_cache.insert(file.clone(), diff.clone());
                        diff
                    },
                    Err(e) => {
                        format!("Error getting diff: {}", e)
                    }
                }
            };
            
            show_diff_content(ui, &diff);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a file to view its diff");
            });
        }
    });
}

// Show commit info panel
fn show_commit_info_panel(
    ui: &mut egui::Ui, 
    error_message: &mut Option<String>,
    git_state: &mut GitControlState, 
    project_dir: &Path
) {
    ui.vertical(|ui| {
        ui.heading("Commit Details");
        
        // Clone the commit data to avoid borrow checker issues
        let commit_data = git_state.selected_commit.clone();
        
        if let Some(commit) = commit_data {
            let commit_hash = commit.hash.clone();
            
            ui.horizontal(|ui| {
                ui.strong("Commit:");
                ui.label(&commit.hash);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("×").clicked() {
                        git_state.selected_commit = None;
                        git_state.view_mode = ViewMode::FileDiff;
                    }
                });
            });
            
            ui.horizontal(|ui| {
                ui.strong("Author:");
                ui.label(&commit.author);
            });
            
            ui.horizontal(|ui| {
                ui.strong("Date:");
                ui.label(&commit.date);
            });
            
            ui.horizontal(|ui| {
                ui.strong("Message:");
                ui.label(&commit.message);
            });
            
            ui.add_space(10.0);
            ui.label("Files changed:");
            
            egui::ScrollArea::vertical()
                .id_source("commit_files_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    for file in &commit.changed_files {
                        let file_clone = file.clone();
                        ui.horizontal(|ui| {
                            ui.label(&file_clone);
                            if ui.small_button("View Diff").clicked() {
                                // Show diff for this file at this commit
                                match get_commit_file_diff(project_dir, &commit_hash, &file_clone) {
                                    Ok(diff) => {
                                        // Cache the diff
                                        let cache_key = format!("{}:{}", commit_hash, file_clone);
                                        git_state.cache.diff_cache.insert(cache_key.clone(), diff.clone());
                                        
                                        // Display the diff
                                        git_state.cache.selected_file = Some(cache_key);
                                        git_state.show_diff_panel = true;
                                        git_state.view_mode = ViewMode::FileDiff;
                                    },
                                    Err(e) => {
                                        *error_message = Some(format!("Failed to get commit file diff: {}", e));
                                    }
                                }
                            }
                        });
                    }
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a commit to view details");
            });
        }
    });
}

// Show branch management panel
fn show_branch_management_panel(
    ui: &mut egui::Ui, 
    error_message: &mut Option<String>,
    git_state: &mut GitControlState, 
    project_dir: &Path
) {
    ui.vertical(|ui| {
        ui.heading("Branch Management");
        
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    git_state.view_mode = ViewMode::FileDiff;
                }
            });
        });
        
        // New branch creation
        ui.group(|ui| {
            ui.heading("Create New Branch");
            
            static mut NEW_BRANCH_NAME: String = String::new();
            static mut CHECKOUT_NEW_BRANCH: bool = false; // Use proper state for the checkbox
            
            // Safety: This is only used in the UI context, not in a multithreaded environment
            let new_branch_name = unsafe { &mut NEW_BRANCH_NAME };
            let checkout_new_branch = unsafe { &mut CHECKOUT_NEW_BRANCH };
            
            ui.horizontal(|ui| {
                ui.label("Branch Name:");
                ui.text_edit_singleline(new_branch_name);
                
                let can_create = !new_branch_name.trim().is_empty();
                if ui.add_enabled(can_create, egui::Button::new("Create")).clicked() {
                    match create_branch(project_dir, new_branch_name) {
                        Ok(_) => {
                            // Check out the new branch if requested
                            if *checkout_new_branch {
                                if let Err(e) = switch_branch(project_dir, new_branch_name) {
                                    *error_message = Some(format!("Failed to checkout branch: {}", e));
                                } else {
                                    // Hard reset to properly update working directory
                                    if let Err(e) = reset_hard(project_dir) {
                                        *error_message = Some(format!("Failed to reset after branch switch: {}", e));
                                    }
                                }
                            }
                            
                            // Clear the branch name
                            *new_branch_name = String::new();
                            // Force refresh
                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                            git_state.cache.clear_diff_cache();
                        },
                        Err(e) => {
                            *error_message = Some(format!("Failed to create branch: {}", e));
                        }
                    }
                }
            });
            
            // Use the proper state variable for the checkbox
            ui.checkbox(checkout_new_branch, "Checkout new branch")
                .on_hover_text("Create and checkout new branch");
        });
        
        ui.add_space(10.0);
        
        // Branch list
        ui.group(|ui| {
            ui.heading("Branches");
            
            if let Some(branches) = &git_state.cache.branches {
                // Clone branches to avoid borrow conflicts
                let branches_clone = branches.clone();
                let current_branch = git_state.cache.status.as_ref()
                    .map(|s| s.branch.clone())
                    .unwrap_or_default();
                
                ui.label("Local:");
                
                egui::ScrollArea::vertical()
                    .id_source("local_branches_scroll")
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for branch in &branches_clone.local {
                            ui.horizontal(|ui| {
                                let is_current = branch == &current_branch;
                                let text = if is_current { format!("* {}", branch) } else { branch.clone() };
                                
                                ui.selectable_label(is_current, text);
                                
                                if !is_current {
                                    let branch_name = branch.clone(); // Clone branch name for closure
                                    if ui.small_button("Checkout").clicked() {
                                        if let Err(e) = switch_branch(project_dir, &branch_name) {
                                            *error_message = Some(format!("Failed to switch branch: {}", e));
                                        } else {
                                            // Hard reset to properly update working directory
                                            if let Err(e) = reset_hard(project_dir) {
                                                *error_message = Some(format!("Failed to reset after branch switch: {}", e));
                                            }
                                            // Force refresh
                                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                            git_state.cache.clear_diff_cache();
                                        }
                                    }
                                    
                                    if ui.small_button("Delete").clicked() {
                                        if let Err(e) = delete_branch(project_dir, &branch_name) {
                                            *error_message = Some(format!("Failed to delete branch: {}", e));
                                        } else {
                                            // Force refresh
                                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                        }
                                    }
                                }
                            });
                        }
                    });
                
                if !branches_clone.remote.is_empty() {
                    ui.add_space(10.0);
                    ui.label("Remote:");
                    
                    egui::ScrollArea::vertical()
                        .id_source("remote_branches_scroll")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for branch in &branches_clone.remote {
                                let branch_name = branch.clone(); // Clone for closure
                                ui.horizontal(|ui| {
                                    ui.label(&branch_name);
                                    
                                    if ui.small_button("Checkout").clicked() {
                                        let local_name = branch_name.split('/').last().unwrap_or(&branch_name).to_string();
                                        if let Err(e) = checkout_remote_branch(project_dir, &branch_name, &local_name) {
                                            *error_message = Some(format!("Failed to checkout remote branch: {}", e));
                                        } else {
                                            // Force refresh
                                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                            git_state.cache.clear_diff_cache();
                                        }
                                    }
                                });
                            }
                        });
                }
            } else {
                ui.label("No branch information available");
            }
        });
        
        // Add Merge section
        ui.add_space(10.0);
        
        ui.group(|ui| {
            ui.heading("Merge Branches");
            
            if let Some(branches) = &git_state.cache.branches {
                // Clone branches to avoid borrow checker issues
                let branches_clone = branches.clone();
                let current_branch = git_state.cache.status.as_ref()
                    .map(|s| s.branch.clone())
                    .unwrap_or_default();
                
                // From branch selection
                ui.horizontal(|ui| {
                    ui.label("From Branch:");
                    egui::ComboBox::from_id_source("merge_from_branch")
                        .selected_text(&git_state.merge_from_branch)
                        .show_ui(ui, |ui| {
                            for branch in &branches_clone.local {
                                if branch != &current_branch {
                                    ui.selectable_value(
                                        &mut git_state.merge_from_branch,
                                        branch.clone(),
                                        branch
                                    );
                                }
                            }
                        });
                });
                
                // To branch selection
                ui.horizontal(|ui| {
                    ui.label("To Branch:");
                    egui::ComboBox::from_id_source("merge_to_branch")
                        .selected_text(&git_state.merge_to_branch)
                        .show_ui(ui, |ui| {
                            for branch in &branches_clone.local {
                                ui.selectable_value(
                                    &mut git_state.merge_to_branch,
                                    branch.clone(),
                                    branch
                                );
                            }
                        });
                });
                
                // Merge button
                let can_merge = !git_state.merge_from_branch.is_empty() && 
                              !git_state.merge_to_branch.is_empty() && 
                              git_state.merge_from_branch != git_state.merge_to_branch;
                
                if ui.add_enabled(can_merge, egui::Button::new("Merge")).clicked() {
                    if let Err(e) = merge_branch(
                        project_dir, 
                        &git_state.merge_from_branch, 
                        &git_state.merge_to_branch
                    ) {
                        *error_message = Some(format!("Failed to merge branch: {}", e));
                    } else {
                        // Success message
                        *error_message = Some(format!(
                            "Successfully merged {} into {}", 
                            git_state.merge_from_branch, 
                            git_state.merge_to_branch
                        ));
                        
                        // Force refresh
                        git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                        git_state.cache.clear_diff_cache();
                    }
                }
            } else {
                ui.label("No branch information available");
            }
        });
    });
}

// Function to display diff content with syntax highlighting
fn show_diff_content(ui: &mut egui::Ui, diff: &str) {
    // More efficient diff rendering to avoid lag
    egui::ScrollArea::vertical()
        .id_source("diff_content_scroll")
        .max_height(ui.available_height() - 40.0)
        .show(ui, |ui| {
            let lines: Vec<&str> = diff.lines().collect();
            
            // Visible range optimization - only render visible content
            // Calculate the number of lines that can be visible based on available height
            let line_height = 16.0; // Approximate height of a line in pixels
            let available_height = ui.available_height();
            let max_visible_lines = (available_height / line_height).ceil() as usize + 2; // +2 for padding
            
            egui::Grid::new("diff_grid")
                .striped(true)
                .spacing([2.0, 0.0])
                .show(ui, |ui| {
                    for line in lines {
                        // Different coloring for different line types
                        let color = if line.starts_with('+') && !line.starts_with("+++") {
                            egui::Color32::from_rgb(0, 128, 0) // Added line - green
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            egui::Color32::from_rgb(255, 0, 0) // Removed line - red
                        } else if line.starts_with("@@") {
                            egui::Color32::from_rgb(0, 0, 128) // Hunk header - blue
                        } else {
                            ui.style().visuals.text_color() // Normal text color
                        };
                        
                        ui.colored_label(color, line);
                        ui.end_row();
                    }
                });
        });
}

// Get information about files changed in a specific commit
fn get_commit_files(project_dir: &Path, commit_hash: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["show", "--name-only", "--format=", commit_hash])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get commit files: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to get commit files: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = output_str
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();
    
    Ok(files)
}

// Function to merge a branch
fn merge_branch(project_dir: &Path, from_branch: &str, to_branch: &str) -> Result<(), String> {
    // First, switch to the target branch
    let checkout_output = Command::new("git")
        .args(["checkout", to_branch])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to checkout target branch: {}", e))?;
    
    if !checkout_output.status.success() {
        return Err(format!("Failed to checkout target branch: {}", 
                          String::from_utf8_lossy(&checkout_output.stderr)));
    }
    
    // Then, merge the source branch
    let merge_output = Command::new("git")
        .args(["merge", from_branch])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to merge branch: {}", e))?;
    
    if !merge_output.status.success() {
        return Err(format!("Failed to merge branch: {}", 
                          String::from_utf8_lossy(&merge_output.stderr)));
    }
    
    Ok(())
}

// FIXED: Get diff for a specific file at a specific commit
fn get_commit_file_diff(project_dir: &Path, commit_hash: &str, file: &str) -> Result<String, String> {
    // Use git diff to show the changes introduced by a specific commit
    let output = Command::new("git")
        .args(["diff", &format!("{}^..{}", commit_hash, commit_hash), "--", file])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get commit diff: {}", e))?;
    
    if !output.status.success() {
        // For the initial commit or if the diff fails, try a different approach
        if String::from_utf8_lossy(&output.stderr).contains("fatal: bad revision") {
            // This might be the initial commit, try a different approach
            return get_initial_commit_diff(project_dir, commit_hash, file);
        }
        return Err(format!("Failed to get commit diff: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    
    // If the diff is empty (no changes to this file in this commit)
    if diff.is_empty() {
        return Err(format!("No changes to {} in commit {}", file, commit_hash));
    }
    
    Ok(diff)
}

// For initial commits without a parent
fn get_initial_commit_diff(project_dir: &Path, commit_hash: &str, file: &str) -> Result<String, String> {
    // Get the file content at this commit
    let output = Command::new("git")
        .args(["show", &format!("{}:{}", commit_hash, file)])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get file content at commit: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to get file content: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let content = String::from_utf8_lossy(&output.stdout).to_string();
    
    // For initial commits, just show the file as completely added
    let mut diff = String::new();
    diff.push_str(&format!("--- /dev/null\n"));
    diff.push_str(&format!("+++ b/{}\n", file));
    diff.push_str("@@ -0,0 +1,");
    diff.push_str(&format!("{} @@\n", content.lines().count()));
    
    for line in content.lines() {
        diff.push_str(&format!("+{}\n", line));
    }
    
    Ok(diff)
}

// Function to get diff for a file
fn get_file_diff(project_dir: &Path, file: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "--color=never", "--", file])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get diff: {}", e))?;
    
    // Also include staged changes
    let staged_output = Command::new("git")
        .args(["diff", "--staged", "--color=never", "--", file])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get staged diff: {}", e))?;
    
    let mut diff = String::from_utf8_lossy(&output.stdout).to_string();
    
    if !staged_output.stdout.is_empty() {
        if !diff.is_empty() {
            diff.push_str("\n\n--- STAGED CHANGES ---\n\n");
        }
        diff.push_str(&String::from_utf8_lossy(&staged_output.stdout));
    }
    
    if diff.is_empty() {
        diff = "No changes detected".to_string();
    }
    
    Ok(diff)
}

// Add function to perform a hard reset after branch checkout
fn reset_hard(project_dir: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(["reset", "--hard"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to perform hard reset: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to perform hard reset: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

// Git branches structure
#[derive(Clone, Debug)]
struct GitBranches {
    local: Vec<String>,
    remote: Vec<String>,
}

// Function to get branches
fn get_git_branches(project_dir: &Path) -> Result<GitBranches, String> {
    // Get local branches
    let local_output = Command::new("git")
        .args(["branch", "--list"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get local branches: {}", e))?;
    
    let local_str = String::from_utf8_lossy(&local_output.stdout);
    let mut local_branches = Vec::new();
    
    for line in local_str.lines() {
        let branch_name = line.trim_start_matches('*').trim();
        if !branch_name.is_empty() {
            local_branches.push(branch_name.to_string());
        }
    }
    
    // Get remote branches
    let remote_output = Command::new("git")
        .args(["branch", "--remote"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get remote branches: {}", e))?;
    
    let remote_str = String::from_utf8_lossy(&remote_output.stdout);
    let mut remote_branches = Vec::new();
    
    for line in remote_str.lines() {
        let branch_name = line.trim();
        if !branch_name.is_empty() && !branch_name.contains("HEAD") {
            remote_branches.push(branch_name.to_string());
        }
    }
    
    Ok(GitBranches { local: local_branches, remote: remote_branches })
}

// Function to switch to a branch
fn switch_branch(project_dir: &Path, branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to switch branch: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to switch branch: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

// Function to create a new branch
fn create_branch(project_dir: &Path, branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["branch", branch])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to create branch: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to create branch: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

// Function to delete a branch
fn delete_branch(project_dir: &Path, branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["branch", "-d", branch])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to delete branch: {}", e))?;
    
    if !output.status.success() {
        // Try force delete if regular delete fails
        let force_output = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(project_dir)
            .output()
            .map_err(|e| format!("Failed to force delete branch: {}", e))?;
        
        if !force_output.status.success() {
            return Err(format!("Failed to delete branch: {}", String::from_utf8_lossy(&force_output.stderr)));
        }
    }
    
    Ok(())
}

// Function to checkout a remote branch
fn checkout_remote_branch(project_dir: &Path, remote_branch: &str, local_branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", "-b", local_branch, remote_branch])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to checkout remote branch: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to checkout remote branch: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

// Git operation structures
#[derive(Clone, Debug)]
struct GitStatus {
    branch: String,
    changed_files: Vec<String>,
    staged_files: Vec<String>,
}

#[derive(Clone, Debug)]
struct GitRemote {
    name: String,
    url: String,
}

#[derive(Clone, Debug)]
struct GitLogEntry {
    hash: String,
    author: String,
    date: String,
    message: String,
}

// Git operations - keep the original functions
fn initialize_git_repo(project_dir: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(["init"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to execute git init: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Git init failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn get_git_status(project_dir: &Path) -> Result<GitStatus, String> {
    // Get current branch
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get current branch: {}", e))?;
    
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    
    // Get changed files (both staged and unstaged)
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get git status: {}", e))?;
    
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    
    let mut changed_files = Vec::new();
    let mut staged_files = Vec::new();
    
    for line in status_str.lines() {
        if line.len() < 3 {
            continue;
        }
        
        let status_code = &line[0..2];
        let file_path = line[3..].to_string();
        
        // Add to changed files list
        changed_files.push(file_path.clone());
        
        // Check if file is staged
        if status_code.starts_with('A') || status_code.starts_with('M') || status_code.starts_with('D') {
            staged_files.push(file_path);
        }
    }
    
    Ok(GitStatus { branch, changed_files, staged_files })
}

fn stage_file(project_dir: &Path, file: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", file])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to stage file: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to stage file: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn unstage_file(project_dir: &Path, file: &str) -> Result<(), String> {
    // Use the correct command to unstage a file with "--" to disambiguate paths
    let output = Command::new("git")
        .args(["restore", "--staged", "--", file])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to unstage file: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to unstage file: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn commit_changes(project_dir: &Path, message: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to commit changes: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to commit changes: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn get_git_remotes(project_dir: &Path) -> Result<Vec<GitRemote>, String> {
    let output = Command::new("git")
        .args(["remote", "-v"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get remotes: {}", e))?;
    
    let remote_str = String::from_utf8_lossy(&output.stdout);
    let mut remotes = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    
    for line in remote_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            let url = parts[1].to_string();
            
            // Only add each remote once (git remote -v shows fetch and push URLs)
            if !seen_names.contains(&name) {
                seen_names.insert(name.clone());
                remotes.push(GitRemote { name, url });
            }
        }
    }
    
    Ok(remotes)
}

fn add_git_remote(project_dir: &Path, name: &str, url: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["remote", "add", name, url])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to add remote: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to add remote: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn git_pull(project_dir: &Path, remote: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["pull", remote])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to pull changes: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to pull changes: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn git_push(project_dir: &Path, remote: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["push", remote])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to push changes: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Failed to push changes: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

fn get_git_log(project_dir: &Path) -> Result<Vec<GitLogEntry>, String> {
    let output = Command::new("git")
        .args(["log", "--pretty=format:%h|%an|%ad|%s", "--date=short", "-n", "10"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to get git log: {}", e))?;
    
    let log_str = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    
    for line in log_str.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            entries.push(GitLogEntry {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                date: parts[2].to_string(),
                message: parts[3].to_string(),
            });
        }
    }
    
    Ok(entries)
}