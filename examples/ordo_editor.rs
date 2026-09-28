//! A visual spike: the design canvas's "Rule editor" screen, drawn in egui.
//!
//! Nothing here touches the engine — the state below is a throwaway mirror of
//! the real model, and no rule is ever saved. The point is to prove the Ordo
//! look and feel are reachable in egui before Phase 4 commits to them.
//!
//!     cargo run --example ordo_editor
//!
//! Delete this file once `p4-apply` builds the real screen.

use eframe::egui;
use egui::text::LayoutJob;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, TextFormat, Vec2};

// ------------------------------------------------------------------ tokens
// Colours are taken verbatim from the design canvas.

const DESKTOP: Color32 = Color32::from_rgb(0xe6, 0xd7, 0xbd);
const SHELL: Color32 = Color32::from_rgb(0xf5, 0xea, 0xd8);
const CHROME: Color32 = Color32::from_rgb(0xeb, 0xdd, 0xc5);
const CARD: Color32 = Color32::from_rgb(0xf9, 0xf4, 0xed);
const INK: Color32 = Color32::from_rgb(0x20, 0x1e, 0x1d);
const INK2: Color32 = Color32::from_rgb(0x47, 0x42, 0x38);
const N700: Color32 = Color32::from_rgb(0x64, 0x5c, 0x50);
const META: Color32 = Color32::from_rgb(0x82, 0x79, 0x6a);
const FAINT: Color32 = Color32::from_rgb(0xa1, 0x97, 0x86);
const N300: Color32 = Color32::from_rgb(0xdc, 0xd3, 0xc4);
const N200: Color32 = Color32::from_rgb(0xee, 0xe7, 0xdb);
const ACCENT: Color32 = Color32::from_rgb(0xc6, 0x71, 0x39);
const ACCENT_HOV: Color32 = Color32::from_rgb(0xb2, 0x62, 0x2d);
const ACC700: Color32 = Color32::from_rgb(0x8c, 0x49, 0x1a);
const ACC_EDGE: Color32 = Color32::from_rgb(0xf6, 0xa0, 0x6b);
const ACC200: Color32 = Color32::from_rgb(0xff, 0xe1, 0xd0);
const ACC100: Color32 = Color32::from_rgb(0xff, 0xf2, 0xeb);
const SAGE_BG: Color32 = Color32::from_rgb(0xf0, 0xfa, 0xe1);
const SAGE_FG: Color32 = Color32::from_rgb(0x56, 0x63, 0x3f);
const HAIRLINE: Color32 = Color32::from_rgb(0xc0, 0xb6, 0xa5);

// One type scale and one radius scale, declared once, used everywhere.

const SIZE_DISPLAY: f32 = 18.0;
const SIZE_BODY: f32 = 13.0;
const SIZE_CONTROL: f32 = 13.0;
const SIZE_LABEL: f32 = 12.5;
const SIZE_SMALL: f32 = 12.0;
const SIZE_HINT: f32 = 10.5;

const RADIUS_CARD: u8 = 28;
const RADIUS_PILL: u8 = 18;
const RADIUS_ROW: u8 = 16;
const RADIUS_CHIP: u8 = 12;

fn mono(size: f32) -> egui::FontId {
    egui::FontId::monospace(size)
}

fn sans(size: f32) -> egui::FontId {
    egui::FontId::proportional(size)
}

/// A card heading. Stands in for Caprasimo, which Phase 4 will vendor.
fn t_display(text: &str) -> RichText {
    RichText::new(text).size(SIZE_DISPLAY).color(INK).strong()
}

fn t_body(text: &str) -> RichText {
    RichText::new(text).size(SIZE_BODY).color(N700)
}

fn t_label(text: &str) -> RichText {
    RichText::new(text).size(SIZE_LABEL).color(INK2).strong()
}

fn t_meta(text: &str) -> RichText {
    RichText::new(text).size(SIZE_SMALL).color(META)
}

fn t_mono(text: &str, size: f32, color: Color32) -> RichText {
    RichText::new(text).font(mono(size)).color(color)
}

// ------------------------------------------------------------------- state

/// The attribute a condition tests — the outer level of the real model's
/// `Condition` enum. Each field owns its own operator list, which is why the
/// operator dropdown can only ever offer valid choices: there is no sequence
/// of clicks that produces "Size contains".
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Extension,
    Stem,
    Modified,
    Size,
}

