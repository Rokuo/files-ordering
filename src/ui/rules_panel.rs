use crate::app::FileOrganizerApp;
use crate::core::rules::{DateFormat, OrganizationRule, NamePattern, default_size_ranges};

pub fn render(ui: &mut egui::Ui, app: &mut FileOrganizerApp) {
    ui.heading("Configure Organization Rules");
    ui.label("Add rules to determine how your files will be organized");
    
    ui.separator();

    // Display current rules
    ui.group(|ui| {
        ui.label("Active Rules:");
        
        if app.organizer.get_rules().is_empty() {
            ui.label("No rules configured yet");
        } else {
            let mut rule_to_remove = None;
            
            for (index, rule) in app.organizer.get_rules().iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", index + 1));
                    ui.label(format_rule(rule));
                    
                    if ui.button("❌").clicked() {
                        rule_to_remove = Some(index);
                    }
                });
            }
            
            if let Some(index) = rule_to_remove {
                app.organizer.remove_rule(index);
            }
        }
    });

    ui.separator();

    // Add new rule section
    ui.heading("Add New Rule");
    
    ui.horizontal(|ui| {
        if ui.button("📁 By Extension").clicked() {
            app.organizer.add_rule(OrganizationRule::ByExtension);
        }
        
        if ui.button("📅 By Date (Year/Month)").clicked() {
            app.organizer.add_rule(OrganizationRule::ByDate {
                format: DateFormat::YearMonth,
            });
        }
        
        if ui.button("📏 By Size").clicked() {
            app.organizer.add_rule(OrganizationRule::BySize {
                ranges: default_size_ranges(),
            });
        }
    });

    ui.separator();

    // Navigation buttons
    ui.horizontal(|ui| {
        if ui.button("⬅ Back").clicked() {
            // Go back to file selection
        }
        
        ui.add_space(10.0);
        
        if !app.organizer.get_rules().is_empty() {
            if ui.button("Next: Preview ➡").clicked() {
                // Move to preview
            }
        }
    });

    // Advanced rule configuration
    ui.collapsing("Advanced Rule Options", |ui| {
        ui.label("Date Format:");
        ui.horizontal(|ui| {
            if ui.button("Year Only").clicked() {
                app.organizer.add_rule(OrganizationRule::ByDate {
                    format: DateFormat::Year,
                });
            }
            if ui.button("Year/Month/Day").clicked() {
                app.organizer.add_rule(OrganizationRule::ByDate {
                    format: DateFormat::YearMonthDay,
                });
            }
        });

        ui.separator();

        ui.label("Name Pattern:");
        ui.horizontal(|ui| {
            let mut pattern_text = String::new();
            ui.label("Starts with:");
            ui.text_edit_singleline(&mut pattern_text);
            if ui.button("Add").clicked() && !pattern_text.is_empty() {
                app.organizer.add_rule(OrganizationRule::ByName {
                    pattern: NamePattern::StartsWith(pattern_text),
                });
            }
        });
    });
}

fn format_rule(rule: &OrganizationRule) -> String {
    match rule {
        OrganizationRule::ByExtension => "Organize by file extension".to_string(),
        OrganizationRule::ByDate { format } => {
            let format_str = match format {
                DateFormat::Year => "YYYY",
                DateFormat::YearMonth => "YYYY/MM",
                DateFormat::YearMonthDay => "YYYY/MM/DD",
            };
            format!("Organize by date ({})", format_str)
        }
        OrganizationRule::BySize { ranges } => {
            format!("Organize by size ({} ranges)", ranges.len())
        }
        OrganizationRule::ByName { pattern } => {
            let pattern_str = match pattern {
                NamePattern::StartsWith(s) => format!("starts with '{}'", s),
                NamePattern::Contains(s) => format!("contains '{}'", s),
                NamePattern::EndsWith(s) => format!("ends with '{}'", s),
                NamePattern::Regex(s) => format!("matches regex '{}'", s),
            };
            format!("Organize by name ({})", pattern_str)
        }
        OrganizationRule::Custom { name, .. } => {
            format!("Custom rule: {}", name)
        }
    }
}