use crate::projects::Project;
use crate::workspace::Workspace;
use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct NodeProject {
    pub name: Option<String>,
}

impl Project for NodeProject {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn load(workspace: &Workspace, project_path: &str) -> Result<Self> {
        let path = workspace.root.join(project_path).join("package.json");
        let contents = fs::read_to_string(path)?;
        let project: NodeProject = serde_json::from_str(&contents)?;
        Ok(project)
    }
}
