use eframe::egui::{self, Color32, RichText, TextEdit, FontId, Theme, Frame, Margin};
use spruce::{format, query};

// ─── Catppuccin Mocha palette ───
mod palette {
    use eframe::egui::Color32;
    pub const BASE:     Color32 = Color32::from_rgb(30,  30,  36);
    pub const MANTLE:   Color32 = Color32::from_rgb(24,  24,  29);
    pub const _CRUST:    Color32 = Color32::from_rgb(20,  20,  24);
    pub const SURFACE0: Color32 = Color32::from_rgb(40,  40,  48);
    pub const SURFACE1: Color32 = Color32::from_rgb(48,  48,  56);
    pub const SURFACE2: Color32 = Color32::from_rgb(56,  56,  64);
    pub const OVERLAY0: Color32 = Color32::from_rgb(76,  76,  86);
    pub const SUBTEXT0: Color32 = Color32::from_rgb(166, 166, 176);
    pub const TEXT:     Color32 = Color32::from_rgb(205, 205, 215);
    pub const GREEN:    Color32 = Color32::from_rgb(166, 227, 161);
    pub const TEAL:     Color32 = Color32::from_rgb(148, 226, 213);
    pub const BLUE:     Color32 = Color32::from_rgb(137, 180, 250);
    pub const MAUVE:    Color32 = Color32::from_rgb(203, 166, 247);
    pub const PINK:     Color32 = Color32::from_rgb(245, 194, 231);
    pub const RED:      Color32 = Color32::from_rgb(243, 139, 168);
    pub const PEACH:    Color32 = Color32::from_rgb(250, 179, 135);
    pub const SAPPHIRE: Color32 = Color32::from_rgb(117, 202, 242);
    pub const ACCENT:   Color32 = Color32::from_rgb(166, 227, 161);  // green
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 740.0])
            .with_min_inner_size([680.0, 500.0])
            .with_title("🌲 Spruce — JSON & XML Formatter"),
        ..Default::default()
    };

    eframe::run_native(
        "Spruce",
        options,
        Box::new(|_cc| {
            Ok(Box::new(SpruceApp::default()))
        }),
    )
}

#[derive(PartialEq, Clone, Copy)]
enum AppMode { Format, Query, Tree }

struct SpruceApp {
    mode: AppMode,
    input: String,
    output_raw: String,
    output_html: Vec<StyledSegment>,
    status: String,
    indent: usize,
    format_type: String,
    query_expr: String,
    current_file: Option<String>,
    output_is_error: bool,
}

#[derive(Clone)]
struct StyledSegment {
    text: String,
    color: Color32,
}

impl Default for SpruceApp {
    fn default() -> Self {
        Self {
            mode: AppMode::Format,
            input: String::new(),
            output_raw: String::new(),
            output_html: Vec::new(),
            status: "Ready  🌲  Spruce up your data".to_string(),
            indent: 2,
            format_type: "auto".to_string(),
            query_expr: String::new(),
            current_file: None,
            output_is_error: false,
        }
    }
}

impl SpruceApp {
    // ── Core actions ──

    fn run_action(&mut self) {
        if self.input.trim().is_empty() {
            self.status = "⚠  Please enter some data first".to_string();
            return;
        }

        let type_hint = match self.format_type.as_str() {
            "json" => Some("json"),
            "xml"  => Some("xml"),
            _      => None,
        };

        self.status = "⏳  Processing…".to_string();
        self.output_raw.clear();
        self.output_html.clear();

        let result = match self.mode {
            AppMode::Format => self.run_format(&self.input, type_hint),
            AppMode::Query  => self.run_query(&self.input, type_hint),
            AppMode::Tree   => self.run_tree(&self.input, type_hint),
        };

        match result {
            Ok(out) => {
                self.output_raw = out;
                self.output_is_error = false;
                // Color the output using ANSI parsing
                self.output_html = parse_ansi(&self.output_raw);
                let lines = self.output_raw.lines().count();
                let mode_name = match self.mode {
                    AppMode::Format => "Formatted",
                    AppMode::Query  => "Query result",
                    AppMode::Tree   => "Tree view",
                };
                self.status = format!("✓  {} — {} lines, {} chars", mode_name, lines, self.output_raw.len());
            }
            Err(e) => {
                let msg = format!("✖  {}", e);
                self.output_raw = msg.clone();
                self.output_is_error = true;
                self.output_html = vec![StyledSegment { text: msg, color: palette::RED }];
                self.status = "✖  Error".to_string();
            }
        }
    }

