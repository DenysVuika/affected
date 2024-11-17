use crate::graph::{NodeType, ProjectNode};
use crate::nx::NxProject;
use crate::projects::Project;
use crate::utils::inspect_workspace;
use crate::Config;
use anyhow::{bail, Context, Result};
use git2::{BranchType, DiffOptions, Repository};
use log::{debug, warn};
use petgraph::Graph;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub type WorkspaceGraph = Graph<NodeType, ()>;

#[derive(Default)]
pub struct Workspace {
    pub root: PathBuf,

    config: Option<Config>,
    repo: Option<Repository>,
    graph: Option<WorkspaceGraph>,

    affected_files: Option<HashSet<String>>,
    affected_projects: Option<HashSet<String>>,
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

    pub fn graph(&self) -> Option<&WorkspaceGraph> {
        self.graph.as_ref()
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
        self.build_projects_graph()?;

        Ok(())
    }

    pub fn affected_files(&self) -> Result<HashSet<String>> {
        if let Some(files) = &self.affected_files {
            Ok(files.clone())
        } else {
            Ok(HashSet::new())
        }
    }

    pub fn affected_projects(&self) -> Result<HashSet<String>> {
        if let Some(projects) = &self.affected_projects {
            Ok(projects.clone())
        } else {
            Ok(HashSet::new())
        }
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

    pub fn is_nx_project_dir(path: &Path) -> bool {
        path.is_dir() && path.join("project.json").is_file()
    }

    fn build_projects_graph(&mut self) -> Result<()> {
        let mut graph = WorkspaceGraph::new();
        let mut project_indices = HashMap::new();

        // todo: support package.json
        let projects = inspect_workspace(&self.root, Workspace::is_nx_project_dir)?;
        if projects.is_empty() {
            return Ok(());
        }

        let affected_files = get_affected_files(self)?;
        if affected_files.is_empty() {
            return Ok(());
        }

        // todo: insert file nodes into the graph

        let mut affected_projects = HashSet::new();

        for project in &projects {
            debug!("Project: {:?}", project);
            // todo: support package.json
            let nx_project = NxProject::load(&self.root, project)?;
            let project_name = nx_project.name().unwrap_or("Unnamed");
            let project_path = nx_project.source_root.clone();

            let project_node = graph.add_node(NodeType::Project(ProjectNode {
                name: project_name.to_string(),
                path: project_path,
                implicit_dependencies: nx_project.implicit_dependencies.clone(),
            }));

            project_indices.insert(project_name.to_string(), project_node);

            // find affected projects
            for file in &affected_files {
                if file.starts_with(project) {
                    // todo: link file nodes to project nodes
                    // println!("Affected project: {}", project);
                    affected_projects.insert(project.clone());
                }
            }
        }

        // update the graph with implicit dependencies

        for node_index in graph.node_indices() {
            let node = graph[node_index].clone();

            if let NodeType::Project(project_node) = node {
                if let Some(dependencies) = &project_node.implicit_dependencies {
                    for dependency in dependencies {
                        if let Some(dependency_node) = project_indices.get(dependency) {
                            graph.add_edge(node_index, *dependency_node, ());
                        } else {
                            warn!("Dependency {} not found", dependency);
                        }
                    }
                }
            }
        }

        self.graph = Some(graph);
        self.affected_files = Some(affected_files);
        self.affected_projects = Some(affected_projects);

        Ok(())
    }
}

fn get_affected_files(workspace: &Workspace) -> Result<HashSet<String>> {
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
