use eframe::egui::{self, Color32, RichText, TextEdit, FontId, Theme};
use spruce::{format, query};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 720.0])
            .with_min_inner_size([600.0, 450.0])
            .with_title("🌲 Spruce — JSON & XML Formatter"),
        ..Default::default()
    };

    eframe::run_native(
        "Spruce",
        options,
        Box::new(|_cc| Ok(Box::new(SpruceApp::default()))),
    )
}

#[derive(PartialEq, Clone, Copy)]
enum AppMode {
    Format,
    Query,
    Tree,
}

struct SpruceApp {
    mode: AppMode,
    input: String,
    output: String,
    status: String,
    indent: usize,
    format_type: String,
    query_expr: String,
    current_file: Option<String>,
}

impl Default for SpruceApp {
    fn default() -> Self {
        Self {
            mode: AppMode::Format,
            input: String::new(),
            output: String::new(),
            status: "Ready".to_string(),
            indent: 2,
            format_type: "auto".to_string(),
            query_expr: String::new(),
            current_file: None,
        }
    }
}

impl SpruceApp {
    fn run_action(&mut self) {
        let input = self.input.clone();
        if input.trim().is_empty() {
            self.status = "⚠ Please enter input data".to_string();
            return;
        }

        let type_hint = match self.format_type.as_str() {
            "json" => Some("json"),
            "xml" => Some("xml"),
            _ => None,
        };

        self.status = "⏳ Processing...".to_string();
        self.output.clear();

        let result = match self.mode {
            AppMode::Format => self.run_format(&input, type_hint),
            AppMode::Query => self.run_query(&input, type_hint),
            AppMode::Tree => self.run_tree(&input, type_hint),
        };

        match result {
            Ok(out) => {
                self.output = out;
                let lines = self.output.lines().count();
                self.status = format!("✓ Done — {} lines, {} chars", lines, self.output.len());
            }
            Err(e) => {
                let err_msg = format!("✖ {}", e);
                self.output = err_msg;
                self.status = "✖ Error".to_string();
            }
        }
    }

    fn run_format(&self, input: &str, type_hint: Option<&str>) -> anyhow::Result<String> {
        let fmt = format::detect_format(input, self.current_file.as_deref(), type_hint)?;
        match fmt {
            "json" => format::format_json(input, self.indent, true),
            "xml" => format::format_xml(input, self.indent, true),
            _ => unreachable!(),
        }
    }

    fn run_query(&self, input: &str, type_hint: Option<&str>) -> anyhow::Result<String> {
        if self.query_expr.trim().is_empty() {
            return Err(anyhow::anyhow!("Please enter a query expression"));
        }
        let fmt = format::detect_format(input, self.current_file.as_deref(), type_hint)?;
        match fmt {
            "json" => query::query_json(input, &self.query_expr, true),
            "xml" => query::query_xml(input, &self.query_expr, true),
            _ => unreachable!(),
        }
    }

    fn run_tree(&self, input: &str, type_hint: Option<&str>) -> anyhow::Result<String> {
        let fmt = format::detect_format(input, self.current_file.as_deref(), type_hint)?;
        match fmt {
            "json" => format::tree_json(input, true),
            "xml" => format::tree_xml(input, true),
            _ => unreachable!(),
        }
    }

