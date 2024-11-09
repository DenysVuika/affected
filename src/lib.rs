use anyhow::{bail, Context, Result};
use git2::{BranchType, DiffOptions, Repository};
use log::{debug, info};

pub fn list_all_targets(repo: &Repository, main: Option<String>) -> Result<()> {
    // Open the Git repository in the current directory
    // let repo = Repository::open(workspace_root).context("Could not open the repository")?;

    // TODO: introduce flag to fetch from remote
    // Fetch the latest changes from the remote repository
    // let mut remote = repo
    //     .find_remote("origin")
    //     .context("Could not find remote 'origin'")?;
    // remote
    //     .fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None)
    //     .context("Failed to fetch from remote repository")?;

    // Get the current branch (HEAD)
    let head = repo.head().context("Could not retrieve HEAD")?;
    let current_branch = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("Could not determine current branch"))?;
    debug!("Current branch: {}", current_branch);

    // Get the OIDs (object IDs) for the current branch and the main branch
    let current_oid = head.target().context("Could not get current branch OID")?;
    debug!("Current OID: {}", current_oid);

    let main_branch = if let Some(main) = main {
        if repo.find_branch(&main, BranchType::Local).is_ok() {
            main
        } else {
            bail!("Could not find the specified main branch '{}'", main);
        }
    } else if repo.find_branch("main", BranchType::Local).is_ok() {
        "main".to_string()
    } else if repo.find_branch("master", BranchType::Local).is_ok() {
        "master".to_string()
    } else {
        bail!("Could not find 'main' or 'master' branch");
    };
    debug!("Main branch: {}", main_branch);

    let main_ref = format!("refs/heads/{}", main_branch);
    debug!("Main ref: {}", main_ref);

    let main_oid = repo
        .revparse_single(&main_ref)
        .context("Could not find the main branch OID")?
        .id();

    debug!("Main OID: {}", main_oid);

    // Get the trees for each branch's commit
    let current_tree = repo.find_commit(current_oid)?.tree()?;
    let main_tree = repo.find_commit(main_oid)?.tree()?;

    // Compare the trees to get the diff
    let mut diff_opts = DiffOptions::new();
    let diff =
        repo.diff_tree_to_tree(Some(&main_tree), Some(&current_tree), Some(&mut diff_opts))?;

    // Iterate over the diff entries and print the file paths
    for delta in diff.deltas() {
        if let Some(path) = delta.new_file().path() {
            println!("{}", path.display());
        }
    }

    Ok(())
}

pub fn list_projects(_repo: &Repository, _main: Option<String>) -> Result<()> {
    info!("Projects (coming soon)");
    Ok(())
}
