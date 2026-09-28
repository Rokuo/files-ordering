#![allow(dead_code)] // old engine — removed in Phase 1 "Retire the old engine"
use std::path::PathBuf;
use chrono::NaiveDate;
#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub destination: Option<PathBuf>,
    pub created_at: Option<NaiveDate>,
    pub modified_at: Option<NaiveDate>,
}

impl FileItem {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        Ok(Self {
            path,
            name,
            extension,
            size: metadata.len(),
            destination: None,
            created_at: metadata.created().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.naive_utc().date()
            }),
            modified_at: metadata.modified().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.naive_utc().date()
            }),
        })
    }
}
