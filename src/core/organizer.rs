use crate::core::file_scanner::FileScanner;
use crate::core::rules::RuleEngine;
use crate::models::file_item::FileItem;
use std::path::PathBuf;

pub struct Organizer {
    scanner: FileScanner,
    rule_engine: RuleEngine,
}

impl Organizer {
    pub fn new() -> Self {
        Self {
            scanner: FileScanner::new(),
            rule_engine: RuleEngine::new(),
        }
    }

    pub fn with_rule_engine(mut self, rule_engine: RuleEngine) -> Self {
        self.rule_engine = rule_engine;
        self
    }

    pub fn set_scanner_options(&mut self, recursive: bool, include_hidden: bool, max_depth: Option<usize>) {
        self.scanner = FileScanner::new()
            .recursive(recursive)
            .include_hidden(include_hidden)
            .max_depth(max_depth);
    }

    // Make these methods PUBLIC
    pub fn get_rules(&self) -> &[crate::core::rules::OrganizationRule] {
        self.rule_engine.get_rules()
    }

    pub fn add_rule(&mut self, rule: crate::core::rules::OrganizationRule) {
        self.rule_engine.add_rule(rule);
    }

    pub fn remove_rule(&mut self, index: usize) {
        self.rule_engine.remove_rule(index);
    }

    pub fn scan_directory(&self, path: &PathBuf) -> std::io::Result<Vec<FileItem>> {
        self.scanner.scan(path)
    }

    pub fn apply_rules(&self, files: &mut [FileItem], output_base: &PathBuf) {
        for file in files.iter_mut() {
            let destination = self.rule_engine.apply_rules(file, output_base);
            file.destination = Some(destination);
        }
    }

    pub fn organize(&self, files: &[FileItem]) -> Result<(), Box<dyn std::error::Error>> {
        for file in files {
            if let Some(ref destination) = file.destination {
                // Create parent directories if they don't exist
                if let Some(parent) = destination.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Copy or move the file
                std::fs::copy(&file.path, destination)?;
                
                // If moving (not copying), remove the original
                // std::fs::remove_file(&file.path)?;
            }
        }
        
        Ok(())
    }
}

impl Default for Organizer {
    fn default() -> Self {
        Self::new()
    }
}