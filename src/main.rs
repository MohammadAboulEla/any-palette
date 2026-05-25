#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use auto_palette::{Algorithm, ImageData, Palette, Theme};
use eframe::egui;
use egui::{Color32, ColorImage, TextureHandle, TextureOptions};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 680.0])
            .with_min_inner_size([720.0, 520.0])
            .with_title("any-palette"),
        ..Default::default()
    };
    eframe::run_native(
        "any-palette",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Default,
    Colorful,
    Vivid,
    Muted,
    Light,
    Dark,
}

impl ThemeChoice {
    const ALL: [ThemeChoice; 6] = [
        ThemeChoice::Default,
        ThemeChoice::Colorful,
        ThemeChoice::Vivid,
        ThemeChoice::Muted,
        ThemeChoice::Light,
        ThemeChoice::Dark,
    ];
    fn label(self) -> &'static str {
        match self {
            ThemeChoice::Default => "Default",
            ThemeChoice::Colorful => "Colorful",
            ThemeChoice::Vivid => "Vivid",
            ThemeChoice::Muted => "Muted",
            ThemeChoice::Light => "Light",
            ThemeChoice::Dark => "Dark",
        }
    }
    fn as_theme(self) -> Option<Theme> {
        match self {
            ThemeChoice::Default => None,
            ThemeChoice::Colorful => Some(Theme::Colorful),
            ThemeChoice::Vivid => Some(Theme::Vivid),
            ThemeChoice::Muted => Some(Theme::Muted),
            ThemeChoice::Light => Some(Theme::Light),
            ThemeChoice::Dark => Some(Theme::Dark),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlgorithmChoice {
    DBSCANpp,
    DBSCAN,
    KMeans,
}

impl AlgorithmChoice {
    const ALL: [AlgorithmChoice; 3] = [
        AlgorithmChoice::DBSCANpp,
        AlgorithmChoice::DBSCAN,
        AlgorithmChoice::KMeans,
    ];
    fn label(self) -> &'static str {
        match self {
            AlgorithmChoice::DBSCANpp => "DBSCAN++ (fast)",
            AlgorithmChoice::DBSCAN => "DBSCAN (accurate)",
            AlgorithmChoice::KMeans => "K-Means (fastest)",
        }
    }
    fn as_algorithm(self) -> Algorithm {
        match self {
            AlgorithmChoice::DBSCANpp => Algorithm::DBSCANpp,
            AlgorithmChoice::DBSCAN => Algorithm::DBSCAN,
            AlgorithmChoice::KMeans => Algorithm::KMeans,
        }
    }
}

impl Default for AlgorithmChoice {
    fn default() -> Self {
        AlgorithmChoice::DBSCANpp
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Downsample {
    Off,
    Px1024,
    Px1600,
    Px2048,
}

impl Downsample {
    const ALL: [Downsample; 4] = [
        Downsample::Px1024,
        Downsample::Px1600,
        Downsample::Px2048,
        Downsample::Off,
    ];
    fn label(self) -> &'static str {
        match self {
            Downsample::Off => "Off (full res)",
            Downsample::Px1024 => "1024px",
            Downsample::Px1600 => "1600px",
            Downsample::Px2048 => "2048px",
        }
    }
    fn max_edge(self) -> Option<u32> {
        match self {
            Downsample::Off => None,
            Downsample::Px1024 => Some(1024),
            Downsample::Px1600 => Some(1600),
            Downsample::Px2048 => Some(2048),
        }
    }
}

impl Default for Downsample {
    fn default() -> Self {
        Downsample::Px1600
    }
}

struct Swatch {
    hex: String,
    color: Color32,
    population: usize,
}

struct ExtractResult {
    source_path: Option<PathBuf>,
    preview: ColorImage,
    swatches: Vec<Swatch>,
}

enum Job {
    Done(ExtractResult),
    Err(String),
}

#[derive(Default)]
struct App {
    preview_tex: Option<TextureHandle>,
    source_label: String,
    swatches: Vec<Swatch>,
    theme: ThemeChoice,
    algorithm: AlgorithmChoice,
    downsample: Downsample,
    max_swatches: usize,
    busy: bool,
    error: Option<String>,
    copied_hex: Option<(String, f64)>,
    rx: Option<mpsc::Receiver<Job>>,
}

impl Default for ThemeChoice {
    fn default() -> Self {
        ThemeChoice::Default
    }
}

impl App {
    fn start_extract_from_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        let theme = self.theme;
        let algorithm = self.algorithm;
        let downsample = self.downsample;
        let max = self.max_swatches.max(1);
        self.busy = true;
        self.error = None;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let ctx_clone = ctx.clone();
        thread::spawn(move || {
            let res = extract_from_path(&path, theme, algorithm, downsample, max);
            let _ = tx.send(match res {
                Ok(r) => Job::Done(r),
                Err(e) => Job::Err(e),
            });
            ctx_clone.request_repaint();
        });
    }

    fn start_extract_from_bytes(&mut self, ctx: &egui::Context, bytes: Vec<u8>, name: String) {
        let theme = self.theme;
        let algorithm = self.algorithm;
        let downsample = self.downsample;
        let max = self.max_swatches.max(1);
        self.busy = true;
        self.error = None;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let ctx_clone = ctx.clone();
        thread::spawn(move || {
            let res = extract_from_bytes(&bytes, name, theme, algorithm, downsample, max);
            let _ = tx.send(match res {
                Ok(r) => Job::Done(r),
                Err(e) => Job::Err(e),
            });
            ctx_clone.request_repaint();
        });
    }

    fn poll_job(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.rx else { return };
        match rx.try_recv() {
            Ok(Job::Done(r)) => {
                self.preview_tex = Some(ctx.load_texture(
                    "preview",
                    r.preview,
                    TextureOptions::LINEAR,
                ));
                self.source_label = r
                    .source_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "dropped image".into());
                self.swatches = r.swatches;
                self.busy = false;
                self.rx = None;
            }
            Ok(Job::Err(e)) => {
                self.error = Some(e);
                self.busy = false;
                self.rx = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.busy = false;
                self.rx = None;
            }
        }
    }
}

