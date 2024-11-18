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
