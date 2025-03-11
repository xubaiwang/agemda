use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use ignore::{types::TypesBuilder, WalkBuilder};
use ouroboros::self_referencing;

use crate::models::todo::Todo;

/// Todo with position (line number).
type TodoWithPos = (usize, Todo);
/// Map from file path to todos in that file.
type TodoMap = HashMap<PathBuf, Vec<TodoWithPos>>;

type TodoGroup<'a> = Vec<(&'a Path, &'a TodoWithPos)>;

#[self_referencing]
struct TodoMapGroupedInner {
    map: TodoMap,
    #[borrows(map)]
    #[covariant]
    groups: Vec<(String, TodoGroup<'this>)>,
}

pub struct TodoMapGrouped(TodoMapGroupedInner);

impl TodoMapGrouped {
    pub fn new(map: TodoMap) -> Self {
        let inner = TodoMapGroupedInner::new(map, |map: &TodoMap| {
            let group_mal = collect_group(filters::mal, &map);
            let group_undue = collect_group(filters::undue, &map);
            let group_overdue = collect_group(filters::overdue, &map);
            let group_due_today = collect_group(filters::due_today, &map);
            let group_due_tomorrow = collect_group(filters::due_tomorrow, &map);
            let group_due_overmorrow = collect_group(filters::due_overmorrow, &map);
            let group_due_week = collect_group(filters::due_week, &map);
            let group_all = collect_group(filters::all, &map);
            vec![
                ("Mal".into(), group_mal),
                ("Undue".into(), group_undue),
                ("Overdue".into(), group_overdue),
                ("Today".into(), group_due_today),
                ("Tomorrow".into(), group_due_tomorrow),
                ("Overmorrow".into(), group_due_overmorrow),
                ("Week".into(), group_due_week),
                ("All".into(), group_all),
            ]
        });
        Self(inner)
    }

    pub fn map(&self) -> &TodoMap {
        self.0.borrow_map()
    }

    pub fn groups(&self) -> &Vec<(String, TodoGroup)> {
        self.0.borrow_groups()
    }
}

fn collect_group<'a, F>(f: F, map: &'a TodoMap) -> TodoGroup<'a>
where
    F: Fn(&'a TodoWithPos) -> bool,
{
    let mut group = vec![];
    for (path, todos) in map.iter() {
        for todo in todos.iter() {
            if f(todo) {
                group.push((path.as_path(), todo));
            }
        }
    }
    group
}

pub mod filters {
    use chrono::{Days, Local, NaiveDate};

    use super::TodoWithPos;

    fn today() -> NaiveDate {
        Local::now().date_naive()
    }

    pub fn mal(todo: &TodoWithPos) -> bool {
        todo.1.agmd.is_none()
    }

    pub fn undue(todo: &TodoWithPos) -> bool {
        todo.1.agmd.as_ref().is_some_and(|agmd| agmd.due.is_none())
    }

    pub fn overdue(todo: &TodoWithPos) -> bool {
        todo.1.agmd.as_ref().is_some_and(|agmd| {
            agmd.completed.is_none() && agmd.due.is_some_and(|due| due < today())
        })
    }

    pub fn due_today(todo: &TodoWithPos) -> bool {
        todo.1
            .agmd
            .as_ref()
            .is_some_and(|agmd| agmd.due.is_some_and(|due| due == today()))
    }

    pub fn due_tomorrow(todo: &TodoWithPos) -> bool {
        todo.1.agmd.as_ref().is_some_and(|agmd| {
            agmd.due
                .is_some_and(|due| due == today().checked_add_days(Days::new(1)).unwrap())
        })
    }

    pub fn due_overmorrow(todo: &TodoWithPos) -> bool {
        todo.1.agmd.as_ref().is_some_and(|agmd| {
            agmd.due
                .is_some_and(|due| due == today().checked_add_days(Days::new(2)).unwrap())
        })
    }

    pub fn due_week(todo: &TodoWithPos) -> bool {
        todo.1.agmd.as_ref().is_some_and(|agmd| {
            agmd.due.is_some_and(|due| {
                due >= today().checked_add_days(Days::new(3)).unwrap()
                    && due <= today().checked_add_days(Days::new(7)).unwrap()
            })
        })
    }

    pub fn all(_todo: &TodoWithPos) -> bool {
        true
    }
}

pub struct Loader {
    root: PathBuf,
}

impl Loader {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self { root }
    }

    pub fn load(&self) -> anyhow::Result<TodoMapGrouped> {
        let map = load_todos_from_root(&self.root)?;
        Ok(TodoMapGrouped::new(map))
    }
}

/// Load todo items from each markdown file under root.
pub fn load_todos_from_root(root: impl AsRef<Path>) -> anyhow::Result<TodoMap> {
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
