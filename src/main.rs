use std::path::PathBuf;

mod files;

fn main() {
    println!("Hello, world!");
    let path: PathBuf = PathBuf::from(r".");
    let files: Result<Vec<PathBuf>, std::io::Error> = files::list_files(&path);
    match files {
        Ok(paths) => paths.into_iter().fold((), |acc, path| match path.file_name() {
            Some(ref file) => println!("{}", file.to_str().unwrap()),
            None => println!("oe")
        }),
        Err(error) => println!("does not exist")
    }

}
