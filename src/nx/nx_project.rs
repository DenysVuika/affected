use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct NxProject {
    pub name: String,
}

impl NxProject {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let project: NxProject = serde_json::from_str(&contents)?;

        Ok(project)
    }
}