impl Field {
    const ALL: [Field; 4] = [Field::Extension, Field::Stem, Field::Modified, Field::Size];

    fn label(self) -> &'static str {
        match self {
            Field::Extension => "Extension",
            Field::Stem => "File name",
            Field::Modified => "Date modified",
            Field::Size => "Size",
        }
    }

    fn ops(self) -> &'static [&'static str] {
        match self {
            Field::Extension => &["is one of", "is not one of"],
            Field::Stem => &["contains", "starts with", "ends with"],
            Field::Modified => &["is after", "is before"],
            Field::Size => &["is larger than", "is smaller than"],
        }
    }

    /// A believable value, used when a row switches to this attribute.
    fn example(self) -> &'static str {
        match self {
            Field::Extension => "png",
            Field::Stem => "invoice",
            Field::Modified => "2024-01-01",
            Field::Size => "500 KB",
        }
    }
}

struct Condition {
    field: Field,
    op: String,
    values: Vec<String>,
}

impl Condition {
    fn new(field: Field) -> Self {
        Self {
            field,
            op: field.ops()[0].to_owned(),
            values: vec![field.example().to_owned()],
        }
    }
}

struct Spike {
    rule_name: String,
    match_all: bool,
    conditions: Vec<Condition>,
    pattern: String,
    rename_on: bool,
    find: String,
    replace: String,
}

impl Default for Spike {
    fn default() -> Self {
        Self {
            rule_name: "Camera dump".to_owned(),
            match_all: true,
            conditions: vec![
                Condition {
                    field: Field::Extension,
                    op: "is one of".to_owned(),
                    values: vec!["jpg".to_owned(), "heic".to_owned(), "raf".to_owned()],
                },
                Condition {
                    field: Field::Modified,
                    op: "is after".to_owned(),
                    values: vec!["2024-01-01".to_owned()],
                },
                Condition {
                    field: Field::Size,
                    op: "is larger than".to_owned(),
                    values: vec!["500 KB".to_owned()],
                },
            ],
            pattern: "{year}/{month}/{type}".to_owned(),
            rename_on: true,
            find: r"^IMG_(\d+)".to_owned(),
            replace: "holiday-$1".to_owned(),
        }
    }
}

const TOKENS: &[(&str, &str)] = &[
    ("{year}", "2026"),
    ("{month}", "09"),
    ("{day}", "10"),
    ("{ext}", "heic"),
    ("{type}", "photos"),
    ("{name}", "IMG_4471"),
    ("{size-band}", "large"),
    ("{parent}", "source folder"),
    ("{counter}", "001"),
];

// ------------------------------------------------------------------ blocks

/// One of the big rounded panels the screen is built from.
fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, HAIRLINE))
        .corner_radius(CornerRadius::same(RADIUS_CARD))
        .inner_margin(Margin::symmetric(22, 20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

/// A rounded text field on the shell colour.
fn pill_input(ui: &mut egui::Ui, value: &mut String, width: f32, monospaced: bool) {
    let font = if monospaced {
        mono(SIZE_BODY)
    } else {
        sans(14.0)
    };
    Frame::new()
        .fill(SHELL)
        .stroke(Stroke::new(1.0, HAIRLINE))
        .corner_radius(CornerRadius::same(RADIUS_PILL))
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(value)
                    .desired_width(width)
                    .frame(Frame::new())
                    .font(font)
                    .text_color(INK),
            );
        });
}

/// Widget states for an accent chip that lifts on hover.
fn accent_chip_visuals(ui: &mut egui::Ui, radius: u8) {
    let widgets = &mut ui.visuals_mut().widgets;
    for state in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
    ] {
        state.corner_radius = CornerRadius::same(radius);
    }
    widgets.inactive.weak_bg_fill = ACC100;
    widgets.inactive.bg_stroke = Stroke::new(1.0, ACC200);
    widgets.hovered.weak_bg_fill = ACC200;
    widgets.hovered.bg_stroke = Stroke::new(1.0, ACC_EDGE);
    widgets.active.weak_bg_fill = ACC200;
    widgets.active.bg_stroke = Stroke::new(1.0, ACC700);
}

