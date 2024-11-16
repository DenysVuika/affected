use anyhow::Result;
use std::path::Path;

/// A trait for defining a project.
pub trait Project {
    fn name(&self) -> Option<&str>;
    fn load(workspace_root: &Path, project_path: &str) -> Result<Self>
    where
        Self: Sized;
}

pub fn is_project_dir(path: &Path) -> bool {
    path.is_dir()
        && (
            path.join("project.json").is_file() || path.join("package.json").is_file()
            // || path.join("Cargo.toml").is_file()
        )
}
