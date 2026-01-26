use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub destination: Option<PathBuf>,
}

impl FileItem {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        
        Ok(Self {
            path,
            name,
            extension,
            size: metadata.len(),
            destination: None,
        })
    }
}