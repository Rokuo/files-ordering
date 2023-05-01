use std::path::PathBuf;

mod files;

fn main() {
    let path: PathBuf = PathBuf::from(r".");
    let mut files: Result<Vec<PathBuf>, std::io::Error> = files::list_files(&path);
    match files {
        Ok(paths) => files::filter_files_by_ext(paths, vec!["md", "lock", "gitignore"]).into_iter().fold((), |acc, path| match path.file_name() {
            Some(ref file) => println!("{}", file.to_str().unwrap()),
            None => println!("oe")
        }),
        Err(error) => println!("does not exist")
    }
}
