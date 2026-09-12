use crate::app::FileOrganizerApp;
use crate::core::file_scanner::format_bytes;
use std::collections::HashMap;

pub fn render(ui: &mut egui::Ui, app: &mut FileOrganizerApp) {
    ui.heading("Preview Organization");
    ui.label("Review how your files will be organized before proceeding");
    
    ui.separator();

    // Summary statistics
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(format!("📁 Total Files: {}", app.files_to_organize.len()));
            ui.separator();
            
            let total_size: u64 = app.files_to_organize.iter().map(|f| f.size).sum();
            ui.label(format!("💾 Total Size: {}", format_bytes(total_size)));
        });
    });

    ui.separator();

    // Group files by destination folder
    let grouped_files = group_by_destination(app);
    
    ui.label(format!("Files will be organized into {} folders:", grouped_files.len()));
    
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for (destination, files) in grouped_files.iter() {
                ui.collapsing(format!("📂 {} ({} files)", destination, files.len()), |ui| {
                    for file in files {
                        ui.horizontal(|ui| {
                            ui.label("  📄");
                            ui.label(&file.name);
                            ui.label(format!("({})", format_bytes(file.size)));
                        });
                    }
                });
            }
        });

    ui.separator();

    // Options
    ui.group(|ui| {
        ui.label("Options:");
        ui.checkbox(&mut app.config.behavior.copy_instead_of_move, "Copy files (instead of move)");
        ui.checkbox(&mut app.config.behavior.overwrite_existing, "Overwrite existing files");
        ui.checkbox(&mut app.config.behavior.dry_run, "Dry run (preview only, don't actually move files)");
    });

    ui.separator();

    // Navigation and action buttons
    ui.horizontal(|ui| {
        if ui.button("⬅ Back to Rules").clicked() {
            // Go back to rules configuration
        }
        
        ui.add_space(10.0);
        
        let button_text = if app.config.behavior.dry_run {
            "🔍 Simulate Organization"
        } else if app.config.behavior.copy_instead_of_move {
            "📋 Copy Files"
        } else {
            "🚀 Organize Files"
        };
        
        if ui.button(button_text).clicked() {
            execute_organization(app);
        }
    });

    // Warning for non-dry-run operations
    if !app.config.behavior.dry_run {
        ui.separator();
        ui.colored_label(
            egui::Color32::from_rgb(255, 165, 0),
            "⚠️ Warning: This will actually move/copy your files!"
        );
    }
}

fn group_by_destination(app: &FileOrganizerApp) -> HashMap<String, Vec<&crate::models::file_item::FileItem>> {
    let mut grouped: HashMap<String, Vec<&crate::models::file_item::FileItem>> = HashMap::new();
    
    for file in &app.files_to_organize {
        if let Some(ref dest) = file.destination {
            let folder = dest.parent()
                .and_then(|p| p.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            grouped.entry(folder).or_insert_with(Vec::new).push(file);
        }
    }
    
    grouped
}

fn execute_organization(app: &mut FileOrganizerApp) {
    if app.config.behavior.dry_run {
        println!("DRY RUN: Would organize {} files", app.files_to_organize.len());
        // Show results in UI
    } else {
        // Actually perform the organization
        match app.organizer.organize(&app.files_to_organize) {
            Ok(_) => {
                println!("Successfully organized {} files", app.files_to_organize.len());
                // Show success message in UI
            }
            Err(e) => {
                eprintln!("Error organizing files: {}", e);
                // Show error in UI
            }
        }
    }
}