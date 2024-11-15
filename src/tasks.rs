use crate::{get_affected_files, Config};
use anyhow::{bail, Context, Result};
use git2::Repository;
use glob::Pattern;
use log::debug;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn run_task_by_name(
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

    let filtered_paths: Vec<_> = if let Some(patterns) = &task.patterns {
        file_paths
            .into_iter()
            .filter(|path| {
                patterns.iter().any(|pattern| {
                    Pattern::new(pattern)
                        .map(|p| p.matches(path))
                        .unwrap_or(false)
                })
            })
            .collect()
    } else {
        file_paths
    };

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

    let mut handles = Vec::new();

    for command_template in &task.commands {
        let command_text = command_template.replace("{files}", files);
        debug!("Running command: {}", &command_text);

        let workspace_root = workspace_root.to_path_buf();
        let handle = tokio::spawn(async move {
            let mut child = Command::new("sh")
                .arg("-c")
                .arg(&command_text)
                .current_dir(&workspace_root)
                .stdout(Stdio::piped())
                .spawn()
                .context("Failed to start the command")?;

            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Some(line) = reader.next_line().await? {
                    println!("{}", line);
                }
            }

            let status = child
                .wait()
                .await
                .context("Failed to wait for the command")?;
            if !status.success() {
                bail!("Command failed: {}", &command_text);
            }

            Ok::<(), anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Await all tasks and fail fast if any command fails
    for handle in handles {
        handle.await.context("Task panicked")??;
    }

    Ok(())
}
