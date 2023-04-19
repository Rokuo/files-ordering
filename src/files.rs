use std::{path::{Path, PathBuf}, fs, io};

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

pub fn order_files_by_type(files: Vec<PathBuf>) -> Vec<PathBuf> {
    todo!()
}

pub fn order_files_by_ext(files: Vec<PathBuf>) -> Vec<PathBuf> {
    todo!()
}

#[cfg(test)]
mod tests {
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

    }

    #[test]
    fn test_order_files_by_ext() {

    }

    #[test]
    fn test_order_files_by_alpha() {

    }

}
