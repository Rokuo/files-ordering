#![allow(dead_code)] // old engine — removed in Phase 1 "Retire the old engine"
use chrono::NaiveDate;
use std::path::PathBuf;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        Ok(Self {
            path,
            name,
            extension,
            size: metadata.len(),
            destination: None,
            created_at: metadata.created().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.date_naive()
            }),
            modified_at: metadata.modified().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.date_naive()
            }),
        })
    }

    pub fn file_name(&self) -> String {
        match &self.extension {
            Some(ext) => format!("{}.{}", self.name, ext),
            None => self.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_name_with_extension() {
        let file_item = FileItem {
            path: PathBuf::from("test.txt"),
            name: "test".into(),
            extension: Some("txt".into()),
            size: 0,
            destination: None,
            created_at: None,
            modified_at: None,
        };
        assert_eq!(file_item.file_name(), "test.txt");
    }

    #[test]
    fn test_file_name_without_extension() {
        let file_item = FileItem {
            path: PathBuf::from("test"),
            name: "test".into(),
            extension: None,
            size: 0,
            destination: None,
            created_at: None,
            modified_at: None,
        };
        assert_eq!(file_item.file_name(), "test");
    }
    /// Extension case is preserved, so a rebuilt name matches the file on
    /// disk byte for byte — which is what lets `file_name()` be used to
    /// compare against a real path on a case-sensitive filesystem.
    #[test]
    fn test_file_name_preserves_the_extension_case() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("IMG_4471.HEIC");
        std::fs::write(&path, b"not really a photo").unwrap();

        let file_item = FileItem::new(path.clone()).unwrap();

        assert_eq!(file_item.extension, Some("HEIC".to_string()));
        assert_eq!(file_item.file_name(), "IMG_4471.HEIC");
        assert_eq!(file_item.path, path);
    }

    #[test]
    fn test_file_name_handles_a_dotfile() {
        let file_item = FileItem {
            path: PathBuf::from(".gitignore"),
            name: ".gitignore".into(),
            extension: None,
            size: 0,
            destination: None,
            created_at: None,
            modified_at: None,
        };

        assert_eq!(file_item.file_name(), ".gitignore");
    }
}
