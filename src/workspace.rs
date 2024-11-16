use crate::Config;
use anyhow::Result;
use git2::Repository;
use std::path::PathBuf;

pub struct Workspace {
    pub root: PathBuf,

    config: Option<Config>,
    repo: Option<Repository>,
}

impl Workspace {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            config: None,
            repo: None,
        }
    }

    pub fn with_config(root: impl Into<PathBuf>, config: Config) -> Self {
        Self {
            root: root.into(),
            config: Some(config),
            repo: None,
        }
    }

    pub fn config(&self) -> Option<&Config> {
        self.config.as_ref()
    }

    pub fn repo(&self) -> Option<&Repository> {
        self.repo.as_ref()
    }

    pub async fn load(&mut self) -> Result<()> {
        let repo = Repository::open(&self.root).expect("Could not open the repository");
        self.repo = Some(repo);

        Ok(())
    }
}
