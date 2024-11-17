use crate::Config;
use anyhow::Result;
use git2::Repository;
use std::path::PathBuf;

pub struct Workspace {
    pub root: PathBuf,

    config: Option<Config>,
    repo: Option<Repository>,

    cached_affected_files: Option<Vec<String>>,
    cached_affected_projects: Option<Vec<String>>,
}

impl Workspace {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            config: None,
            repo: None,
            cached_affected_files: None,
            cached_affected_projects: None,
        }
    }

    pub fn with_config(root: impl Into<PathBuf>, config: Config) -> Self {
        Self {
            root: root.into(),
            config: Some(config),
            repo: None,
            cached_affected_files: None,
            cached_affected_projects: None,
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

        // TODO: introduce flag to fetch from remote
        // Fetch the latest changes from the remote repository
        // let mut remote = repo
        //     .find_remote("origin")
        //     .context("Could not find remote 'origin'")?;
        // remote
        //     .fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None)
        //     .context("Failed to fetch from remote repository")?;

        self.repo = Some(repo);

        Ok(())
    }

    pub fn affected_files(&mut self) -> Result<Vec<String>> {
        if let Some(ref cached_files) = self.cached_affected_files {
            return Ok(cached_files.clone());
        }

        let repo = self.repo.as_ref().expect("Repository not loaded");
        let config = self.config.as_ref().expect("Configuration not loaded");

        let affected_files = crate::get_affected_files(repo, config)?;
        self.cached_affected_files = Some(affected_files.clone());

        Ok(affected_files)
    }

    pub fn affected_projects(&mut self) -> Result<Vec<String>> {
        if let Some(ref cached_projects) = self.cached_affected_projects {
            return Ok(cached_projects.clone());
        }

        let repo = self.repo.as_ref().expect("Repository not loaded");
        let config = self.config.as_ref().expect("Configuration not loaded");

        let affected_projects = crate::get_affected_projects(&self.root, repo, config)?;
        self.cached_affected_projects = Some(affected_projects.clone());

        Ok(affected_projects)
    }
}
