mod config;
pub mod graph;
pub mod logger;
mod node;
pub mod nx;
mod projects;
pub mod tasks;
pub mod workspace;

use anyhow::Result;
use clap::ValueEnum;
pub use config::Config;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Text,
}

pub fn print_lines(lines: &HashSet<String>, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&lines)?;
            println!("{}", json_output);
        }
        _ => {
            for line in lines {
                println!("{}", line);
            }
        }
    }
    Ok(())
}

pub fn find_git_root(starting_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = starting_dir;

    while current_dir != current_dir.parent()? {
        if current_dir.join(".git").exists() {
            return Some(current_dir.to_path_buf());
        }
        current_dir = current_dir.parent()?;
    }

    None
}
