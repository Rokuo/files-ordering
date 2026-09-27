#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rename {
    pub pattern: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Destination {
    pub folder: PathBuf,
    pub sub_folder_pattern: String,
    pub rename: Option<Rename>,
}
