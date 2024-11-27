use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

// Resolves the module path relative to the root directory
fn resolve_module(root_dir: &Path, current_file: &Path, specifier: &str) -> Option<PathBuf> {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        // Handle relative paths
        let mut path = current_file.parent()?.join(specifier);
        if path.is_dir() {
            path.push("index.ts"); // Handle directory with index.ts
        }
        if path.exists() {
            return Some(path);
        }
        // Try with .ts extension
        path.set_extension("ts");
        if path.exists() {
            return Some(path);
        }
    } else {
        // Handle node_modules resolution
        let node_modules_path = root_dir.join("node_modules").join(specifier);
        if node_modules_path.exists() {
            return Some(node_modules_path);
        }

        // Fallback: Search node_modules with ignore crate
        return find_in_node_modules(root_dir, specifier);
    }
    None
}

// Searches within node_modules using the ignore crate
fn find_in_node_modules(root_dir: &Path, module_name: &str) -> Option<PathBuf> {
    let node_modules_dir = root_dir.join("node_modules");
    if !node_modules_dir.exists() {
        return None;
    }

    let walker = WalkBuilder::new(node_modules_dir)
        .standard_filters(true) // Respect .gitignore and .ignore
        .build();

    for result in walker {
        if let Ok(entry) = result {
            let path = entry.path();
            if path.ends_with(module_name) {
                return Some(path.to_path_buf());
            }
        }
    }
    None
}

pub fn demo() {
    // let root_dir = Path::new("/project");
    // let current_file = root_dir.join("src/example.ts");
    // std::env::current_dir()

    let root_dir = std::env::current_dir().unwrap();
    let current_file = Path::new("libs/studio-shared/sdk/src/lib/services/uuid.service.ts");

    let imports = vec![
        "@angular/core",     // Package import
        "./relative/module", // Relative import
        "../other/module",   // Another relative import
    ];

    for import in imports {
        match resolve_module(&root_dir, &current_file, import) {
            Some(resolved_path) => {
                println!("Resolved '{}' to '{}'", import, resolved_path.display())
            }
            None => println!("Could not resolve '{}'", import),
        }
    }
}
