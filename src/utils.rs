use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn parse_workspace<F>(workspace_root: &PathBuf, filter_fn: F) -> Result<Vec<String>>
where
    F: Fn(&Path) -> bool,
{
    let walker = WalkBuilder::new(&workspace_root)
        .follow_links(true)
        .standard_filters(true) // Respect .gitignore, .ignore, etc.
        .build();

    let mut paths = vec![];

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path().to_path_buf();

                if let Ok(relative_path) = path.strip_prefix(&workspace_root) {
                    // check if we have not reached the root directory
                    if relative_path.to_string_lossy().is_empty() {
                        continue;
                    }
                    if filter_fn(&path) {
                        if !relative_path.to_string_lossy().is_empty() {
                            paths.push(relative_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(paths)
}
