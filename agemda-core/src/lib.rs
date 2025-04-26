use std::path::PathBuf;

use chrono::{DateTime, Local};

/// The attributes specified in `<agmd:>` link.
pub struct Attributes {
    pub start: Option<DateTime<Local>>,
    pub due: Option<DateTime<Local>>,
    pub completed: Option<DateTime<Local>>,
}

pub struct Metadata {
    pub path: PathBuf,
}

/// A todo task corresponding to ical VTODO.
pub struct Todo {
    pub summary: String,
    /// When parse error, return the raw string.
    pub attributes: Result<Attributes, String>,
    pub metadata: Metadata,
}
