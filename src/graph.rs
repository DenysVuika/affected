use crate::nx::NxProject;
use crate::projects::Project;
use anyhow::Result;
use petgraph::Graph;
use std::path::Path;

pub fn build_graph(workspace_root: &Path, project_paths: &[String]) -> Result<Graph<String, ()>> {
    let mut graph = Graph::new();
    let projects: Vec<NxProject> = project_paths
        .iter()
        .map(|path| NxProject::load(workspace_root, path))
        .collect::<Result<Vec<NxProject>>>()?;

    for project in projects {
        let project_name = project.name().unwrap_or("Unnamed");
        let project_node = graph.add_node(project_name.to_string());

        if let Some(dependencies) = project.implicit_dependencies {
            dependencies.iter().for_each(|dependency| {
                let dependency_node = graph.add_node(dependency.to_string());
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
