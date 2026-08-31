use eframe::egui;
use moviola::{
    media::{self, MediaKind},
    tools::{chop_video::ChopVideoConfig, trim_silence::SilenceTrimConfig},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Chop,
    Trim,
}
struct Gui {
    tool: Tool,
    input: Option<PathBuf>,
    output: String,
    max_duration: String,
    cut_frequency: String,
    threshold: String,
    min_silence: String,
    status: Arc<Mutex<String>>,
    running: bool,
}
impl Default for Gui {
    fn default() -> Self {
        Self {
            tool: Tool::Chop,
            input: None,
            output: String::new(),
            max_duration: String::new(),
            cut_frequency: String::new(),
            threshold: "-20".into(),
            min_silence: "250".into(),
            status: Arc::new(Mutex::new("Select a media file to begin.".into())),
            running: false,
        }
    }
}
impl Gui {
    fn set_status(&self, s: impl Into<String>) {
        *self.status.lock().unwrap() = s.into();
    }
    fn submit(&mut self) {
        let Some(input) = self.input.clone() else {
            self.set_status("Choose an input file first.");
            return;
        };
        let output = if self.output.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.output.trim()))
        };
        let status = self.status.clone();
        self.running = true;
        let tool = self.tool;
        let max = self.max_duration.clone();
        let cut = self.cut_frequency.clone();
        let threshold = self.threshold.clone();
        let silence = self.min_silence.clone();
        thread::spawn(move || {
            let result = match tool {
                Tool::Chop => {
                    let max: f64 = match max.trim().parse() {
                        Ok(v) if v > 0.0 => v,
                        _ => {
                            return *status.lock().unwrap() =
                                "Output duration must be a positive number.".into();
                        }
                    };
                    let cut: f64 = if cut.trim().is_empty() {
                        max / 10.0
                    } else {
                        match cut.trim().parse() {
                            Ok(v) if v > 0.0 => v,
                            _ => {
                                return *status.lock().unwrap() =
                                    "Cut frequency must be a positive number.".into();
                            }
                        }
                    };
                    let out = output.unwrap_or_else(|| {
                        media::default_output_path(&input, "chopped", MediaKind::Video)
                    });
                    moviola::tools::chop_video::run_with_config(&ChopVideoConfig {
                        input_path: input,
                        output_path: out,
                        max_duration_seconds: max,
                        cut_frequency_seconds: cut,
                    })
                }
                Tool::Trim => {
                    let threshold: f32 = match threshold.trim().parse() {
                        Ok(v) if v < 0.0 => v,
                        _ => {
                            return *status.lock().unwrap() =
                                "Silence threshold must be a negative number.".into();
                        }
                    };
                    let ms: f32 = match silence.trim().parse() {
                        Ok(v) if v > 0.0 => v,
                        _ => {
                            return *status.lock().unwrap() =
                                "Minimum silence must be positive milliseconds.".into();
                        }
                    };
                    let kind = match MediaKind::from_path(&input) {
                        Some(k) => k,
                        None => {
                            return *status.lock().unwrap() = "Unsupported media extension.".into();
                        }
                    };
                    let out = output.unwrap_or_else(|| {
                        media::default_output_path(&input, "silence_trimmed", kind)
                    });
                    moviola::tools::trim_silence::run_with_config(&SilenceTrimConfig {
                        input_path: input,
                        output_path: out,
                        threshold_db: threshold,
                        min_silence_duration_seconds: ms / 1000.0,
                        media_kind: kind,
                    })
                }
            };
            *status.lock().unwrap() = match result {
                Ok(()) => "Finished successfully.".into(),
                Err(e) => format!("Error: {e}"),
            };
        });
    }
}
impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for file in &ctx.input(|i| i.raw.dropped_files.clone()) {
            if let Some(path) = &file.path {
                self.input = Some(path.clone());
                self.set_status(format!("Selected: {}", path.display()));
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Moviola");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tool, Tool::Chop, "Chop video");
                ui.selectable_value(&mut self.tool, Tool::Trim, "Trim silence");
            });
            ui.separator();
            if ui.button("Import media…").clicked() {
                if let Some(p) = rfd::FileDialog::new().pick_file() {
                    self.input = Some(p);
                }
            }
            ui.label(
                self.input
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "Or drag and drop a file here".into()),
            );
            ui.horizontal(|ui| {
                ui.label("Output path (optional)");
                ui.text_edit_singleline(&mut self.output);
                if ui.button("Choose…").clicked() {
                    if let Some(p) = rfd::FileDialog::new()
                        .set_file_name("output.mp4")
                        .save_file()
                    {
                        self.output = p.display().to_string();
                    }
                }
            });
            if self.output.trim().is_empty() {
                ui.small("Leave blank to use the default output name next to the input.");
            }
            match self.tool {
                Tool::Chop => {
                    ui.horizontal(|ui| {
                        ui.label("Duration (seconds)");
                        ui.text_edit_singleline(&mut self.max_duration);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Cut frequency (optional)");
                        ui.text_edit_singleline(&mut self.cut_frequency);
                    });
                }
                Tool::Trim => {
                    ui.horizontal(|ui| {
                        ui.label("Silence threshold (dB)");
                        ui.text_edit_singleline(&mut self.threshold);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Minimum silence (ms)");
                        ui.text_edit_singleline(&mut self.min_silence);
                    });
                }
            }
            if ui.button("Submit").clicked() && !self.running {
                self.submit();
            }
            ui.separator();
            ui.label(self.status.lock().unwrap().as_str());
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Moviola",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(Gui::default()))),
    )
}
