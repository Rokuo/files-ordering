#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    pub folder: String,
    pub sub_folder_pattern: String,
    pub rename_file: bool,
    pub filename_pattern: String,
}