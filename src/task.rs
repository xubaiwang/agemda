//! Task data model and parsers.

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

mod fragment;
mod parse;

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub done: bool,
    pub text: String,
    pub raw: String,
    pub agmd: Option<Agmd>,
    pub source: Source,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Agmd {
    pub start: Option<NaiveDate>,
    pub completed: Option<NaiveDate>,
    pub due: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    pub path: PathBuf,
    pub line: usize,
}
