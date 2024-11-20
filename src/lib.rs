mod config;
pub mod graph;
pub mod logger;
mod node;
pub mod nx;
mod projects;
pub mod tasks;
pub mod workspace;

use anyhow::Result;
pub use config::Config;
use std::collections::HashSet;

pub fn print_lines(lines: &HashSet<String>, format: &str) -> Result<()> {
    match format {
        "json" => {
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