    fn run_format(&self, input: &str, hint: Option<&str>) -> anyhow::Result<String> {
        let fmt = format::detect_format(input, self.current_file.as_deref(), hint)?;
        match fmt {
            "json" => format::format_json(input, self.indent, false),  // false = with color
            "xml"  => format::format_xml(input, self.indent, false),
            _ => unreachable!(),
        }
    }

    fn run_query(&self, input: &str, hint: Option<&str>) -> anyhow::Result<String> {
        if self.query_expr.trim().is_empty() {
            return Err(anyhow::anyhow!("Type a query expression first"));
        }
        let fmt = format::detect_format(input, self.current_file.as_deref(), hint)?;
        match fmt {
            "json" => query::query_json(input, &self.query_expr, false),
            "xml"  => query::query_xml(input, &self.query_expr, false),
            _ => unreachable!(),
        }
    }

    fn run_tree(&self, input: &str, hint: Option<&str>) -> anyhow::Result<String> {
        let fmt = format::detect_format(input, self.current_file.as_deref(), hint)?;
        match fmt {
            "json" => format::tree_json(input, true),
            "xml"  => format::tree_xml(input, true),
            _ => unreachable!(),
        }
    }

    // ── File I/O ──

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
                    self.status = format!("📂  Loaded: {}", name);
                    self.output_raw.clear();
                    self.output_html.clear();
                    self.output_is_error = false;
                }
                Err(e) => self.status = format!("✖  {}", e),
            }
        }
    }

    fn save_output(&mut self) {
        if self.output_raw.is_empty() || self.output_is_error {
            self.status = "⚠  Nothing to save".to_string();
            return;
        }
        let default = self.current_file.as_ref()
            .and_then(|p| std::path::Path::new(p).file_stem()?.to_str().map(|s| format!("{}.formatted", s)))
            .unwrap_or_else(|| "output.formatted".into());

        if let Some(path) = rfd::FileDialog::new().set_file_name(&default).save_file() {
            match std::fs::write(&path, &self.output_raw) {
                Ok(_)  => self.status = format!("💾  Saved: {}", path.display()),
                Err(e) => self.status = format!("✖  {}", e),
            }
        }
    }
}

// ─── ANSI → egui RichText converter ───

fn parse_ansi(text: &str) -> Vec<StyledSegment> {
    let mut segments: Vec<StyledSegment> = Vec::new();
    let mut buf = String::new();
    let mut chars = text.chars().peekable();
    let mut fg: Option<Color32> = None;

    let ansi_color = |code: u8| -> Option<Color32> {
        Some(match code {
            30 => Color32::from_rgb(60, 60, 70),
            31 => palette::RED,
            32 => palette::GREEN,
            33 => palette::PEACH,
            34 => palette::BLUE,
            35 => palette::MAUVE,
            36 => palette::SAPPHIRE,
            37 => palette::SUBTEXT0,
            90 => Color32::from_rgb(80, 80, 90),
            91 => Color32::from_rgb(255, 100, 120),
            92 => Color32::from_rgb(100, 220, 130),
            93 => Color32::from_rgb(255, 200, 100),
            94 => Color32::from_rgb(100, 160, 255),
            95 => palette::PINK,
            96 => palette::TEAL,
            97 => palette::TEXT,
            _  => return None,
        })
    };

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.next() == Some('[') {
            // Flush buffer
            if !buf.is_empty() {
                segments.push(StyledSegment { text: std::mem::take(&mut buf), color: fg.unwrap_or(palette::TEXT) });
            }
            // Parse escape code
            let mut code_str = String::new();
            while let Some(&c) = chars.peek() {
                if c == 'm' { chars.next(); break; }
                code_str.push(c); chars.next();
            }
            // Handle codes like "0;36", "1;34", "0"
            for part in code_str.split(';') {
                match part {
                    "0" => fg = None,      // reset
                    "1" => {}               // bold — skip for now
                    n   => {
                        if let Ok(num) = n.parse::<u8>() {
                            if let Some(c) = ansi_color(num) {
                                fg = Some(c);
                            }
                        }
                    }
                }
            }
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        segments.push(StyledSegment { text: buf, color: fg.unwrap_or(palette::TEXT) });
    }
    segments
}

