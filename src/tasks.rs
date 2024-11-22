use crate::config::Task;
use crate::workspace::Workspace;
use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSetBuilder};
use log::debug;
use std::process::Stdio;
use tokio::process::Command;

pub async fn run_tasks(workspace: &Workspace, pattern: &str) -> Result<()> {
    let config = workspace.config().context("No configuration found")?;
    let tasks = config.get_tasks(pattern);

    if tasks.is_empty() {
        println!("No tasks matched the pattern");
        return Ok(());
    }

    for task in tasks {
        run_task(workspace, task).await?;
    }

    Ok(())
}

async fn run_task(workspace: &Workspace, task: &Task) -> Result<()> {
    let file_paths = workspace.affected_files()?;
    let projects: Vec<String> = workspace.affected_projects()?.into_iter().collect();

    // filter out files that exist on the filesystem
    let file_paths: Vec<_> = file_paths
        .into_iter()
        .filter(|path| workspace.root.join(path).exists())
        .collect();

    if file_paths.is_empty() {
        debug!("No files affected");
        return Ok(());
    }

    let mut builder = GlobSetBuilder::new();

    if let Some(patterns) = &task.patterns {
        for pattern in patterns {
            builder.add(Glob::new(pattern)?);
        }
    } else {
        builder.add(Glob::new("**/*")?);
    }

    let patterns = builder.build()?;

    let filtered_paths: Vec<_> = file_paths
        .into_iter()
        .filter(|path| patterns.is_match(path))
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
    let projects = &projects.join(separator);

    let mut handles = Vec::new();

    for command_template in &task.commands {
        let template = command_template.clone();
        let command_text = template
            .replace("{files}", files)
            .replace("{projects}", projects);
        debug!("Running command: {}", &command_text);

        let workspace_root = workspace.root.to_path_buf();
        let handle = tokio::spawn(async move {
            let mut child = Command::new("sh")
                .arg("-c")
                .arg(&command_text)
                .current_dir(&workspace_root)
                .stdout(Stdio::piped())
                .env("FORCE_COLOR", "1")
                .spawn()
                .context("Failed to start the command")?;

            if let Some(mut stdout) = child.stdout.take() {
                tokio::io::copy(&mut stdout, &mut tokio::io::stdout()).await?;
            }

            let status = child
                .wait()
                .await
                .context("Failed to wait for the command")?;

            if !status.success() {
                bail!("Command failed: {}", &template);
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