    fn load_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Data files", &["json", "xml"])
            .add_filter("JSON", &["json"])
            .add_filter("XML", &["xml"])
            .set_title("Open JSON or XML file")
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.input = content;
                    self.current_file = Some(path.display().to_string());
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    self.status = format!("📂 Loaded: {}", name);
                    self.output.clear();
                }
                Err(e) => {
                    self.status = format!("✖ Failed to read file: {}", e);
                }
            }
        }
    }

    fn save_output(&mut self) {
        if self.output.is_empty() || self.output.starts_with('✖') {
            self.status = "⚠ Nothing to save".to_string();
            return;
        }

        let default_name = match &self.current_file {
            Some(path) => {
                let p = std::path::Path::new(path);
                let stem = p.file_stem().unwrap_or_default().to_string_lossy();
                format!("{}.formatted", stem)
            }
            None => "output".to_string(),
        };

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .save_file()
        {
            match std::fs::write(&path, &self.output) {
                Ok(_) => self.status = format!("💾 Saved: {}", path.display()),
                Err(e) => self.status = format!("✖ Failed to save: {}", e),
            }
        }
    }

    fn header_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.heading(RichText::new("🌲 Spruce").color(Color32::from_rgb(46, 160, 67)).size(20.0));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                let modes = [
                    ("📝  Format", AppMode::Format),
                    ("🔍  Query", AppMode::Query),
                    ("🌳  Tree", AppMode::Tree),
                ];

                for (label, mode) in &modes {
                    let selected = self.mode == *mode;
                    let btn = egui::Button::new(
                        RichText::new(*label).size(14.0).color(if selected {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(180, 180, 190)
                        }),
                    )
                    .fill(if selected {
                        Color32::from_rgb(46, 160, 67)
                    } else {
                        Color32::from_rgb(40, 40, 48)
                    })
                    .min_size(egui::vec2(100.0, 30.0));

                    if ui.add(btn).clicked() {
                        self.mode = *mode;
                        self.output.clear();
                    }
                }
            });
        });
    }

    fn content_ui(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_height();

        // Input area (40%)
        let input_h = (avail * 0.40).max(100.0);
        egui::ScrollArea::vertical().id_salt("input_scroll").max_height(input_h).show(ui, |ui| {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(22, 22, 28))
                .show(ui, |ui| {
                    ui.set_min_height(input_h - 16.0);
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::multiline(&mut self.input)
                            .font(FontId::monospace(14.0))
                            .desired_width(f32::INFINITY)
                            .hint_text("Paste JSON or XML here, or click 📂 Open File..."),
                    );
                });
        });

        ui.add_space(6.0);

        // Controls
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(34, 34, 40))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("📂  Open File").size(13.0)).clicked() {
                        self.load_file();
                    }

                    ui.separator();

                    ui.label(RichText::new("Type:").size(13.0));
                    let types = ["auto", "json", "xml"];
                    egui::ComboBox::from_id_salt("type_sel")
                        .selected_text(self.format_type.clone())
                        .width(70.0)
                        .show_ui(ui, |ui| {
                            for t in &types {
                                ui.selectable_value(&mut self.format_type, t.to_string(), *t);
                            }
                        });

                    if self.mode == AppMode::Format {
                        ui.separator();
                        ui.label(RichText::new("Indent:").size(13.0));
                        ui.add(egui::Slider::new(&mut self.indent, 0..=8).integer().text("spaces"));
                    }

                    if self.mode == AppMode::Query {
                        ui.separator();
                        ui.label(RichText::new("Query:").size(13.0));
                        ui.add_sized(
                            egui::vec2(200.0, 24.0),
                            TextEdit::singleline(&mut self.query_expr)
                                .font(FontId::monospace(13.0))
                                .hint_text(".users[0].name"),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("▶  Run").size(14.0).color(Color32::BLACK))
                                .fill(Color32::from_rgb(46, 160, 67))
                                .min_size(egui::vec2(90.0, 30.0)),
                        ).clicked() {
                            self.run_action();
                        }

                        if ui.button(RichText::new("🗑  Clear").size(13.0)).clicked() {
                            self.input.clear();
                            self.output.clear();
                            self.status = "Cleared".to_string();
                        }
                    });
                });
            });

        ui.add_space(6.0);

        // Output area
        let output_h = ui.available_height().max(80.0) - 28.0;
        egui::ScrollArea::vertical().id_salt("output_scroll").max_height(output_h).show(ui, |ui| {
            let bg = if self.output.starts_with('✖') {
                Color32::from_rgb(38, 18, 18)
            } else {
                Color32::from_rgb(20, 20, 26)
            };

            egui::Frame::group(ui.style())
                .fill(bg)
                .show(ui, |ui| {
                    ui.set_min_height(output_h - 16.0);
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::multiline(&mut self.output)
                            .font(FontId::monospace(13.0))
                            .desired_width(f32::INFINITY)
                            .text_color(Color32::from_rgb(210, 210, 218)),
                    );
                });
        });

        ui.add_space(4.0);

        // Status bar
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(28, 28, 34))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let sc = if self.status.starts_with('✖') || self.status.starts_with("⚠") {
                        Color32::from_rgb(240, 120, 120)
                    } else if self.status.starts_with('✓') {
                        Color32::from_rgb(80, 200, 120)
                    } else {
                        Color32::from_rgb(160, 160, 170)
                    };
                    ui.label(RichText::new(&self.status).size(12.0).color(sc));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("💾  Save").size(12.0)).clicked() {
                            self.save_output();
                        }
                        if ui.button(RichText::new("📋  Copy").size(12.0)).clicked() {
                            ui.ctx().copy_text(self.output.clone());
                            self.status = "✓ Copied to clipboard".to_string();
                        }
                    });
                });
            });
    }
}

impl eframe::App for SpruceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Dark theme
        let mut style = (*ui.ctx().style_of(Theme::Light)).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_fill = Color32::from_rgb(28, 28, 34);
        style.visuals.panel_fill = Color32::from_rgb(34, 34, 40);
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(40, 40, 48);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(44, 44, 52);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(52, 52, 62);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 48, 58);
        ui.ctx().set_style_of(Theme::Dark, style);

        // Header
        egui::Panel::top("header").show(ui, |ui| {
            self.header_ui(ui);
        });

        // Main content
        egui::CentralPanel::default().show(ui, |ui| {
            self.content_ui(ui);
        });
    }
}
