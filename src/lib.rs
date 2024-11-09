mod utils;

use crate::utils::parse_workspace;
use anyhow::{bail, Context, Result};
use git2::{BranchType, DiffOptions, Repository};
use log::debug;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn list_affected_files(repo: &Repository, base: Option<String>) -> Result<Vec<String>> {
    // Get the current branch (HEAD)
    let head = repo.head().context("Could not retrieve HEAD")?;
    let current_branch = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("Could not determine current branch"))?;
    debug!("Current branch: {}", current_branch);

    // Get the OIDs (object IDs) for the current branch and the main branch
    let current_oid = head.target().context("Could not get current branch OID")?;
    debug!("Current OID: {}", current_oid);

    let base_branch = if let Some(main) = base {
        if repo.find_branch(&main, BranchType::Local).is_ok() {
            main
        } else {
            bail!("Could not find the specified base branch '{}'", main);
        }
    } else if repo.find_branch("main", BranchType::Local).is_ok() {
        "main".to_string()
    } else if repo.find_branch("master", BranchType::Local).is_ok() {
        "master".to_string()
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
    let current_tree = repo.find_commit(current_oid)?.tree()?;
    let base_tree = repo.find_commit(main_oid)?.tree()?;

    // Compare the trees to get the diff
    let mut diff_opts = DiffOptions::new();
    let diff =
        repo.diff_tree_to_tree(Some(&base_tree), Some(&current_tree), Some(&mut diff_opts))?;

    let mut result = vec![];

    // Iterate over the diff entries and print the file paths
    for delta in diff.deltas() {
        if let Some(path) = delta.new_file().path() {
            result.push(path.to_string_lossy().to_string());
        }
    }

    Ok(result)
}

pub fn list_affected_projects(
    workspace_root: &PathBuf,
    repo: &Repository,
    main: Option<String>,
) -> Result<Vec<String>> {
    let filter_fn = |path: &Path| path.is_dir() && path.join("project.json").is_file();
    let projects = parse_workspace(workspace_root, filter_fn)?;
    let mut affected_projects = HashSet::new();

    if !projects.is_empty() {
        let affected_files = list_affected_files(repo, main)?;
        // Check if any of the affected files are in the projects
        for project in projects {
            for file in &affected_files {
                if file.starts_with(&project) {
                    affected_projects.insert(project.clone());
                    break;
                }
            }
        }
    }

    Ok(affected_projects.into_iter().collect())
}

pub fn list_all_projects(
    workspace_root: &PathBuf,
    _repo: &Repository,
    _main: Option<String>,
) -> Result<()> {
    let filter_fn = |path: &Path| path.is_dir() && path.join("project.json").is_file();
    let projects = parse_workspace(workspace_root, filter_fn)?;

    for project in projects {
        println!("{}", project);
    }
    Ok(())
}
