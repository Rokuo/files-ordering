use crate::app::FileOrganizerApp;
use crate::files::{list_files};

use std::path::Path;

pub fn render(ui: &mut egui::Ui, app: &mut FileOrganizerApp) {
    ui.heading("Select Input Folder");
    
    ui.horizontal(|ui| {
        if let Some(ref path) = app.selected_input_path {
            ui.label(format!("Selected: {}", path));
        } else {
            ui.label("No folder selected");
        }
        
        if ui.button("Browse...").clicked() {
            // File dialog integration here
            // You might want to add `rfd` crate for native file dialogs
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.selected_input_path = Some(path.display().to_string());
            }
        }
    });
    
    if app.selected_input_path.is_some() {
        if ui.button("Next: Configure Rules").clicked() {
            // Scan files and move to next view
            list_files(Path::new(app.selected_input_path.as_ref().unwrap()))
                .map(|files| app.files_to_organize = files.iter().map(|p| crate::models::file_item::FileItem::new(p.to_path_buf()).unwrap()).collect())
                .unwrap_or_else(|err| {
                    eprintln!("Error listing files: {}", err);
                });
        }
    }

    if app.files_to_organize.is_empty() {
        ui.label("No files found in the selected folder.");
    } else {
        ui.label(format!("{} files found.", app.files_to_organize.len()));
    }
}