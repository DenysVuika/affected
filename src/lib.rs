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

use anyhow::Result;
use clap::ValueEnum;
pub use config::Config;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tabled::builder::Builder;
use tabled::settings::Style;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Text,
}

pub fn print_lines(lines: &HashSet<String>, format: &OutputFormat, header: &str) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&lines)?;
            println!("{}", json_output);
        }
        OutputFormat::Table => {
            let mut builder = Builder::default();
            builder.push_record([header]);

            for line in lines {
                builder.push_record([line]);
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