/// A dropdown pill, laid out and painted by hand so the padding, the height
/// and the chevron all sit exactly where the design puts them. `Button` would
/// size itself from its text, which leaves no room for the chevron.
fn dropdown(
    ui: &mut egui::Ui,
    salt: (&'static str, usize),
    selected: &str,
    options: &[&str],
) -> Option<usize> {
    const PAD_X: f32 = 14.0;
    const PAD_Y: f32 = 7.0;
    const CHEVRON_GAP: f32 = 9.0;
    const CHEVRON_W: f32 = 9.0;

    let galley = ui
        .painter()
        .layout_no_wrap(selected.to_owned(), sans(SIZE_CONTROL), INK);

    let size = Vec2::new(
        galley.size().x + PAD_X * 2.0 + CHEVRON_GAP + CHEVRON_W,
        (galley.size().y + PAD_Y * 2.0).max(30.0),
    );

    let response = ui
        .push_id(salt, |ui| {
            ui.allocate_exact_size(size, egui::Sense::click()).1
        })
        .inner;

    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
    let fill = if open || response.is_pointer_button_down_on() {
        N300
    } else if response.hovered() {
        N200
    } else {
        CARD
    };

    let rect = response.rect;
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(RADIUS_ROW),
        fill,
        Stroke::new(1.0, HAIRLINE),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        egui::pos2(rect.left() + PAD_X, rect.center().y - galley.size().y / 2.0),
        galley,
        INK,
    );

    // egui's bundled fonts carry no chevron glyph, so draw one.
    let tip = egui::pos2(
        rect.right() - PAD_X - CHEVRON_W / 2.0,
        rect.center().y + 1.5,
    );
    let stroke = Stroke::new(1.6, META);
    painter.line_segment([tip + Vec2::new(-4.0, -4.0), tip], stroke);
    painter.line_segment([tip, tip + Vec2::new(4.0, -4.0)], stroke);

    let mut picked = None;
    egui::Popup::menu(&response)
        .gap(5.0)
        .frame(
            Frame::new()
                .fill(CARD)
                .stroke(Stroke::new(1.0, HAIRLINE))
                .corner_radius(CornerRadius::same(RADIUS_ROW))
                .inner_margin(Margin::same(6)),
        )
        .show(|ui| {
            ui.set_min_width(rect.width().max(170.0));
            ui.spacing_mut().item_spacing.y = 2.0;
            for (index, option) in options.iter().enumerate() {
                if menu_row(ui, option, *option == selected) {
                    picked = Some(index);
                }
            }
        });

    picked
}

/// One row inside a dropdown menu.
fn menu_row(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::new(10.0, 6.0);
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = if selected {
            ACC200
        } else {
            Color32::TRANSPARENT
        };
        widgets.hovered.weak_bg_fill = N200;
        widgets.active.weak_bg_fill = N300;
        for state in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
        ] {
            state.corner_radius = CornerRadius::same(10);
            state.bg_stroke = Stroke::NONE;
        }
        let color = if selected { ACC700 } else { INK };
        ui.add(
            egui::Button::new(RichText::new(label).size(SIZE_CONTROL).color(color))
                .min_size(Vec2::new(ui.available_width(), 0.0)),
        )
        .clicked()
    })
    .inner
}

/// An accent chip holding one condition value. True when clicked to remove.
fn value_chip(ui: &mut egui::Ui, text: &str) -> bool {
    let mut clicked = false;
    ui.scope(|ui| {
        accent_chip_visuals(ui, RADIUS_CHIP);
        ui.spacing_mut().button_padding = Vec2::new(10.0, 4.0);
        ui.spacing_mut().interact_size.y = 0.0;
        let label = RichText::new(format!("{text}  \u{00D7}"))
            .font(mono(SIZE_SMALL))
            .color(ACC700)
            .strong();
        if ui
            .add(egui::Button::new(label))
            .on_hover_text("Remove this value")
            .clicked()
        {
            clicked = true;
        }
    });
    clicked
}

/// A two-line token chip: the token above, an example value below.
fn token_chip(ui: &mut egui::Ui, token: &str, hint: &str) -> bool {
    let mut job = LayoutJob::default();
    job.append(
        token,
        0.0,
        TextFormat {
            font_id: mono(SIZE_SMALL),
            color: ACC700,
            ..Default::default()
        },
    );
    job.append(
        &format!("\n{hint}"),
        0.0,
        TextFormat {
            font_id: sans(SIZE_HINT),
            color: META,
            ..Default::default()
        },
    );

    let mut clicked = false;
    ui.scope(|ui| {
        accent_chip_visuals(ui, RADIUS_ROW);
        ui.spacing_mut().button_padding = Vec2::new(11.0, 6.0);
        ui.spacing_mut().interact_size.y = 0.0;
        if ui
            .add(egui::Button::new(job))
            .on_hover_text("Insert into the pattern")
            .clicked()
        {
            clicked = true;
        }
    });
    clicked
}

