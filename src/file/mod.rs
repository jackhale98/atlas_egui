// src/file/mod.rs
use anyhow::{Result, Context, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use crate::config::{ProjectFile, Component};
use crate::file::mates::MatesFile;
use crate::config::project::AnalysisReference;
use crate::analysis::stackup::{StackupAnalysis, AnalysisResults};
use crate::config::ComponentReference;

pub mod project;
pub mod component;
pub mod mates;
pub mod analysis;

// Core trait for file operations
pub trait FileHandler<T> {
    fn load(&self, path: &Path) -> Result<T>;
    fn save(&self, data: &T, path: &Path) -> Result<()>;
}


#[derive(Debug)]
pub struct FileManager {
    project_dir: Option<PathBuf>,
    project_handler: project::ProjectFileHandler,
    component_handler: component::ComponentFileHandler,
    mates_handler: mates::MatesFileHandler,
    pub analysis_handler: analysis::AnalysisFileManager,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            project_dir: None,
            project_handler: project::ProjectFileHandler::new(),
            component_handler: component::ComponentFileHandler::new(),
            mates_handler: mates::MatesFileHandler::new(),
            analysis_handler: analysis::AnalysisFileManager::new(Path::new("")),
        }
    }

    pub fn set_project_dir(&mut self, path: PathBuf) -> Result<()> {
        // Verify the path exists and is a directory
        if !path.exists() {
            return Err(anyhow!("Project directory does not exist: {}", path.display()));
        }
        if !path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", path.display()));
        }

        self.project_dir = Some(path.clone());
        self.analysis_handler = analysis::AnalysisFileManager::new(&path);

        // Create project structure but don't fail if directories already exist
        if let Err(e) = self.create_project_structure() {
            eprintln!("Warning: Failed to create some project directories: {}", e);
        }

        Ok(())
    }

    pub fn create_project_structure(&self) -> Result<()> {
        if let Some(project_dir) = &self.project_dir {
            fs::create_dir_all(project_dir)?;
            fs::create_dir_all(project_dir.join("components"))?;
            fs::create_dir_all(project_dir.join("analyses"))?;
            fs::create_dir_all(project_dir.join("analyses/oring"))?;
            fs::create_dir_all(project_dir.join("analyses/stackups"))?;
            Ok(())
        } else {
            Err(anyhow!("No project directory set"))
        }
    }

    pub fn load_project(&self, path: &Path) -> Result<(ProjectFile, Vec<Component>, MatesFile, Vec<(StackupAnalysis, Option<AnalysisResults>)>)> {
        // First verify the project file exists
        if !path.exists() {
            return Err(anyhow!("Project file not found: {}", path.display()));
        }

        let project_file = self.project_handler.load(path)?;
        let mut components = Vec::new();
        let mut analyses = Vec::new();

        let project_dir = path.parent()
            .ok_or_else(|| anyhow!("Invalid project path: {}", path.display()))?;

        // Load components with better error handling
        for comp_ref in &project_file.component_references {
            let normalized_path = comp_ref.path.replace('\\', "/");
            let comp_path = project_dir.join(normalized_path);

            if !comp_path.exists() {
                return Err(anyhow!(
                    "Component file not found: {}. Project dir: {}",
                    comp_path.display(),
                    project_dir.display()
                ));
            }

            let component = self.component_handler.load(&comp_path)
                .with_context(|| format!("Failed to load component from {}", comp_path.display()))?;
            components.push(component);
        }

        // Load analyses with better error handling
        for analysis_ref in &project_file.analyses {
            let normalized_path = analysis_ref.path.replace('\\', "/");
            let analysis_path = project_dir.join(normalized_path);

            // Only try to load if the analysis file exists
            if analysis_path.exists() {
                let analysis_dir = analysis_path.parent()
                    .ok_or_else(|| anyhow!("Invalid analysis path: {}", analysis_path.display()))?;
                let analysis_id = analysis_dir.file_name()
                    .ok_or_else(|| anyhow!("Invalid analysis directory: {}", analysis_dir.display()))?
                    .to_string_lossy();

                match self.analysis_handler.load_analysis(&analysis_id) {
                    Ok((analysis, results)) => analyses.push((analysis, results)),
                    Err(e) => {
                        // Log the error but continue loading other analyses
                        eprintln!("Warning: Failed to load analysis {}: {}", analysis_id, e);
                    }
                }
            }
        }

        let mates_path = project_dir.join("mates.ron");
        // Create empty mates file if it doesn't exist
        let mates_file = if mates_path.exists() {
            self.mates_handler.load(&mates_path)?
        } else {
            MatesFile::new()
        };

        Ok((project_file, components, mates_file, analyses))
    }

    pub fn save_project(&mut self, project_file: &ProjectFile, components: &[Component], analyses: &[StackupAnalysis]) -> Result<()> {
        if let Some(project_dir) = &self.project_dir {
            // Update analysis references with platform-independent paths
            let mut updated_project = project_file.clone();
            updated_project.analyses = analyses.iter().map(|analysis| {
                AnalysisReference {
                    // Use forward slashes for paths in RON files
                    path: format!("analyses/stackups/{}/analysis.ron", analysis.id),
                    analysis_type: "stackup".to_string(),
                }
            }).collect();

            let project_path = project_dir.join("project.ron");
            self.project_handler.save(&updated_project, &project_path)?;

            // Save components
            let components_dir = project_dir.join("components");
            fs::create_dir_all(&components_dir)?;

            for component in components {
                let filename = format!("{}.ron", component.name.to_lowercase().replace(" ", "_"));
                // Always use forward slashes when storing paths
                let rel_path = format!("components/{}", filename).replace('\\', "/");
                let comp_path = components_dir.join(&filename);
                self.component_handler.save(component, &comp_path)?;

                // Update component reference in project file to use forward slashes
                if !updated_project.component_references.iter().any(|r| r.path == rel_path) {
                    updated_project.component_references.push(ComponentReference {
                        path: rel_path,
                    });
                }
            }

            Ok(())
        } else {
            Err(anyhow!("No project directory set"))
        }
    }

    pub fn save_mates(&self, mates: &mates::MatesFile) -> Result<()> {
        let project_dir = self.project_dir
            .as_ref()
            .ok_or_else(|| anyhow!("No project directory set"))?;

        self.mates_handler.save(mates, &project_dir.join("mates.ron"))
    }

    pub fn load_mates(&self) -> Result<mates::MatesFile> {
        let project_dir = self.project_dir
            .as_ref()
            .ok_or_else(|| anyhow!("No project directory set"))?;

        self.mates_handler.load(&project_dir.join("mates.ron"))
    }
}