fn extract_from_path(
    path: &std::path::Path,
    theme: ThemeChoice,
    algorithm: AlgorithmChoice,
    downsample: Downsample,
    max: usize,
) -> Result<ExtractResult, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {e}"))?;
    let preview = to_color_image(&img);
    let swatches = extract_from_dynamic(&img, theme, algorithm, downsample, max)?;
    Ok(ExtractResult {
        source_path: Some(path.to_path_buf()),
        preview,
        swatches,
    })
}

fn extract_from_bytes(
    bytes: &[u8],
    _name: String,
    theme: ThemeChoice,
    algorithm: AlgorithmChoice,
    downsample: Downsample,
    max: usize,
) -> Result<ExtractResult, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Decode failed: {e}"))?;
    let preview = to_color_image(&img);
    let swatches = extract_from_dynamic(&img, theme, algorithm, downsample, max)?;
    Ok(ExtractResult {
        source_path: None,
        preview,
        swatches,
    })
}

fn extract_from_dynamic(
    img: &image::DynamicImage,
    theme: ThemeChoice,
    algorithm: AlgorithmChoice,
    downsample: Downsample,
    max: usize,
) -> Result<Vec<Swatch>, String> {
    let (w, h) = (img.width(), img.height());
    let scaled = match downsample.max_edge() {
        Some(limit) if w.max(h) > limit => {
            let scale = limit as f32 / w.max(h) as f32;
            img.thumbnail((w as f32 * scale) as u32, (h as f32 * scale) as u32)
        }
        _ => img.clone(),
    };
    let rgba = scaled.to_rgba8();
    let (sw, sh) = rgba.dimensions();
    let image_data = ImageData::new(sw, sh, rgba.as_raw())
        .map_err(|e| format!("auto-palette ImageData failed: {e:?}"))?;
    run_palette(&image_data, theme, algorithm, max)
}

