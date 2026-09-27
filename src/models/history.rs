#![allow(dead_code)]
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct History {
    pub last_run: Option<NaiveDate>,
    pub last_matched: Option<u64>,
}