/// A quiet outline button, as used for "Add condition" and "Cancel".
fn ghost_button(ui: &mut egui::Ui, text: &str) -> bool {
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.hovered.weak_bg_fill = N200;
        widgets.active.weak_bg_fill = N300;
        for state in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
        ] {
            state.corner_radius = CornerRadius::same(RADIUS_PILL);
            state.bg_stroke = Stroke::new(1.0, HAIRLINE);
        }
        ui.add(egui::Button::new(
            RichText::new(text).size(SIZE_CONTROL).color(INK),
        ))
        .clicked()
    })
    .inner
}

/// The single accent action on a screen.
fn primary_button(ui: &mut egui::Ui, text: &str) -> bool {
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = ACCENT;
        widgets.hovered.weak_bg_fill = ACCENT_HOV;
        widgets.active.weak_bg_fill = ACC700;
        for state in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
        ] {
            state.corner_radius = CornerRadius::same(RADIUS_PILL);
            state.bg_stroke = Stroke::NONE;
        }
        ui.add(egui::Button::new(
            RichText::new(text).size(SIZE_CONTROL).color(SHELL).strong(),
        ))
        .clicked()
    })
    .inner
}

/// A small round remove control, used to drop a condition row.
fn remove_button(ui: &mut egui::Ui) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
    let response = response.on_hover_text("Remove this condition");
    let hovered = response.hovered();
    if hovered {
        ui.painter().circle_filled(rect.center(), 14.0, ACC200);
    }
    let stroke = Stroke::new(1.7, if hovered { ACC700 } else { FAINT });
    let c = rect.center();
    let r = 4.5;
    let painter = ui.painter();
    painter.line_segment([c + Vec2::new(-r, -r), c + Vec2::new(r, r)], stroke);
    painter.line_segment([c + Vec2::new(r, -r), c + Vec2::new(-r, r)], stroke);
    response.clicked()
}

/// The All / Any segmented control.
fn segmented(ui: &mut egui::Ui, match_all: &mut bool) {
    Frame::new()
        .fill(SHELL)
        .stroke(Stroke::new(1.0, HAIRLINE))
        .corner_radius(CornerRadius::same(RADIUS_ROW))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.spacing_mut().button_padding = Vec2::new(15.0, 5.0);
            for (label, is_all) in [("All conditions", true), ("Any condition", false)] {
                let selected = *match_all == is_all;
                let (fill, fg) = if selected {
                    (ACCENT, SHELL)
                } else {
                    (Color32::TRANSPARENT, N700)
                };
                if ui
                    .add(
                        egui::Button::new(RichText::new(label).size(SIZE_LABEL).color(fg).strong())
                            .fill(fill)
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(14)),
                    )
                    .clicked()
                {
                    *match_all = is_all;
                }
            }
        });
}

/// The rename on/off switch, painted by hand.
fn switch(ui: &mut egui::Ui, on: &mut bool) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(46.0, 26.0), egui::Sense::click());
    if response.clicked() {
        *on = !*on;
    }
    let track = if *on {
        if response.hovered() {
            ACCENT_HOV
        } else {
            ACCENT
        }
    } else if response.hovered() {
        HAIRLINE
    } else {
        N300
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(13), track);
    let knob_x = if *on {
        rect.right() - 13.0
    } else {
        rect.left() + 13.0
    };
    ui.painter()
        .circle_filled(egui::pos2(knob_x, rect.center().y), 10.0, CARD);
}

// ------------------------------------------------------------------ screen

impl Spike {
    fn name_card(&mut self, ui: &mut egui::Ui) {
        let rule_name = &mut self.rule_name;
        card(ui, |ui| {
            ui.label(t_label("Rule name"));
            ui.add_space(7.0);
            pill_input(ui, rule_name, 380.0, false);
        });
    }

