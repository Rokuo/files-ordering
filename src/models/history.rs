#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub last_run: Option<String>,
    pub last_matched: Option<u64>,
}