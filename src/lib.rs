mod config;
pub mod graph;
pub mod logger;
mod node;
pub mod nx;
mod projects;
pub mod reports;
pub mod tasks;
pub mod ts;
pub mod workspace;

use clap::ValueEnum;
pub use config::Config;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Text,
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
