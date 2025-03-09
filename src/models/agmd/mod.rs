use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub mod parse;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Agmd {
    pub start: Option<NaiveDate>,
    pub due: Option<NaiveDate>,
    pub completed: Option<NaiveDate>,
}
