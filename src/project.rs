use anyhow::Result;
use std::path::Path;

/// A trait for defining a project.
pub trait Project {
    fn name(&self) -> Option<&str>;
    fn load(workspace_root: &Path, project_path: &str) -> Result<Self>
    where
        Self: Sized;
}
