use std::{
    env,
    path::{absolute, PathBuf},
    sync::LazyLock,
};

pub use crate::{cache::*, task::*};

mod cache;
mod task;

/// Either first arg or current dir.
pub static ROOT_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| absolute(env::args().nth(1).unwrap_or(".".to_string())).unwrap());
