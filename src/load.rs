use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use ignore::{types::TypesBuilder, WalkBuilder};

use crate::models::todo::Todo;

/// Load todo items from each markdown file under root.
pub fn load_todos_from_root(
    root: impl AsRef<Path>,
) -> anyhow::Result<HashMap<PathBuf, Vec<(usize, Todo)>>> {
    let root = root.as_ref();
    let mut map = HashMap::new();

    for path in walk_markdown_files(root) {
        let path = path?;
        let file = fs::read_to_string(&path)?;
        let todos = Todo::parse_file(&file);
        // let diff = diff_paths(&path, root).unwrap_or(path);
        map.insert(path, todos);
    }

    Ok(map)
}

fn walk_markdown_files(
    root: impl AsRef<Path>,
) -> impl Iterator<Item = Result<PathBuf, ignore::Error>> {
    let walk = WalkBuilder::new(root.as_ref())
        .types(
            TypesBuilder::new()
                .add_defaults()
                .select("md")
                .build()
                .unwrap(),
        )
        .add_custom_ignore_filename(".agmdignore")
        .build();

    walk.into_iter().filter_map(|entry| match entry {
        Ok(entry) => {
            let path = entry.into_path();
            if path.is_dir() {
                return None;
            }
            Some(Ok(path))
        }
        Err(err) => Some(Err(err)),
    })
}
