use anyhow::Result;
use std::path::Path;

/// A trait for defining a project.
pub trait Project {
    fn name(&self) -> Option<&str>;
    fn load(workspace_root: &Path, project_path: &str) -> Result<Self>
    where
        Self: Sized;
}

// pub fn get_project(workspace: &Workspace, project_path: &str) -> Result<Box<dyn Project>> {
//     let project_root = workspace.root.join(project_path);
//     let project_json_path = project_root.join("project.json");
//     let package_json_path = project_root.join("package.json");
//
//     if project_json_path.is_file() {
//         let nx_proj = nx::NxProject::load(workspace, project_path)?;
//         debug!("{:?}", nx_proj);
//         Ok(Box::new(nx_proj))
//     } else if package_json_path.is_file() {
//         let node_proj = node::NodeProject::load(workspace, project_path)?;
//         debug!("{:?}", node_proj);
//         Ok(Box::new(node_proj))
//     } else {
//         bail!("Could not find 'project.json' or 'package.json' in the project directory");
//     }
// }