// ─── UI ───

impl eframe::App for SpruceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_theme(ui);

        // ── Header ──
        egui::Panel::top("header").frame(Frame {
            fill: palette::MANTLE,
            inner_margin: Margin::symmetric(16, 10),
            ..Default::default()
        }).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🌲 Spruce").color(palette::ACCENT).size(22.0).strong());
                ui.label(RichText::new("v0.1.0").color(palette::OVERLAY0).size(11.0));
                ui.add_space(24.0);

                let modes = [
                    ("📝  Format", AppMode::Format),
                    ("🔍  Query",  AppMode::Query),
                    ("🌳  Tree",   AppMode::Tree),
                ];
                for (label, mode) in &modes {
                    let sel = self.mode == *mode;
                    let resp = ui.add(
                        egui::Button::new(RichText::new(*label).size(13.0).color(if sel { Color32::WHITE } else { palette::SUBTEXT0 }))
                            .fill(if sel { palette::GREEN.gamma_multiply(0.25) } else { palette::SURFACE0 })
                            .stroke(if sel { egui::Stroke::new(1.0, palette::GREEN.gamma_multiply(0.5)) } else { egui::Stroke::NONE })
                            .min_size(egui::vec2(98.0, 28.0)),
                    );
                    if resp.clicked() { self.mode = *mode; self.output_raw.clear(); self.output_html.clear(); }
                }
            });
        });

        // ── Body ──
        egui::CentralPanel::default().frame(Frame {
            fill: palette::BASE,
            inner_margin: Margin::symmetric(16, 10),
            ..Default::default()
        }).show(ui, |ui| {
            let avail_h = ui.available_height();

            // ── Input ──
            let input_h = (avail_h * 0.38).max(90.0);
            let input_frame = Frame {
                fill: palette::MANTLE,
                corner_radius: 6.0.into(),
                inner_margin: Margin::symmetric(8, 6),
                ..Default::default()
            };

            input_frame.show(ui, |ui| {
                ui.set_min_height(input_h - 8.0);
                egui::ScrollArea::vertical().id_salt("input").max_height(input_h - 8.0).show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::multiline(&mut self.input)
                            .font(FontId::monospace(14.0))
                            .desired_width(f32::INFINITY)
                            .text_color(palette::TEXT)
                            .hint_text("Paste JSON or XML here…"),
                    );
                });
            });

            ui.add_space(4.0);

            // ── Controls ──
            Frame { fill: palette::MANTLE, corner_radius: 6.0.into(), inner_margin: Margin::symmetric(10, 6), ..Default::default() }
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("📂  Open").size(12.5)).clicked() { self.load_file(); }

                        ui.separator();

                        ui.label(RichText::new("Type:").size(12.5).color(palette::SUBTEXT0));
                        egui::ComboBox::from_id_salt("t").width(70.0)
                            .selected_text(self.format_type.clone())
                            .show_ui(ui, |ui| {
                                for t in &["auto", "json", "xml"] {
                                    ui.selectable_value(&mut self.format_type, t.to_string(), *t);
                                }
                            });

                        if self.mode == AppMode::Format {
                            ui.separator();
                            ui.label(RichText::new("Indent:").size(12.5).color(palette::SUBTEXT0));
                            ui.add(egui::Slider::new(&mut self.indent, 0..=8).integer().text("spaces"));
                        }

                        if self.mode == AppMode::Query {
                            ui.separator();
                            ui.label(RichText::new("Expr:").size(12.5).color(palette::SUBTEXT0));
                            ui.add_sized(egui::vec2(200.0, 22.0),
                                TextEdit::singleline(&mut self.query_expr)
                                    .font(FontId::monospace(13.0))
                                    .text_color(palette::SAPPHIRE)
                                    .hint_text(".users[0].name"),
                            );
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(
                                egui::Button::new(RichText::new("▶  Run").size(13.5).color(Color32::BLACK))
                                    .fill(palette::GREEN)
                                    .min_size(egui::vec2(80.0, 28.0)),
                            ).clicked() { self.run_action(); }

                            if ui.button(RichText::new("🗑  Clear").size(12.0)).clicked() {
                                self.input.clear(); self.output_raw.clear(); self.output_html.clear();
                                self.status = "Cleared".to_string();
                            }
                        });
                    });
                });

            ui.add_space(4.0);

            // ── Divider ──
            ui.add(egui::Separator::default().spacing(4.0));

            // ── Output ──
            let output_h = ui.available_height().max(60.0) - 28.0;
            let output_frame = Frame {
                fill: Color32::from_rgb(18, 18, 22),
                corner_radius: 6.0.into(),
                inner_margin: Margin::symmetric(8, 6),
                ..Default::default()
            };

            output_frame.show(ui, |ui| {
                ui.set_min_height(output_h);
                if self.output_html.is_empty() {
                    // Plain text fallback
                    egui::ScrollArea::vertical().id_salt("output").max_height(output_h).show(ui, |ui| {
                        ui.add_sized(
                            ui.available_size(),
                            TextEdit::multiline(&mut self.output_raw)
                                .font(FontId::monospace(13.0))
                                .desired_width(f32::INFINITY)
                                .text_color(palette::SUBTEXT0),
                        );
                    });
                } else {
                    // Syntax-highlighted output using spans
                    egui::ScrollArea::vertical().id_salt("output").max_height(output_h).show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let mut row_start = true;
                        for seg in &self.output_html {
                            if row_start {
                                // indent guide
                                row_start = false;
                            }
                            for line in seg.text.split('\n') {
                                let label = egui::Label::new(
                                    RichText::new(line).color(seg.color).font(FontId::monospace(13.0))
                                ).selectable(true);
                                ui.add(label);
                            }
                            if seg.text.ends_with('\n') {
                                row_start = true;
                            }
                        }
                    });
                }
            });

            ui.add_space(6.0);

            // ── Status bar ──
            Frame { fill: palette::MANTLE, corner_radius: 4.0.into(), inner_margin: Margin::symmetric(12, 6), ..Default::default() }
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let sc = if self.status.starts_with('✖') || self.status.starts_with("⚠") {
                            palette::RED
                        } else if self.status.starts_with('✓') {
                            palette::GREEN
                        } else {
                            palette::SUBTEXT0
                        };
                        ui.label(RichText::new(&self.status).size(12.0).color(sc));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("💾  Save").size(11.5)).clicked() { self.save_output(); }
                            if ui.button(RichText::new("📋  Copy").size(11.5)).clicked() {
                                ui.ctx().copy_text(self.output_raw.clone());
                                self.status = "✓  Copied".to_string();
                            }
                        });
                    });
                });
        });
    }
}

fn apply_theme(ui: &mut egui::Ui) {
    set_dark_theme(ui.ctx());
}

fn set_dark_theme(ctx: &egui::Context) {
    // Force dark mode every frame — critical on macOS where initial theme may be light
    ctx.set_theme(egui::Theme::Dark);

    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = palette::BASE;
    visuals.panel_fill = palette::BASE;
    visuals.widgets.noninteractive.bg_fill = palette::SURFACE0;
    visuals.widgets.inactive.bg_fill = palette::SURFACE1;
    visuals.widgets.active.bg_fill = palette::SURFACE2;
    visuals.widgets.hovered.bg_fill = palette::SURFACE2;
    visuals.selection.bg_fill = palette::BLUE.gamma_multiply(0.3);
    visuals.selection.stroke.color = palette::BLUE;
    visuals.hyperlink_color = palette::SAPPHIRE;
    ctx.set_visuals_of(egui::Theme::Dark, visuals);

    let mut style = (*ctx.style_of(Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.button_padding = egui::vec2(10.0, 3.0);
    ctx.set_style_of(Theme::Dark, style);
}
