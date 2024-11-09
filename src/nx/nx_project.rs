use crate::project::Project;
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// A struct representing a Nx project
#[derive(Debug, Deserialize)]
pub struct NxProject {
    /// The name of the project
    pub name: String,
    /// The type of project. Can be either `library` or `application`
    #[serde(rename = "projectType")]
    pub project_type: ProjectType,
}

/// An enum representing the type of project
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Library,
    Application,
}

impl Project for NxProject {
    fn name(&self) -> &str {
        &self.name
    }

    fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let project: NxProject = serde_json::from_str(&contents)?;
        Ok(project)
    }
}
