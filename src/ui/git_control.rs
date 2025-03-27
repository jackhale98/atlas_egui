// src/ui/git_control.rs
use eframe::egui;
use std::process::Command;
use std::io::Write;
use std::path::Path;
use crate::state::AppState;

pub fn show_git_control(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Git Version Control");
    
    if state.project_dir.is_none() {
        ui.label("No project directory selected. Please open or create a project first.");
        return;
    }
    
    let project_dir = state.project_dir.as_ref().unwrap();
    let git_dir = project_dir.join(".git");
    
    // Check if the project is a git repository
    let is_git_repo = git_dir.exists() && git_dir.is_dir();
    
    ui.group(|ui| {
        ui.heading("Repository Status");
        
        if !is_git_repo {
            ui.horizontal(|ui| {
                ui.label("This project is not yet under version control.");
                if ui.button("Initialize Git Repository").clicked() {
                    match initialize_git_repo(project_dir) {
                        Ok(_) => {
                            // Success, refresh status
                        },
                        Err(e) => {
                            state.error_message = Some(format!("Failed to initialize git repository: {}", e));
                        }
                    }
                }
            });
        } else {
            // Get repository status
            match get_git_status(project_dir) {
                Ok(status) => {
                    ui.label(format!("Branch: {}", status.branch));
                    
                    ui.add_space(10.0);
                    
                    // Show changed files
                    ui.group(|ui| {
                        ui.heading("Changed Files");
                        
                        if status.changed_files.is_empty() {
                            ui.label("No changes detected");
                        } else {
                            // Make the file list scrollable with a fixed height
                            egui::ScrollArea::vertical()
                                .id_source("git_changed_files_scroll") // Use a unique ID
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    for file in &status.changed_files {
                                        let mut checked = status.staged_files.contains(file);
                                        if ui.checkbox(&mut checked, file).changed() {
                                            if checked {
                                                // Stage file
                                                if let Err(e) = stage_file(project_dir, file) {
                                                    state.error_message = Some(format!("Failed to stage file: {}", e));
                                                }
                                            } else {
                                                // Unstage file
                                                if let Err(e) = unstage_file(project_dir, file) {
                                                    state.error_message = Some(format!("Failed to unstage file: {}", e));
                                                }
                                            }
                                        }
                                    }
                                });
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    // Commit area
                    ui.group(|ui| {
                        ui.heading("Commit Changes");
                        
                        static mut COMMIT_MESSAGE: String = String::new();
                        
                        // Safety: This is not thread-safe, but egui runs in a single thread
                        let commit_message = unsafe { &mut COMMIT_MESSAGE };
                        
                        ui.label("Commit Message:");
                        ui.text_edit_multiline(commit_message);
                        
                        ui.horizontal(|ui| {
                            let can_commit = !status.staged_files.is_empty() && !commit_message.trim().is_empty();
                            if ui.add_enabled(can_commit, egui::Button::new("Commit")).clicked() {
                                match commit_changes(project_dir, commit_message) {
                                    Ok(_) => {
                                        // Clear commit message after successful commit
                                        commit_message.clear();
                                    },
                                    Err(e) => {
                                        state.error_message = Some(format!("Failed to commit changes: {}", e));
                                    }
                                }
                            }
                            
                            if ui.button("Refresh Status").clicked() {
                                // Status will refresh on next frame
                            }
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Remote repository operations
                    ui.group(|ui| {
                        ui.heading("Remote Repository");
                        
                        // Get remotes
                        match get_git_remotes(project_dir) {
                            Ok(remotes) => {
                                if remotes.is_empty() {
                                    ui.label("No remote repositories configured.");
                                    static mut REMOTE_URL: String = String::new();
                                    static mut REMOTE_NAME: String = String::new();
                                    
                                    // Safety: This is not thread-safe, but egui runs in a single thread
                                    let remote_url = unsafe { &mut REMOTE_URL };
                                    let remote_name = unsafe { &mut REMOTE_NAME };
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        ui.text_edit_singleline(remote_name);
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("URL:");
                                        ui.text_edit_singleline(remote_url);
                                    });
                                    
                                    let can_add = !remote_url.trim().is_empty() && !remote_name.trim().is_empty();
                                    if ui.add_enabled(can_add, egui::Button::new("Add Remote")).clicked() {
                                        match add_git_remote(project_dir, remote_name, remote_url) {
                                            Ok(_) => {
                                                // Clear fields after successful add
                                                remote_name.clear();
                                                remote_url.clear();
                                            },
                                            Err(e) => {
                                                state.error_message = Some(format!("Failed to add remote: {}", e));
                                            }
                                        }
                                    }
                                } else {
                                    for remote in &remotes {
                                        ui.horizontal(|ui| {
                                            ui.label(&remote.name);
                                            ui.label(remote.url.clone());
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("Pull").clicked() {
                                                    if let Err(e) = git_pull(project_dir, &remote.name) {
                                                        state.error_message = Some(format!("Failed to pull changes: {}", e));
                                                    }
                                                }
                                                
                                                if ui.button("Push").clicked() {
                                                    if let Err(e) = git_push(project_dir, &remote.name) {
                                                        state.error_message = Some(format!("Failed to push changes: {}", e));
                                                    }
                                                }
                                            });
                                        });
                                    }
                                }
                            },
                            Err(e) => {
                                ui.label(format!("Failed to get remote repositories: {}", e));
                            }
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    // Commit history
                    ui.group(|ui| {
                        ui.heading("Commit History");
                        
                        match get_git_log(project_dir) {
                            Ok(log_entries) => {
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
                            },
                            Err(e) => {
                                ui.label(format!("Failed to get commit history: {}", e));
                            }
                        }
                    });
                },
                Err(e) => {
                    ui.label(format!("Failed to get repository status: {}", e));
                }
            }
        }
    });
}

// Git operation structures
struct GitStatus {
    branch: String,
    changed_files: Vec<String>,
    staged_files: Vec<String>,
}

struct GitRemote {
    name: String,
    url: String,
}

struct GitLogEntry {
    hash: String,
    author: String,
    date: String,
    message: String,
}

// Git operations
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

// src/ui/git_control.rs - Update the unstage_file function
fn unstage_file(project_dir: &Path, file: &str) -> Result<(), String> {
    // Fix the unstage command to use "--" to disambiguate paths
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