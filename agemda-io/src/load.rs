use std::path::{Path, PathBuf};

use agemda_core::Todo;
use ignore::{WalkBuilder, types::TypesBuilder};

use crate::parse::parse_file;

pub fn walk_markdown_files(
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

pub fn load_todos_from_root(root: impl AsRef<Path>) -> anyhow::Result<Vec<Todo>> {
    let root = root.as_ref();
    let mut todos = vec![];

    for path in walk_markdown_files(root) {
        let path = path?;
        parse_file(&mut todos, path)?;
    }

    Ok(todos)
}
