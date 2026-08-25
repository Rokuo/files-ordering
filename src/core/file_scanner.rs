use crate::models::file_item::FileItem;
use std::path::{Path};

pub struct FileScanner {
    recursive: bool,
    include_hidden: bool,
    max_depth: Option<usize>,
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            recursive: false,
            include_hidden: false,
            max_depth: None,
        }
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn include_hidden(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn scan(&self, path: &Path) -> std::io::Result<Vec<FileItem>> {
        let mut files = Vec::new();
        
        if self.recursive {
            self.scan_recursive(path, &mut files, 0)?;
        } else {
            self.scan_directory(path, &mut files)?;
        }

        Ok(files)
    }

    fn scan_directory(&self, path: &Path, files: &mut Vec<FileItem>) -> std::io::Result<()> {
        if !path.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                if !self.include_hidden && self.is_hidden(&entry_path) {
                    continue;
                }

                if let Ok(file_item) = FileItem::new(entry_path) {
                    files.push(file_item);
                }
            }
        }

        Ok(())
    }

    fn scan_recursive(
        &self,
        path: &Path,
        files: &mut Vec<FileItem>,
        current_depth: usize,
    ) -> std::io::Result<()> {
        if let Some(max) = self.max_depth {
            if current_depth >= max {
                return Ok(());
            }
        }

        if !path.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if !self.include_hidden && self.is_hidden(&entry_path) {
                continue;
            }

            if entry_path.is_file() {
                if let Ok(file_item) = FileItem::new(entry_path) {
                    files.push(file_item);
                }
            } else if entry_path.is_dir() {
                self.scan_recursive(&entry_path, files, current_depth + 1)?;
            }
        }

        Ok(())
    }

    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScanResult {
    pub files: Vec<FileItem>,
    pub total_size: u64,
    pub file_count: usize,
    pub errors: Vec<String>,
}

impl ScanResult {
    pub fn from_files(files: Vec<FileItem>) -> Self {
        let total_size = files.iter().map(|f| f.size).sum();
        let file_count = files.len();

        Self {
            files,
            total_size,
            file_count,
            errors: Vec::new(),
        }
    }

    pub fn format_size(&self) -> String {
        format_bytes(self.total_size)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}