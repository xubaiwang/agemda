//! For caching data

use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::SystemTime,
};

use chrono::{Datelike, NaiveDate};
use fjall::{Config, TransactionalKeyspace, TransactionalPartition};
use uuid::Uuid;

use crate::Task;

pub static CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::cache_dir().unwrap().join("agemda").join("fjall"));

/// The cache manager.
pub struct Cache {
    /// The root of path to scan
    root: PathBuf,

    /// The overall database interface.
    keyspace: TransactionalKeyspace,

    // Task related partitions
    /// `Task` data, serialized with postcard.
    table_tasks: TransactionalPartition,
    /// Whether task agmd is malformed.
    table_tasks_index_mal_id: TransactionalPartition,
    /// `Task.source.path`
    table_tasks_index_path_id: TransactionalPartition,
    table_tasks_index_done_path: TransactionalPartition,
    table_tasks_index_done_path_start: TransactionalPartition,
    table_tasks_index_done_path_start_due: TransactionalPartition,
    table_tasks_index_done_path_start_due_id: TransactionalPartition,

    // Path related partitions, `(mtime, sha256)`.
    table_paths_metadata: TransactionalPartition,
}

impl Cache {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();

        let keyspace = Config::new(CACHE_DIR.as_path())
            .open_transactional()
            .unwrap();

        macro_rules! partitions {
            ($($name:ident),*) => {
                $(
                    let $name = keyspace
                        .open_partition(stringify!($name), Default::default())
                        .unwrap();
                )*
            };
        }

        // task related partitions
        partitions!(
            table_tasks,
            table_tasks_index_mal_id,
            table_tasks_index_path_id,
            table_tasks_index_done_path,
            table_tasks_index_done_path_start,
            table_tasks_index_done_path_start_due,
            table_tasks_index_done_path_start_due_id
        );

        // path related partitions
        partitions!(table_paths);

        Self {
            root,
            keyspace,
            table_tasks,
            table_tasks_index_mal_id,
            table_tasks_index_path_id,
            table_tasks_index_done_path,
            table_tasks_index_done_path_start,
            table_tasks_index_done_path_start_due,
            table_tasks_index_done_path_start_due_id,
            table_paths_metadata: table_paths,
        }
    }

    /// load from file and update cache store.
    fn load_from_root(&self) {
        todo!()
    }

    fn insert_task(&self, task: &Task) {
        let mut tx = self.keyspace.write_tx();

        // table
        let id = Uuid::new_v4();
        let serialized = postcard::to_allocvec(task).unwrap();
        tx.insert(&self.table_tasks, id.as_bytes(), serialized);

        // index task id
        let mut path_id: Vec<u8> = task.source.path.to_string_lossy().to_string().into_bytes();
        path_id.push(0);
        path_id.extend_from_slice(id.as_bytes());
        tx.insert(&self.table_tasks_index_path_id, path_id, []);
        match &task.agmd {
            // malform index
            None => {
                tx.insert(&self.table_tasks_index_mal_id, id.as_bytes(), []);
            }
            // other index
            Some(agmd) => {
                // done_path
                let done_path =
                    format!("{}{}", task.done as u8, task.source.path.to_string_lossy());
                let done_path_pid = match tx
                    .get(&self.table_tasks_index_done_path, &done_path)
                    .unwrap()
                {
                    Some(id) => id.to_vec(),
                    None => {
                        let id = Uuid::new_v4();
                        tx.insert(&self.table_tasks_index_done_path, done_path, id.as_bytes());
                        id.into()
                    }
                };
                // done_path_start
                let mut done_path_start = done_path_pid;
                match agmd.start {
                    Some(start) => {
                        done_path_start.extend_from_slice(
                            &start
                                .num_days_from_ce()
                                .wrapping_add(i32::MIN)
                                .to_be_bytes(),
                        );
                    }
                    None => {
                        // start mapped to -inf
                        done_path_start.push(0);
                    }
                }
                tx.insert(
                    &self.table_tasks_index_done_path_start,
                    &done_path_start,
                    [],
                );
                // done_path_start_due
                let mut done_path_start_due_id = done_path_start;
                match agmd.start {
                    Some(start) => {
                        done_path_start_due_id.extend_from_slice(
                            &start
                                .num_days_from_ce()
                                .wrapping_add(i32::MIN)
                                .to_be_bytes(),
                        );
                    }
                    None => {
                        // due None mapped to +inf
                        done_path_start_due_id.push(1);
                    }
                }
                done_path_start_due_id.extend_from_slice(id.as_bytes());
                tx.insert(
                    &self.table_tasks_index_done_path_start_due_id,
                    done_path_start_due_id,
                    [],
                );
            }
        }
        tx.commit().unwrap();
    }

    fn insert_path(
        &self,
        path_insert_record: &TransactionalPartition,
        path: &Path,
        mtime: SystemTime,
        sha: &str,
    ) {
        let v = postcard::to_allocvec(&(mtime, sha)).unwrap();
        let mut tx = self.keyspace.write_tx();
        tx.insert(&self.table_paths_metadata, path, v);
        tx.insert(path_insert_record, path, []);
        tx.commit().unwrap();
    }

    // TODO: remove old path
    fn remove_old_path(&self, path_insert_record: &TransactionalPartition) {
        let read_tx = self.keyspace.read_tx();
        for path in read_tx.keys(&self.table_paths_metadata) {
            let path = path.unwrap();
            if !read_tx.contains_key(path_insert_record, &path).unwrap() {
                let mut prefix = path.to_vec();
                prefix.push(0);

                let mut write_tx = self.keyspace.write_tx();
                write_tx.remove(&self.table_paths_metadata, path);

                for kv in read_tx.prefix(&self.table_tasks_index_path_id, &prefix) {
                    let (path_id, _) = kv.unwrap();
                    let id = path_id.split(|&u| u == b'\x00').nth(1).unwrap();
                    write_tx.remove(&self.table_tasks, id);
                    // TODO: remove index
                }

                write_tx.commit().unwrap();
            }
        }
    }

    // TODO: select_with_intersection
    pub fn select_task(
        done: Option<bool>,
        path: Option<&str>,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) {
    }
}

// pub fn find_tasks(done: bool) -> impl Iterator<Item = Task> {
//     TASKS_DONE.prefix([done as u8]).flatten().map(|(k, _)| {
//         let key = &k[1..];
//         let item = TASKS.get(key).unwrap().unwrap();
//         let task: Task = postcard::from_bytes(&item).unwrap();
//         task
//     })
// }
