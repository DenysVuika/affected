mod config;
pub mod logger;
mod node;
pub mod nx;
mod project;
mod utils;

use crate::project::Project;
use crate::utils::parse_workspace;
use anyhow::{bail, Context, Result};
pub use config::Config;
use git2::{BranchType, DiffOptions, Repository};
use glob::Pattern;
use log::debug;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn list_affected_files(repo: &Repository, config: &Config) -> Result<Vec<String>> {
    // Get the current branch (HEAD)
    let head = repo.head().context("Could not retrieve HEAD")?;
    let current_branch = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("Could not determine current branch"))?;
    debug!("Current branch: {}", current_branch);

    // Get the OIDs (object IDs) for the current branch and the main branch
    let current_oid = head.target().context("Could not get current branch OID")?;
    debug!("Current OID: {}", current_oid);

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

fn is_project_dir(path: &Path) -> bool {
    path.is_dir()
        && (
            path.join("project.json").is_file() || path.join("package.json").is_file()
            // || path.join("Cargo.toml").is_file()
        )
}

// TODO: provide a way to specify the display options: name as folder, package.json, project.json, etc.
pub fn list_affected_projects(
    workspace_root: &PathBuf,
    repo: &Repository,
    config: &Config,
) -> Result<Vec<String>> {
    let projects = parse_workspace(workspace_root, is_project_dir)?;
    let mut affected_projects = HashSet::new();

    if !projects.is_empty() {
        let affected_files: HashSet<_> = list_affected_files(repo, config)?.into_iter().collect();
        // Check if any of the affected files are in the projects
        for project in projects {
            if affected_files.iter().any(|file| file.starts_with(&project)) {
                affected_projects.insert(project);
            } else {
                debug!("Skipping project '{}'", project);
            }
        }
    }

    Ok(affected_projects.into_iter().collect())
}

pub fn list_all_projects(
    workspace_root: &PathBuf,
    _repo: &Repository,
    _config: &Config,
) -> Result<()> {
    let filter_fn = |path: &Path| path.is_dir() && path.join("project.json").is_file();
    let projects = parse_workspace(workspace_root, filter_fn)?;

    for project in projects {
        println!("{}", project);
    }
    Ok(())
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

pub fn run_task_by_name(
    workspace_root: &Path,
    repo: &Repository,
    config: &Config,
    task_name: &str,
) -> Result<()> {
    debug!("Running task: {}", task_name);

    let task = config.get_task(task_name).context("Task not found")?;
    let file_paths = list_affected_files(repo, config)?;

    // filter out files that exist on the filesystem
    let file_paths: Vec<_> = file_paths
        .into_iter()
        .filter(|path| workspace_root.join(path).exists())
        .collect();

    if file_paths.is_empty() {
        debug!("No files affected");
        return Ok(());
    }

    let filtered_paths: Vec<_> = file_paths
        .into_iter()
        .filter(|path| {
            task.patterns.iter().any(|pattern| {
                Pattern::new(pattern)
                    .map(|p| p.matches(path))
                    .unwrap_or(false)
            })
        })
        .collect();

    if filtered_paths.is_empty() {
        println!("No files matched the patterns");
        return Ok(());
    }

    debug!("Filtered files:");
    for path in &filtered_paths {
        debug!("- {}", path);
    }

    let files = &filtered_paths.join(" ");

    for command_template in &task.commands {
        let command_text = command_template.replace("{files}", files);
        debug!("Running command: {}", &command_text);

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command_text)
            .current_dir(workspace_root)
            .output()
            .context("Failed to run the command")?;

        if !output.status.success() {
            bail!("Command failed: {}", &command_text);
        }

        debug!("{}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}
