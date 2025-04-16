use std::{env::current_dir, path::PathBuf};

use argh::FromArgs;

#[derive(Clone, Debug, FromArgs)]
/// Run agmd on given root.
pub struct Cli {
    /// the root path to search for md files
    #[argh(positional, default = "default_root()")]
    pub root: PathBuf,
}

fn default_root() -> PathBuf {
    current_dir().expect("fail to get current dir")
}
