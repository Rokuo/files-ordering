use crate::app::FileOrganizerApp;

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
        }
    });
    
    if app.selected_input_path.is_some() {
        if ui.button("Next: Configure Rules").clicked() {
            // Scan files and move to next view
        }
    }
}