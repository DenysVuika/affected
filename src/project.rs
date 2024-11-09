use anyhow::Result;
use std::path::Path;

/// A trait for defining a project.
pub trait Project {
    fn name(&self) -> &str;
    fn load(path: &Path) -> Result<Self>
    where
        Self: Sized;
}
