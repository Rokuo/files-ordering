use crate::core::organizer::Organizer;
use crate::models::file_item::FileItem;
use crate::core::rules::RuleEngine;
use crate::models::config::AppConfig;
use egui::Context;

pub struct FileOrganizerApp {
    pub organizer: Organizer,
    pub rule_engine: RuleEngine,
    pub config: AppConfig,
    pub selected_input_path: Option<String>,
    pub selected_output_path: Option<String>,
    pub files_to_organize: Vec<FileItem>,
    pub current_view: AppView,
}

enum AppView {
    FileSelection,
    RulesConfiguration,
    Preview,
    Processing,
}

impl Default for FileOrganizerApp {
    fn default() -> Self {
        Self {
            organizer: Organizer::new(),
            rule_engine: RuleEngine::new(),
            config: AppConfig::default(),
            selected_input_path: None,
            selected_output_path: None,
            files_to_organize: Vec::new(),
            current_view: AppView::FileSelection,
        }
    }
}

impl eframe::App for FileOrganizerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Organizer");
            
            ui.separator();
            
            match self.current_view {
                AppView::FileSelection => {
                    crate::ui::file_selector::render(ui, self);
                }
                AppView::RulesConfiguration => {
                    crate::ui::rules_panel::render(ui, self);
                }
                AppView::Preview => {
                    crate::ui::preview_panel::render(ui, self);
                }
                AppView::Processing => {
                    ui.label("Processing files...");
                }
            }
        });
    }
}