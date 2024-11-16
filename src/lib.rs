mod config;
pub mod graph;
pub mod logger;
mod node;
pub mod nx;
mod projects;
pub mod tasks;
mod utils;
pub mod workspace;

use crate::projects::{is_project_dir, Project};
use crate::utils::inspect_workspace;
use anyhow::{bail, Context, Result};
pub use config::Config;
use git2::{BranchType, DiffOptions, Repository};
use log::debug;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn get_affected_files(repo: &Repository, config: &Config) -> Result<Vec<String>> {
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

    let mut result = vec![];

    for delta in diff.deltas() {
        if let Some(path) = delta.new_file().path() {
            result.push(path.to_string_lossy().to_string());
        }
    }

    Ok(result)
}

pub fn get_affected_projects(
    workspace_root: &PathBuf,
    repo: &Repository,
    config: &Config,
) -> Result<Vec<String>> {
    let affected_files: HashSet<_> = get_affected_files(repo, config)?.into_iter().collect();
    if affected_files.is_empty() {
        return Ok(vec![]);
    }

    let mut projects = inspect_workspace(workspace_root, is_project_dir)?;
    if projects.is_empty() {
        return Ok(vec![]);
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

pub fn get_all_projects(workspace_root: &PathBuf) -> Result<Vec<String>> {
    let filter_fn = |path: &Path| path.is_dir() && path.join("project.json").is_file();
    let projects = inspect_workspace(workspace_root, filter_fn)?;

    Ok(projects)
}

pub fn get_project(workspace_root: &Path, project_path: &str) -> Result<Box<dyn Project>> {
    let project_root = workspace_root.join(project_path);
    let project_json_path = project_root.join("project.json");
    let package_json_path = project_root.join("package.json");

    if project_json_path.is_file() {
        let nx_proj = nx::NxProject::load(workspace_root, project_path)?;
        debug!("{:?}", nx_proj);
        Ok(Box::new(nx_proj))
    } else if package_json_path.is_file() {
        let node_proj = node::NodeProject::load(workspace_root, project_path)?;
        debug!("{:?}", node_proj);
        Ok(Box::new(node_proj))
    } else {
        bail!("Could not find 'project.json' or 'package.json' in the project directory");
    }
}