fn run_palette(
    image_data: &ImageData,
    theme: ThemeChoice,
    algorithm: AlgorithmChoice,
    max: usize,
) -> Result<Vec<Swatch>, String> {
    let palette: Palette<f64> = Palette::builder()
        .algorithm(algorithm.as_algorithm())
        .build(image_data)
        .map_err(|e| format!("Palette build failed: {e:?}"))?;
    let raw = match theme.as_theme() {
        None => palette
            .find_swatches(max)
            .map_err(|e| format!("find_swatches failed: {e:?}"))?,
        Some(t) => palette
            .find_swatches_with_theme(max, t)
            .map_err(|e| format!("find_swatches_with_theme failed: {e:?}"))?,
    };
    Ok(raw
        .into_iter()
        .map(|sw| {
            let hex = sw.color().to_hex_string();
            let rgb = sw.color().to_rgb();
            Swatch {
                color: Color32::from_rgb(rgb.r, rgb.g, rgb.b),
                hex,
                population: sw.population(),
            }
        })
        .collect())
}

fn to_color_image(img: &image::DynamicImage) -> ColorImage {
    let max_dim = 1200u32;
    let (w, h) = (img.width(), img.height());
    let scaled = if w.max(h) > max_dim {
        let scale = max_dim as f32 / w.max(h) as f32;
        img.thumbnail((w as f32 * scale) as u32, (h as f32 * scale) as u32)
    } else {
        img.clone()
    };
    let rgba = scaled.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    ColorImage::from_rgba_unmultiplied(size, rgba.as_raw())
}

fn copy_to_clipboard(text: &str) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text.to_string()))
        .is_ok()
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.max_swatches == 0 {
            self.max_swatches = 8;
        }
        self.poll_job(ctx);

        // Drag & drop
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(file) = dropped.into_iter().next() {
            if let Some(path) = file.path {
                self.start_extract_from_path(ctx, path);
            } else if let Some(bytes) = file.bytes {
                self.start_extract_from_bytes(ctx, bytes.to_vec(), file.name);
            }
        }

        // Hover overlay while dragging
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("any-palette");
                ui.separator();
                if ui.button("Open image…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
                        .pick_file()
                    {
                        self.start_extract_from_path(ctx, path);
                    }
                }
                ui.separator();
                ui.label("Theme:");
                egui::ComboBox::from_id_salt("theme")
                    .selected_text(self.theme.label())
                    .show_ui(ui, |ui| {
                        for t in ThemeChoice::ALL {
                            ui.selectable_value(&mut self.theme, t, t.label());
                        }
                    });
                ui.separator();
                ui.label("Algorithm:");
                egui::ComboBox::from_id_salt("algorithm")
                    .selected_text(self.algorithm.label())
                    .show_ui(ui, |ui| {
                        for a in AlgorithmChoice::ALL {
                            ui.selectable_value(&mut self.algorithm, a, a.label());
                        }
                    });
                ui.separator();
                ui.label("Downsample:");
                egui::ComboBox::from_id_salt("downsample")
                    .selected_text(self.downsample.label())
                    .show_ui(ui, |ui| {
                        for d in Downsample::ALL {
                            ui.selectable_value(&mut self.downsample, d, d.label());
                        }
                    });
                ui.separator();
                ui.label("Swatches:");
                ui.add(egui::DragValue::new(&mut self.max_swatches).range(1..=32));
                if ui.button("Re-extract").clicked() && !self.busy {
                    if let Some(tex) = &self.preview_tex {
                        let _ = tex;
                    }
                }
                if self.busy {
                    ui.separator();
                    ui.spinner();
                    ui.label("Extracting…");
                }
                if !self.source_label.is_empty() {
                    ui.separator();
                    ui.weak(&self.source_label);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = &self.error {
                ui.colored_label(Color32::LIGHT_RED, err);
                ui.separator();
            }

            ui.columns(2, |cols| {
                // Left: preview / drop zone
                cols[0].vertical(|ui| {
                    ui.label("Image");
                    let avail = ui.available_size();
                    let (rect, _resp) =
                        ui.allocate_exact_size(avail, egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 6.0, Color32::from_gray(28));
                    if let Some(tex) = &self.preview_tex {
                        let img_size = tex.size_vec2();
                        let scale =
                            (rect.width() / img_size.x).min(rect.height() / img_size.y).min(1.0);
                        let draw = img_size * scale;
                        let center = rect.center();
                        let r = egui::Rect::from_center_size(center, draw);
                        painter.image(
                            tex.id(),
                            r,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Drop an image here or click \"Open image…\"",
                            egui::FontId::proportional(18.0),
                            Color32::from_gray(160),
                        );
                    }
                    if hovering {
                        painter.rect_stroke(
                            rect,
                            6.0,
                            egui::Stroke::new(2.0, Color32::LIGHT_BLUE),
                        );
                    }
                });

                // Right: swatches
                cols[1].vertical(|ui| {
                    ui.label("Swatches");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.swatches.is_empty() {
                            ui.weak("No swatches yet.");
                            return;
                        }
                        let swatches = std::mem::take(&mut self.swatches);
                        for sw in &swatches {
                            self.draw_swatch_row(ui, sw);
                        }
                        self.swatches = swatches;
                    });
                });
            });
        });

        // Fade copy toast
        if let Some((_, t)) = &mut self.copied_hex {
            *t -= ctx.input(|i| i.unstable_dt) as f64;
        }
        if let Some((_, t)) = &self.copied_hex {
            if *t <= 0.0 {
                self.copied_hex = None;
            }
        }
        if let Some((hex, _)) = &self.copied_hex {
            egui::Area::new(egui::Id::new("toast"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(format!("Copied {hex}"));
                    });
                });
            ctx.request_repaint();
        }

        if self.busy {
            ctx.request_repaint();
        }
    }
}

