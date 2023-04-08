#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_file_does_not_exists() {
        let file: &Path = Path::new("test.txt");
        assert_eq!(file.exists(), false);
    }

    #[test]
    fn test_file_exists() {
        let file: &Path = Path::new("Cargo.toml");
        assert_eq!(file.exists(), true);
    }

    #[test]
    fn test_file_read() {
        let file: &Path = Path::new("Cargo.toml");
        let mut f = File::open(file).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        assert_eq!(s.len(), 0);
    }
}