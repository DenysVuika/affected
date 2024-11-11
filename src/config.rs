use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub base: Option<String>,
    pub tasks: Option<Vec<Task>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
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
