use crate::nx::NxProject;
use crate::projects::Project;
use crate::workspace::Workspace;
use anyhow::Result;
use petgraph::Graph;

pub type WorkspaceGraph = Graph<NodeType, ()>;

#[derive(Debug, Clone)]
pub enum NodeType {
    Project(ProjectNode),
    File(FileNode),
}

#[derive(Debug, Default, Clone)]
pub struct ProjectNode {
    pub name: String,
    pub path: Option<String>,
    pub implicit_dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
}

pub fn build_graph(workspace: &Workspace) -> Result<WorkspaceGraph> {
    let mut graph = Graph::new();

    let affected_projects = workspace.affected_projects()?;
    if affected_projects.is_empty() {
        return Ok(graph);
    }

    let projects: Vec<NxProject> = affected_projects
        .iter()
        .map(|path| NxProject::load(&workspace.root, path))
        .collect::<Result<Vec<NxProject>>>()?;

    for project in projects {
        let project_name = project.name().unwrap_or("Unnamed");
        let project_node = graph.add_node(NodeType::Project(ProjectNode {
            name: project_name.to_string(),
            path: project.source_root,
            implicit_dependencies: project.implicit_dependencies.clone(),
        }));

        // todo: support globset for implicit dependencies
        if let Some(dependencies) = project.implicit_dependencies {
            dependencies.iter().for_each(|dependency| {
                let dependency_node = graph.add_node(NodeType::Project(ProjectNode {
                    name: dependency.to_string(),
                    ..Default::default()
                }));

                graph.add_edge(project_node, dependency_node, ());
            });
        }

        // let project_dependencies = project.dependencies();
        // for dependency in project_dependencies {
        //     let dependency_name = dependency.name().unwrap_or("Unnamed");
        //     let dependency_node = graph.add_node(dependency_name.to_string());
        //
        //     graph.add_edge(project_node, dependency_node, 1);
        // }
    }

    Ok(graph)
}

// fn get_all_projects(workspace_root: &PathBuf) -> Result<Vec<String>> {
//     let filter_fn = |path: &Path| path.is_dir() && path.join("project.json").is_file();
//     let projects = inspect_workspace(workspace_root, filter_fn)?;
//
//     Ok(projects)
// }
