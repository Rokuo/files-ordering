// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::path::PathBuf;
mod files;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted with normality from Rust!", name)
}

#[tauri::command]
fn list_files(name: &str) -> Vec<String> {
    let path: PathBuf = PathBuf::from(r".");
    let files: Result<Vec<PathBuf>, std::io::Error> = files::list_files(&path);
    match files {
        Ok(paths) => {
            paths.iter().filter_map(|path| path.file_name()).filter_map(|name| name.to_str()).map(|name| format!("There is {}!", name)).collect::<Vec<_>>()
            // let filtered = files::filter_files_by_ext(paths, vec![]);
            // filtered.iter()
            //     .filter_map(|path| path.file_name())
            //     .filter_map(|name| name.to_str())
            //     .map(|name| format!("There is {}!", name))
            //     .collect::<Vec<_>>()
        },
        Err(_error) => Vec::<String>::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, list_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
