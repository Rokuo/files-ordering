use std::{path::{Path, PathBuf}, fs, io, ffi::{OsStr}};

pub fn list_files(path: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    let path: Result<fs::ReadDir, io::Error> = fs::read_dir(path);
    match path {
        Ok(dir) => dir.into_iter().for_each(
            |elem: Result<fs::DirEntry, std::io::Error>| files.push(elem.unwrap().path())
        ),
        Err(error) => return Err(error)
    }
    Ok(files)
}

pub fn move_file(from: &Path, to: &Path) -> Result<(), io::Error> {
    fs::rename(from, to)
}

pub fn sort_files_by_name_ascending(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    files
}

pub fn sort_files_by_name_descending(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    files
}

pub fn sort_files_by_ext(files: Vec<PathBuf>,  exts: Vec<&str>) -> Vec<PathBuf> {
    todo!("sort_files_by_ext")
}

pub fn filter_files_by_ext(mut files: Vec<PathBuf>, exts: Vec<&str>) -> Vec<PathBuf> {
    files.retain(|file| {
        match file.extension() {
            Some(ext) => exts.contains(&ext.to_str().unwrap()),
            None => false
        }
    });
    files
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use tempfile::TempDir;
    use super::*;

    #[test]
    fn test_list_files_on_existing_folder() {
        // arrange
        let dir: TempDir = tempfile::tempdir().unwrap();
        let files: Vec<&str> = vec!["file1.txt", "file2.txt", "file3.png", "file4.mp3", "file5"];
        std::fs::write(dir.path().join(files[0]), "hello").unwrap();
        std::fs::write(dir.path().join(files[1]), "world").unwrap();
        std::fs::write(dir.path().join(files[2]), b"test").unwrap();
        std::fs::write(dir.path().join(files[3]), b"test").unwrap();

        // act
        let files: Vec<PathBuf> = list_files(dir.path()).unwrap();

        // Assert
        assert_eq!(files.len(), 4);
        assert!(files.contains(&files[0]));
        assert!(files.contains(&files[1]));
    }

    #[test]
    fn test_list_files_on_empty_folder() {
        let dir: TempDir = tempfile::tempdir().unwrap();

        let files: Vec<PathBuf> = list_files(dir.path()).unwrap();

        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_list_files_on_non_existing_folder() {
        let non_existing_folder: PathBuf = PathBuf::new();
        let result: Result<Vec<PathBuf>, io::Error> = list_files(&non_existing_folder);

        assert!(result.is_err());
    }

    #[test]
    fn test_filter_files_by_ext() {
        let dir: TempDir = tempfile::tempdir().unwrap();
        let extensions = vec!["txt", "png"];
        let orig_files: Vec<&str> = vec!["sound.mp3", "aleks.txt", "file2.txt", "test.png", "bytes"];
        std::fs::write(dir.path().join(orig_files[0]), b"test").unwrap();
        std::fs::write(dir.path().join(orig_files[1]), "hello").unwrap();
        std::fs::write(dir.path().join(orig_files[2]), "world").unwrap();
        std::fs::write(dir.path().join(orig_files[3]), b"test").unwrap();

        let files: Vec<PathBuf> = filter_files_by_ext(list_files(dir.path()).unwrap(), extensions);

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_sort_files_by_ext() {

    }

    #[test]
    fn test_sort_files_by_alpha_ascending() {
        let dir: TempDir = tempfile::tempdir().unwrap();
        let files: Vec<&str> = vec!["sound.mp3", "aleks.txt", "file2.txt", "test.png", "bytes"];
        std::fs::write(dir.path().join(files[0]), b"test").unwrap();
        std::fs::write(dir.path().join(files[1]), "hello").unwrap();
        std::fs::write(dir.path().join(files[2]), "world").unwrap();
        std::fs::write(dir.path().join(files[3]), b"test").unwrap();

        let files: Vec<PathBuf> = sort_files_by_name_ascending(list_files(dir.path()).unwrap());

        assert_eq!(files.len(), 4);
        assert_eq!(files[0].file_name(), Some(OsStr::new("aleks.txt")));
        assert_eq!(files[3].file_name(), Some(OsStr::new("test.png")));
    }

    #[test]
    fn test_sort_files_by_alpha_descending() {
        let dir: TempDir = tempfile::tempdir().unwrap();
        let files: Vec<&str> = vec!["sound.mp3", "aleks.txt", "file2.txt", "test.png", "bytes"];
        std::fs::write(dir.path().join(files[0]), b"test").unwrap();
        std::fs::write(dir.path().join(files[1]), "hello").unwrap();
        std::fs::write(dir.path().join(files[2]), "world").unwrap();
        std::fs::write(dir.path().join(files[3]), b"test").unwrap();

        let files: Vec<PathBuf> = sort_files_by_name_descending(list_files(dir.path()).unwrap());

        assert_eq!(files.len(), 4);
        assert_eq!(files[0].file_name(), Some(OsStr::new("test.png")));
        assert_eq!(files[3].file_name(), Some(OsStr::new("aleks.txt")));
    }

    #[test]
    fn test_move_file_existing_file() {
        let dir: TempDir = tempfile::tempdir().unwrap();
        let file: &str = "file.txt";
        std::fs::write(dir.path().join(file), b"test").unwrap();

        let result: Result<(), io::Error> = move_file(&dir.path().join(file), &dir.path().join("new_file.txt"));

        assert!(result.is_ok());
        assert!(!dir.path().join(file).exists());
        assert!(dir.path().join("new_file.txt").exists());
    }

}
