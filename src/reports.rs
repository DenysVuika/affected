use crate::workspace::Workspace;
use crate::OutputFormat;
use anyhow::Result;
use std::collections::HashSet;
use tabled::builder::Builder;
use tabled::settings::Style;

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

pub fn display_tasks(workspace: &Workspace, format: &OutputFormat) -> Result<()> {
    let tasks = workspace.tasks();

    if tasks.is_empty() {
        println!("No tasks defined");
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&tasks)?;
            println!("{}", json_output);
        }
        OutputFormat::Table => {
            let mut builder = Builder::default();
            builder.push_record(["#", "Name", "Description", "Patterns"]);

            for (index, task) in tasks.iter().enumerate().map(|(i, task)| (i + 1, task)) {
                builder.push_record([
                    &index.to_string(),
                    &task.name.clone(),
                    &task.description.clone().unwrap_or_default(),
                    &task
                        .patterns
                        .as_ref()
                        .map_or(String::new(), |p| p.join(", ")),
                ]);
            }

            let mut table = builder.build();
            table.with(Style::modern());

            println!("{}", table);
        }
        _ => {
            for task in tasks {
                println!("{}", task.name);
            }
        }
    }

    Ok(())
}

pub fn print_lines(lines: &HashSet<String>, format: &OutputFormat, header: &str) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&lines)?;
            println!("{}", json_output);
        }
        OutputFormat::Table => {
            let mut builder = Builder::default();
            builder.push_record(["#", header]);

            for (index, line) in lines.iter().enumerate().map(|(i, line)| (i + 1, line)) {
                builder.push_record([&index.to_string(), line]);
            }

            let mut table = builder.build();
            table.with(Style::modern());

            println!("{}", table);
        }
        _ => {
            for line in lines {
                println!("{}", line);
            }
        }
    }
    Ok(())
}