impl App {
    fn draw_swatch_row(&mut self, ui: &mut egui::Ui, sw: &Swatch) {
        let row_h = 44.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_h),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        let swatch_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(row_h, row_h)).shrink(2.0);
        painter.rect_filled(swatch_rect, 4.0, sw.color);
        painter.rect_stroke(
            swatch_rect,
            4.0,
            egui::Stroke::new(1.0, Color32::from_gray(60)),
        );

        let text_x = swatch_rect.right() + 10.0;
        let hex_pos = egui::pos2(text_x, rect.center().y - 8.0);
        painter.text(
            hex_pos,
            egui::Align2::LEFT_CENTER,
            sw.hex.to_uppercase(),
            egui::FontId::monospace(15.0),
            Color32::WHITE,
        );
        let pop_pos = egui::pos2(text_x, rect.center().y + 10.0);
        painter.text(
            pop_pos,
            egui::Align2::LEFT_CENTER,
            format!("pop {}", sw.population),
            egui::FontId::proportional(11.0),
            Color32::from_gray(150),
        );

        let btn_size = egui::vec2(70.0, 26.0);
        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(rect.right() - btn_size.x - 6.0, rect.center().y - btn_size.y / 2.0),
            btn_size,
        );
        let btn_resp = ui.interact(
            btn_rect,
            ui.id().with(&sw.hex),
            egui::Sense::click(),
        );
        let bg = if btn_resp.hovered() {
            Color32::from_gray(70)
        } else {
            Color32::from_gray(50)
        };
        painter.rect_filled(btn_rect, 4.0, bg);
        painter.text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Copy",
            egui::FontId::proportional(13.0),
            Color32::WHITE,
        );
        if btn_resp.clicked() {
            let hex = sw.hex.to_uppercase();
            if copy_to_clipboard(&hex) {
                self.copied_hex = Some((hex, 1.6));
            }
        }
    }
}
