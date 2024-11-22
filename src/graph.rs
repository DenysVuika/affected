use crate::workspace::WorkspaceGraph;
use log::debug;
use std::collections::HashSet;

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

pub fn check_graph_recursively(
    graph: &WorkspaceGraph,
    initial_affected_projects: &HashSet<String>,
) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut affected_projects = HashSet::new();

    for node_index in graph.node_indices() {
        let node = &graph[node_index];
        if let NodeType::Project(project_node) = node {
            if initial_affected_projects.contains(&project_node.name) {
                debug!("Starting DFS for affected project: {}", project_node.name);
                dfs_visit(graph, node_index, &mut visited, &mut affected_projects);
            }
        }
    }

    affected_projects
}

fn dfs_visit(
    graph: &WorkspaceGraph,
    node_index: petgraph::graph::NodeIndex,
    visited: &mut HashSet<petgraph::graph::NodeIndex>,
    affected_projects: &mut HashSet<String>, // Collect found project names here
) {
    if visited.contains(&node_index) {
        return;
    }

    visited.insert(node_index);

    let node = &graph[node_index];
    if let NodeType::Project(project_node) = node {
        debug!("Visiting project: {}", project_node.name);

        // Add the project name to affected_projects
        affected_projects.insert(project_node.name.clone());

        let neighbors: Vec<_> = graph.neighbors(node_index).collect();
        debug!(
            "Neighbors (Outgoing) of {}: {:?}",
            project_node.name, neighbors
        );

        for neighbor in graph.neighbors(node_index) {
            let neighbor_node = &graph[neighbor];
            if let NodeType::Project(neighbor_project) = neighbor_node {
                debug!(
                    "{} <-> (implicit) <-> {}",
                    project_node.name, neighbor_project.name
                );
                dfs_visit(graph, neighbor, visited, affected_projects);
            }
        }

        let incoming_neighbors: Vec<_> = graph
            .neighbors_directed(node_index, petgraph::Direction::Incoming)
            .collect();
        debug!(
            "Neighbors (Incoming) of {}: {:?}",
            project_node.name, incoming_neighbors
        );

        for neighbor in incoming_neighbors {
            let neighbor_node = &graph[neighbor];
            if let NodeType::Project(neighbor_project) = neighbor_node {
                debug!(
                    "{} <-> (implicit) <-> {}",
                    neighbor_project.name, project_node.name
                );
                dfs_visit(graph, neighbor, visited, affected_projects);
            }
        }
    }
}
