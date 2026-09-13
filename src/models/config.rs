#![allow(dead_code)] // old engine — removed in Phase 1 "Retire the old engine"
use crate::core::rules::OrganizationRule;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub rules: Vec<OrganizationRule>,
    pub scan_options: ScanOptions,
    pub behavior: BehaviorOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanOptions {
    pub recursive: bool,
    pub include_hidden: bool,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BehaviorOptions {
    pub copy_instead_of_move: bool,
    pub overwrite_existing: bool,
    pub create_directories: bool,
    pub dry_run: bool,
}

impl AppConfig {
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.input_path.is_none() {
            return Err("Input path is not set".to_string());
        }

        if self.output_path.is_none() {
            return Err("Output path is not set".to_string());
        }

        if self.rules.is_empty() {
            return Err("No organization rules configured".to_string());
        }

        Ok(())
    }
}
