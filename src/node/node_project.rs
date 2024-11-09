use crate::project::Project;
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct NodeProject {
    pub name: String,
}

impl Project for NodeProject {
    fn name(&self) -> &str {
        &self.name
    }

    fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let project: NodeProject = serde_json::from_str(&contents)?;
        Ok(project)
    }
}
