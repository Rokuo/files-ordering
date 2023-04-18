use std::{path::{Path, PathBuf}, fs};

fn list_files(path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    let path: fs::ReadDir = fs::read_dir(path).unwrap();
    path.into_iter().for_each(|elem: Result<fs::DirEntry, std::io::Error>| files.push(elem.unwrap().path()));
    files
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    struct Directory {
        path: TempDir,
        files: Vec<PathBuf>,
    }

    //fn setup() -> Directory {
        // Create a temporary directory with some files

        //Directory { path: &dir, files: files.into_iter().map(|elem| dir.path().join(elem)).collect() }
    //}

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
        let files: Vec<PathBuf> = list_files(dir.path());

        // Assert
        assert_eq!(files.len(), 4);
        assert!(files.contains(&files[0]));
        assert!(files.contains(&files[1]));
    }

    #[test]
    fn test_list_files_on_empty_folder() {
        let dir: TempDir = tempfile::tempdir().unwrap();

        let files: Vec<PathBuf> = list_files(dir.path());

        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_list_files_on_non_existing_folder() {
        let non_existing_folder: PathBuf = PathBuf::new();

        let files: Vec<PathBuf> = list_files(&non_existing_folder.as_path());

        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_filter_files() {

    }

    #[test]
    fn test_order_files() {

    }

}
