use crate::Config;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Workspace {
    pub root: PathBuf,
    config: Option<Config>,
}

impl Workspace {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            config: None,
        }
    }

    pub fn with_config(root: impl Into<PathBuf>, config: Config) -> Self {
        Self {
            root: root.into(),
            config: Some(config),
        }
    }

    pub fn config(&self) -> Option<&Config> {
        self.config.as_ref()
    }
}
