use crate::project::Project;
use anyhow::Result;
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

    fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let project: NxProject = serde_json::from_str(&contents)?;
        Ok(project)
    }
}
