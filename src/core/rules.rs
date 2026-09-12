use crate::models::file_item::FileItem;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationRule {
    ByExtension,
    ByDate { format: DateFormat },
    BySize { ranges: Vec<SizeRange> },
    ByName { pattern: NamePattern },
    Custom { name: String, logic: CustomLogic },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DateFormat {
    Year,           // 2024/
    YearMonth,      // 2024/01/
    YearMonthDay,   // 2024/01/15/
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeRange {
    pub name: String,
    pub min_bytes: u64,
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamePattern {
    StartsWith(String),
    Contains(String),
    EndsWith(String),
    Regex(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomLogic {
    // Placeholder for future custom logic
    None,
}

pub struct RuleEngine {
    rules: Vec<OrganizationRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_rules(rules: Vec<OrganizationRule>) -> Self {
        Self { rules }
    }

    pub fn add_rule(&mut self, rule: OrganizationRule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, index: usize) {
        if index < self.rules.len() {
            self.rules.remove(index);
        }
    }

    pub fn get_rules(&self) -> &[OrganizationRule] {
        &self.rules
    }

    pub fn apply_rules(&self, file: &FileItem, base_output: &PathBuf) -> PathBuf {
        let mut destination = base_output.clone();

        for rule in &self.rules {
            match rule {
                OrganizationRule::ByExtension => {
                    if let Some(ref ext) = file.extension {
                        destination.push(ext);
                    } else {
                        destination.push("no_extension");
                    }
                }
                OrganizationRule::ByDate { format } => {
                    if let Ok(metadata) = std::fs::metadata(&file.path) {
                        if let Ok(modified) = metadata.modified() {
                            let datetime: chrono::DateTime<chrono::Local> = modified.into();
                            
                            match format {
                                DateFormat::Year => {
                                    destination.push(datetime.format("%Y").to_string());
                                }
                                DateFormat::YearMonth => {
                                    destination.push(datetime.format("%Y/%m").to_string());
                                }
                                DateFormat::YearMonthDay => {
                                    destination.push(datetime.format("%Y/%m/%d").to_string());
                                }
                            }
                        }
                    }
                }
                OrganizationRule::BySize { ranges } => {
                    let size_folder = self.get_size_folder(file.size, ranges);
                    destination.push(size_folder);
                }
                OrganizationRule::ByName { pattern } => {
                    if let Some(folder) = self.match_name_pattern(&file.name, pattern) {
                        destination.push(folder);
                    }
                }
                OrganizationRule::Custom { name, .. } => {
                    destination.push(name);
                }
            }
        }

        destination.push(&file.name);
        destination
    }

    fn get_size_folder(&self, size: u64, ranges: &[SizeRange]) -> String {
        for range in ranges {
            if size >= range.min_bytes {
                if let Some(max) = range.max_bytes {
                    if size <= max {
                        return range.name.clone();
                    }
                } else {
                    return range.name.clone();
                }
            }
        }
        "other".to_string()
    }

    fn match_name_pattern(&self, name: &str, pattern: &NamePattern) -> Option<String> {
        match pattern {
            NamePattern::StartsWith(prefix) => {
                if name.starts_with(prefix) {
                    Some(format!("starts_with_{}", prefix))
                } else {
                    None
                }
            }
            NamePattern::Contains(substring) => {
                if name.contains(substring) {
                    Some(format!("contains_{}", substring))
                } else {
                    None
                }
            }
            NamePattern::EndsWith(suffix) => {
                if name.ends_with(suffix) {
                    Some(format!("ends_with_{}", suffix))
                } else {
                    None
                }
            }
            NamePattern::Regex(_pattern) => {
                // You could implement regex matching here with the `regex` crate
                None
            }
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Predefined size ranges for convenience
pub fn default_size_ranges() -> Vec<SizeRange> {
    vec![
        SizeRange {
            name: "tiny (< 1MB)".to_string(),
            min_bytes: 0,
            max_bytes: Some(1_024 * 1_024),
        },
        SizeRange {
            name: "small (1MB - 10MB)".to_string(),
            min_bytes: 1_024 * 1_024,
            max_bytes: Some(10 * 1_024 * 1_024),
        },
        SizeRange {
            name: "medium (10MB - 100MB)".to_string(),
            min_bytes: 10 * 1_024 * 1_024,
            max_bytes: Some(100 * 1_024 * 1_024),
        },
        SizeRange {
            name: "large (> 100MB)".to_string(),
            min_bytes: 100 * 1_024 * 1_024,
            max_bytes: None,
        },
    ]
}