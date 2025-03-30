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
}

// Implement Debug for GitControlState
impl fmt::Debug for GitControlState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitControlState")
            .field("commit_message", &self.commit_message)
            .field("remote_url", &self.remote_url)
            .field("remote_name", &self.remote_name)
            .field("show_diff_panel", &self.show_diff_panel)
            .finish()
    }
}

struct GitCache {
    status: Option<GitStatus>,
    remotes: Option<Vec<GitRemote>>,
    log_entries: Option<Vec<GitLogEntry>>,
    last_refresh: Instant,
    diff_data: HashMap<String, String>,
    selected_file: Option<String>,
}

impl Default for GitCache {
    fn default() -> Self {
        Self {
            status: None,
            remotes: None,
            log_entries: None,
            last_refresh: Instant::now() - Duration::from_secs(10), // Force initial refresh
            diff_data: HashMap::new(),
            selected_file: None,
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
        self.last_refresh = Instant::now();
        Ok(())
    }

    fn clear_diff_cache(&mut self) {
        self.diff_data.clear();
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
    
    let project_dir = state.project_dir.as_ref().unwrap();
    let git_dir = project_dir.join(".git");
    
    // Check if the project is a git repository
    let is_git_repo = git_dir.exists() && git_dir.is_dir();
    
    // Split the UI into left and right panels
    ui.columns(2, |columns| {
        // Left panel - Repository info and controls
        columns[0].group(|ui| {
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
                                state.error_message = Some(format!("Failed to initialize git repository: {}", e));
                            }
                        }
                    }
                });
            } else {
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
                
                ui.label(format!("Branch: {}", branch));
                
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
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for file in &changed_files {
                                    ui.horizontal(|ui| {
                                        let mut checked = staged_files.contains(file);
                                        if ui.checkbox(&mut checked, file.clone()).changed() {
                                            if checked {
                                                // Stage file
                                                if let Err(e) = stage_file(project_dir, file) {
                                                    state.error_message = Some(format!("Failed to stage file: {}", e));
                                                }
                                                // Clear diff cache when staging status changes
                                                git_state.cache.clear_diff_cache();
                                                // Force refresh on next frame
                                                git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                            } else {
                                                // Unstage file
                                                if let Err(e) = unstage_file(project_dir, file) {
                                                    state.error_message = Some(format!("Failed to unstage file: {}", e));
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
                                    state.error_message = Some(format!("Failed to commit changes: {}", e));
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
                                    state.error_message = Some(format!("Failed to add remote: {}", e));
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
                                            state.error_message = Some(format!("Failed to pull changes: {}", e));
                                        } else {
                                            // Force refresh
                                            git_state.cache.last_refresh = Instant::now() - Duration::from_secs(10);
                                            git_state.cache.clear_diff_cache();
                                        }
                                    }
                                    
                                    if ui.button("Push").clicked() {
                                        if let Err(e) = git_push(project_dir, &remote_name) {
                                            state.error_message = Some(format!("Failed to push changes: {}", e));
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
                    .max_height(200.0).show(ui, |ui| {
                        for entry in &log_entries {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(&entry.hash);
                                    ui.label(&entry.date);
                                });
                                ui.label(&entry.author);
                                ui.label(&entry.message);
                            });
                        }
                    });
                });
            }
        });
        
        // Right panel - Diff viewer (if a file is selected)
        columns[1].group(|ui| {
            if git_state.show_diff_panel {
                ui.vertical(|ui| {
                    ui.heading("Diff Viewer");
                    
                    if let Some(file) = git_state.cache.selected_file.clone() {
                        ui.horizontal(|ui| {
                            ui.heading(&file);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("×").clicked() {
                                    git_state.show_diff_panel = false;
                                }
                            });
                        });
                        
                        ui.separator();
                        
                        // Get diff
                        match get_file_diff(project_dir, &file) {
                            Ok(diff) => {
                                // Store in cache for future reference
                                git_state.cache.diff_data.insert(file.clone(), diff.clone());
                                show_diff_content(ui, &diff);
                            },
                            Err(e) => {
                                ui.label(format!("Error getting diff: {}", e));
                            }
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a file to view its diff");
                        });
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a file to view its changes");
                });
            }
        });
    });
}

// New function to display diff content with syntax highlighting
fn show_diff_content(ui: &mut egui::Ui, diff: &str) {
    egui::ScrollArea::vertical()
        .id_source("diff_content_scroll")
        .max_height(ui.available_height() - 40.0)
        .show(ui, |ui| {
            let lines = diff.lines();
            
            for line in lines {
                if line.starts_with('+') && !line.starts_with("+++") {
                    // Added line
                    ui.colored_label(egui::Color32::from_rgb(0, 128, 0), line);
                } else if line.starts_with('-') && !line.starts_with("---") {
                    // Removed line
                    ui.colored_label(egui::Color32::from_rgb(255, 0, 0), line);
                } else if line.starts_with("@@") {
                    // Hunk header
                    ui.colored_label(egui::Color32::from_rgb(0, 0, 128), line);
                } else {
                    // Context line
                    ui.label(line);
                }
            }
        });
}

// New function to get file diff
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