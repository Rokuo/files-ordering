pub mod files;
mod app;
mod core;
mod models;
mod ui;

// fn main() {
//     let path: PathBuf = PathBuf::from(r".");
//     let mut files: Result<Vec<PathBuf>, std::io::Error> = files::list_files(&path);
//     match files {
//         Ok(paths) => files::filter_files_by_ext(paths, vec!["md", "lock", "gitignore"]).into_iter().fold((), |acc, path| match path.file_name() {
//             Some(ref file) => println!("{}", file.to_str().unwrap()),
//             None => println!("oe")
//         }),
//         Err(error) => println!("does not exist")
//     }
// }
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("File Organizer"),
        ..Default::default()
    };
    
    eframe::run_native(
        "File Organizer",
        options,
        Box::new(|_cc| Ok(Box::new(app::FileOrganizerApp::default()))),
    )
}