    fn match_card(&mut self, ui: &mut egui::Ui) {
        let mut drop_row: Option<usize> = None;
        let mut drop_value: Option<(usize, usize)> = None;
        let mut add_row = false;

        let conditions = &mut self.conditions;
        let match_all = &mut self.match_all;

        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t_display("Match"));
                ui.add_space(6.0);
                segmented(ui, match_all);
            });
            ui.add_space(14.0);

            let joiner_for = |row: usize, all: bool| {
                if row == 0 {
                    "where"
                } else if all {
                    "and"
                } else {
                    "or"
                }
            };

            let last = conditions.len().saturating_sub(1);
            for (row, condition) in conditions.iter_mut().enumerate() {
                Frame::new()
                    .fill(SHELL)
                    .stroke(Stroke::new(1.0, HAIRLINE))
                    .corner_radius(CornerRadius::same(RADIUS_ROW))
                    .inner_margin(Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                Vec2::new(36.0, 22.0),
                                egui::Label::new(t_meta(joiner_for(row, *match_all)))
                                    .selectable(false),
                            );

                            let labels: Vec<&str> = Field::ALL.iter().map(|f| f.label()).collect();
                            if let Some(index) =
                                dropdown(ui, ("field", row), condition.field.label(), &labels)
                                && Field::ALL[index] != condition.field
                            {
                                // Operators are scoped to the attribute, so a new
                                // field falls back to that field's first operator
                                // and its old values stop meaning anything.
                                condition.field = Field::ALL[index];
                                condition.op = condition.field.ops()[0].to_owned();
                                condition.values = vec![condition.field.example().to_owned()];
                            }

                            let ops = condition.field.ops();
                            if let Some(index) = dropdown(ui, ("op", row), &condition.op, ops) {
                                condition.op = ops[index].to_owned();
                            }

                            for (index, value) in condition.values.iter().enumerate() {
                                if value_chip(ui, value) {
                                    drop_value = Some((row, index));
                                }
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if remove_button(ui) {
                                        drop_row = Some(row);
                                    }
                                },
                            );
                        });
                    });
                if row != last {
                    ui.add_space(9.0);
                }
            }

            ui.add_space(12.0);
            if ghost_button(ui, "+  Add condition") {
                add_row = true;
            }
        });

        if let Some((row, index)) = drop_value {
            let values = &mut self.conditions[row].values;
            if values.len() > 1 {
                values.remove(index);
            }
        }
        if let Some(row) = drop_row
            && self.conditions.len() > 1
        {
            self.conditions.remove(row);
        }
        if add_row {
            self.conditions.push(Condition::new(Field::Stem));
        }
    }

    fn pattern_card(&mut self, ui: &mut egui::Ui) {
        let pattern = &mut self.pattern;
        card(ui, |ui| {
            ui.label(t_display("Destination pattern"));
            ui.add_space(4.0);
            ui.label(t_body(
                "Folders are created from the pattern. \
                 Type freely, or click a token to insert it.",
            ));
            ui.add_space(14.0);

            Frame::new()
                .fill(SHELL)
                .stroke(Stroke::new(1.0, HAIRLINE))
                .corner_radius(CornerRadius::same(20))
                .inner_margin(Margin::symmetric(16, 5))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(t_mono("~/Pictures/", SIZE_BODY, META));
                        ui.add(
                            egui::TextEdit::singleline(pattern)
                                .desired_width(ui.available_width() - 100.0)
                                .frame(Frame::new())
                                .font(mono(13.5))
                                .text_color(INK),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add(
                                egui::Button::new(
                                    RichText::new("Browse\u{2026}")
                                        .size(SIZE_SMALL)
                                        .color(N700)
                                        .strong(),
                                )
                                .fill(N200)
                                .stroke(Stroke::NONE)
                                .corner_radius(CornerRadius::same(RADIUS_ROW)),
                            );
                        });
                    });
                });

            ui.add_space(14.0);
            ui.horizontal_wrapped(|ui| {
                for (token, hint) in TOKENS {
                    if token_chip(ui, token, hint) {
                        if !pattern.is_empty() && !pattern.ends_with('/') {
                            pattern.push('/');
                        }
                        pattern.push_str(token);
                    }
                }
            });
        });
    }

    fn rename_card(&mut self, ui: &mut egui::Ui) {
        let rename_on = &mut self.rename_on;
        let find = &mut self.find;
        let replace = &mut self.replace;

        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t_display("Rename files"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    switch(ui, rename_on);
                });
            });

            if !*rename_on {
                ui.add_space(2.0);
                ui.label(t_body("Files keep the name they already have."));
                return;
            }

            ui.add_space(14.0);
            ui.columns(2, |cols| {
                cols[0].label(t_label("Find (regular expression)"));
                cols[0].add_space(6.0);
                pill_input(&mut cols[0], find, 180.0, true);

                cols[1].label(t_label("Replace with"));
                cols[1].add_space(6.0);
                pill_input(&mut cols[1], replace, 180.0, true);
            });

            ui.add_space(12.0);
            Frame::new()
                .fill(SAGE_BG)
                .corner_radius(CornerRadius::same(RADIUS_ROW))
                .inner_margin(Margin::symmetric(14, 9))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (tick, _) =
                            ui.allocate_exact_size(Vec2::new(15.0, 15.0), egui::Sense::hover());
                        let stroke = Stroke::new(2.2, SAGE_FG);
                        let painter = ui.painter();
                        painter.line_segment(
                            [
                                tick.left_center() + Vec2::new(1.0, 0.0),
                                tick.center() + Vec2::new(-1.0, 4.0),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                tick.center() + Vec2::new(-1.0, 4.0),
                                tick.right_center() + Vec2::new(-1.0, -5.0),
                            ],
                            stroke,
                        );
                        ui.label(t_mono(
                            "IMG_4471.HEIC  \u{2192}  holiday-4471.heic",
                            SIZE_LABEL,
                            SAGE_FG,
                        ));
                    });
                });
        });
    }

    fn preview_line(&self) -> String {
        let renamed = if self.rename_on {
            "holiday-4471.heic"
        } else {
            "IMG_4471.heic"
        };
        let pattern = self.pattern.trim_matches('/');
        if pattern.is_empty() {
            format!("IMG_4471.HEIC  \u{2192}  ~/Pictures/{renamed}")
        } else {
            format!("IMG_4471.HEIC  \u{2192}  ~/Pictures/{pattern}/{renamed}")
        }
    }
}

