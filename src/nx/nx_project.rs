use crate::projects::Project;
use anyhow::Result;
use log::debug;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// A struct representing a Nx project
#[derive(Debug, Deserialize)]
pub struct NxProject {
    /// The name of the project
    pub name: Option<String>,
    /// Project's location relative to the root of the workspace
    pub root: Option<String>,
    /// The location of project's sources relative to the root of the workspace
    #[serde(rename = "sourceRoot")]
    pub source_root: Option<String>,
    /// Type of project supported
    #[serde(rename = "projectType")]
    pub project_type: Option<ProjectType>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "implicitDependencies")]
    pub implicit_dependencies: Option<Vec<String>>,
}

/// An enum representing the type of project
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Library,
    Application,
}

impl Project for NxProject {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn load(workspace_root: &Path, project_path: &str) -> Result<Self> {
        let path = workspace_root.join(project_path).join("project.json");
        debug!("Loading project from {:?}", path);
        let contents = fs::read_to_string(path).expect("Could not read project.json");
        let mut project: NxProject =
            serde_json::from_str(&contents).expect("Could not parse project.json");

        if project.root.is_none() {
            project.root = Some(project_path.to_string());
        }

        Ok(project)
    }
}
