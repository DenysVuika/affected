use crate::graph::WorkspaceGraph;
use crate::utils::inspect_workspace;
use crate::Config;
use anyhow::{bail, Context, Result};
use git2::{BranchType, DiffOptions, Repository};
use log::debug;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Workspace {
    pub root: PathBuf,

    config: Option<Config>,
    repo: Option<Repository>,
    graph: Option<WorkspaceGraph>,
}

impl Workspace {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            ..Default::default()
        }
    }

    /// Creates a new workspace with a configuration
    pub fn with_config(root: impl Into<PathBuf>, config: Config) -> Self {
        Self {
            root: root.into(),
            config: Some(config),
            ..Default::default()
        }
    }

    pub fn config(&self) -> Option<&Config> {
        self.config.as_ref()
    }

    pub fn repo(&self) -> Option<&Repository> {
        self.repo.as_ref()
    }

    /// Loads the repository
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
        self.graph = Some(build_projects_graph(self)?);

        Ok(())
    }

    pub fn affected_files(&self) -> Result<HashSet<String>> {
        let affected_files = get_affected_files(self)?;
        Ok(affected_files)
    }

    pub fn affected_projects(&self) -> Result<HashSet<String>> {
        let affected_projects = get_affected_projects(&self)?;
        Ok(affected_projects)
    }

    /// Returns a list of tasks defined in the configuration
    pub fn tasks(&self) -> Vec<String> {
        let config = self.config.as_ref().expect("Configuration not loaded");

        if let Some(tasks) = &config.tasks {
            tasks.iter().map(|task| task.name.clone()).collect()
        } else {
            vec![]
        }
    }

    pub async fn run_task(&self, task_name: &str) -> Result<()> {
        crate::tasks::run_task_by_name(self, task_name).await
    }

    pub fn is_project_dir(path: &Path) -> bool {
        path.is_dir()
            && (
                path.join("project.json").is_file() || path.join("package.json").is_file()
                // || path.join("Cargo.toml").is_file()
            )
    }
}

pub fn get_affected_files(workspace: &Workspace) -> Result<HashSet<String>> {
    let repo = workspace.repo.as_ref().expect("Repository not loaded");
    let config = workspace.config.as_ref().expect("Configuration not loaded");

    // Get the current branch (HEAD)
    let head = repo.head().context("Could not retrieve HEAD")?;
    let current_branch = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("Could not determine current branch"))?;
    debug!("Current branch: {}", current_branch);

    // Get the OIDs (object IDs) for the current branch and the main branch
    // let current_oid = head.target().context("Could not get current branch OID")?;
    // debug!("Current OID: {}", current_oid);

    let base: Option<&str> = config.base.as_deref();

    let base_branch = if let Some(main) = base {
        if repo.find_branch(main, BranchType::Local).is_ok() {
            main
        } else {
            bail!("Could not find the specified base branch '{}'", main);
        }
    } else if repo.find_branch("main", BranchType::Local).is_ok() {
        "main"
    } else if repo.find_branch("master", BranchType::Local).is_ok() {
        "master"
    } else {
        bail!("Could not find 'main' or 'master' branch");
    };
    debug!("Base branch: {}", base_branch);

    let main_ref = format!("refs/heads/{}", base_branch);
    debug!("Base ref: {}", main_ref);

    let main_oid = repo
        .revparse_single(&main_ref)
        .context("Could not find the base branch OID")?
        .id();

    debug!("Base OID: {}", main_oid);

    // Get the trees for each branch's commit
    // let current_tree = repo.find_commit(current_oid)?.tree()?;
    let base_tree = repo.find_commit(main_oid)?.tree()?;

    // Compare the trees to get the diff
    let mut diff_opts = DiffOptions::new();
    // let diff =
    //     repo.diff_tree_to_tree(Some(&base_tree), Some(&current_tree), Some(&mut diff_opts))?;
    let diff = repo.diff_tree_to_workdir_with_index(Some(&base_tree), Some(&mut diff_opts))?;

    let mut result = HashSet::new();

    for delta in diff.deltas() {
        if let Some(path) = delta.new_file().path() {
            result.insert(path.to_string_lossy().to_string());
        }
    }

    Ok(result)
}

fn get_affected_projects(workspace: &Workspace) -> Result<HashSet<String>> {
    let affected_files: HashSet<_> = get_affected_files(workspace)?.into_iter().collect();
    if affected_files.is_empty() {
        return Ok(HashSet::new());
    }

    let mut projects = inspect_workspace(&workspace.root, Workspace::is_project_dir)?;
    if projects.is_empty() {
        return Ok(HashSet::new());
    }
    projects.retain(|project| {
        if affected_files.iter().any(|file| file.starts_with(project)) {
            true
        } else {
            debug!("Skipping project '{}'", project);
            false
        }
    });

    Ok(projects)
}

fn build_projects_graph(workspace: &Workspace) -> Result<WorkspaceGraph> {
    let mut graph = WorkspaceGraph::new();
    // let node_indices = HashMap::new();

    Ok(graph)
}