fn apply_theme(ui: &mut egui::Ui) {
    // Global spacing — the design breathes more than egui does by default.
    let spacing = ui.spacing_mut();
    spacing.button_padding = Vec2::new(14.0, 7.0);
    spacing.interact_size.y = 30.0;
    spacing.menu_margin = Margin::same(6);

    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(INK);
    visuals.panel_fill = DESKTOP;
    visuals.window_fill = CARD;
    visuals.window_stroke = Stroke::new(1.0, HAIRLINE);
    visuals.extreme_bg_color = SHELL;
    visuals.selection.bg_fill = ACC200;
    visuals.selection.stroke = Stroke::new(1.0, ACC700);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, HAIRLINE);
}

impl eframe::App for Spike {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_theme(ui);

        egui::Panel::bottom("ordo-footer")
            .frame(
                Frame::new()
                    .fill(CHROME)
                    .stroke(Stroke::new(1.0, HAIRLINE))
                    .inner_margin(Margin::symmetric(24, 14)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t_mono("Preview", SIZE_LABEL, META));
                    ui.label(t_mono(&self.preview_line(), SIZE_LABEL, INK2));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        primary_button(ui, "Save rule");
                        ghost_button(ui, "Cancel");
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(DESKTOP)
                    .inner_margin(Margin::symmetric(24, 22)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // A centred column, but children still left-aligned:
                        // `vertical_centered` would centre every label inside it.
                        let column = 880.0_f32.min(ui.available_width());
                        let gutter = ((ui.available_width() - column) * 0.5).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_space(gutter);
                            ui.vertical(|ui| {
                                ui.set_width(column);
                                ui.spacing_mut().item_spacing = Vec2::new(8.0, 16.0);

                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("RULE EDITOR")
                                            .size(11.0)
                                            .color(META)
                                            .strong(),
                                    );
                                });

                                self.name_card(ui);
                                self.match_card(ui);
                                self.pattern_card(ui);
                                self.rename_card(ui);
                                ui.add_space(4.0);
                            });
                        });
                    });
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 900.0])
            .with_min_inner_size([560.0, 500.0])
            .with_title("Ordo \u{2014} rule editor (visual spike)"),
        ..Default::default()
    };

    eframe::run_native(
        "ordo-spike",
        options,
        Box::new(|_cc| Ok(Box::new(Spike::default()))),
    )
}
