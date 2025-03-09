//! Task data model and parsers.

use serde::{Deserialize, Serialize};

use super::agmd::Agmd;

mod parse;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Todo {
    pub summary: String,
    pub raw: String,
    pub agmd: Option<Agmd>,
}
