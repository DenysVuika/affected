use anyhow::Result;
use globset::Glob;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub base: Option<String>,
    pub tasks: Option<Vec<Task>>,
}

impl Config {
    pub fn get_task(&self, task_name: &str) -> Option<&Task> {
        self.tasks
            .as_ref()
            .and_then(|tasks| tasks.iter().find(|task| task.name == task_name))
    }

    pub fn get_tasks(&self, pattern: &str) -> Vec<&Task> {
        let glob = Glob::new(pattern).unwrap().compile_matcher();
        self.tasks
            .as_ref()
            .map(|tasks| {
                tasks
                    .iter()
                    .filter(|task| glob.is_match(&task.name))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            base: Some("main".to_string()),
            tasks: Some(vec![
                Task {
                    name: "lint".to_string(),
                    description: Some("Runs eslint for all affected files (example)".to_string()),
                    patterns: Some(vec!["*.{ts,tsx,js,jsx}".to_string()]),
                    commands: vec!["npx eslint {files}".to_string()],
                    ..Default::default()
                },
                Task {
                    name: "prettier".to_string(),
                    description: Some("Runs prettier for all affected files (example)".to_string()),
                    patterns: Some(vec!["*.{ts,tsx,js,jsx}".to_string()]),
                    commands: vec!["npx prettier --check {files}".to_string()],
                    ..Default::default()
                },
            ]),
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub patterns: Option<Vec<String>>,
    pub separator: Option<String>,
    pub commands: Vec<String>,
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{} ({})", self.name, description)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl Config {
    pub fn to_file(&self, output_path: &PathBuf) -> Result<()> {
        let yaml_data = serde_yaml::to_string(&self)?;
        let mut file = File::create(output_path)?;
        file.write_all(yaml_data.as_bytes())?;

        Ok(())
    }

    pub fn from_file(file_path: &PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let dictionary: Self = serde_yaml::from_reader(reader)?;

        Ok(dictionary)
    }
}
