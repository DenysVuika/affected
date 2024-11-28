use crate::workspace::Workspace;
use crate::{print_lines, OutputFormat};
use anyhow::Result;

pub fn display_affected_files(workspace: &Workspace, format: &OutputFormat) -> Result<()> {
    let file_paths = workspace.affected_files()?;

    if file_paths.is_empty() {
        println!("No files affected");
        return Ok(());
    }

    print_lines(&file_paths, format, "Path")?;

    Ok(())
}

pub fn display_affected_projects(workspace: &Workspace, format: &OutputFormat) -> Result<()> {
    let projects = workspace.affected_projects()?;

    if projects.is_empty() {
        println!("No projects affected");
        return Ok(());
    }

    print_lines(&projects, format, "Project")?;

    Ok(())
}
