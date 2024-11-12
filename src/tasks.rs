use crate::{get_affected_files, Config};
use anyhow::{bail, Context, Result};
use git2::Repository;
use glob::Pattern;
use log::debug;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_task_by_name(
    workspace_root: &Path,
    repo: &Repository,
    config: &Config,
    task_name: &str,
) -> Result<()> {
    debug!("Running task: {}", task_name);

    let task = config.get_task(task_name).context("Task not found")?;
    let file_paths = get_affected_files(repo, config)?;

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

    let separator = task.separator.as_deref().unwrap_or(" ");
    let files = &filtered_paths.join(separator);

    for command_template in &task.commands {
        let command_text = command_template.replace("{files}", files);
        debug!("Running command: {}", &command_text);

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&command_text)
            .current_dir(workspace_root)
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to start the command")?;

        if let Some(stdout) = child.stdout.take() {
            let reader = io::BufReader::new(stdout);
            for line in reader.lines() {
                let line = line?;
                println!("{}", line); // Print each line in real-time
            }
        }

        let status = child.wait().context("Failed to wait for the command")?;
        if !status.success() {
            bail!("Command failed: {}", &command_text);
        }
    }

    Ok(())
}
