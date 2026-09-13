use crate::models::file_item::FileItem;

pub struct FileOrganizerApp {
    pub selected_input_path: Option<String>,
    pub selected_output_path: Option<String>,
    pub files_to_organize: Vec<FileItem>,
    pub current_view: AppView,
}

pub enum AppView {
    FileSelection,
}

impl Default for FileOrganizerApp {
    fn default() -> Self {
        Self {
            selected_input_path: None,
            selected_output_path: None,
            files_to_organize: Vec::new(),
            current_view: AppView::FileSelection,
        }
    }
}

impl eframe::App for FileOrganizerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("File Organizer");

            ui.separator();

            match self.current_view {
                AppView::FileSelection => {
                    crate::ui::file_selector::render(ui, self);
                }
            }
        });
    }
}
