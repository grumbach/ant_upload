// Mock server implementation
mod server {
    pub async fn put_data(bytes: &[u8]) -> Result<String, String> {
        println!("Uploading {} bytes...", bytes.len());
        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
        println!("Upload complete!");
        Ok("Upload successful".to_string())
    }
}

use eframe::egui;
use std::fs;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct UploadStatus {
    filename: String,
    start_time: std::time::Instant,
    completed: bool,
    success: Option<bool>,
    message: String,
}

// Define the status update event
enum UploadEvent {
    Complete {
        index: usize,
        duration: std::time::Duration,
        filename: String,
    },
    Failed {
        index: usize,
        filename: String,
        error: String,
    },
}

struct UploadApp {
    dropped_files: Vec<egui::DroppedFile>,
    upload_statuses: Vec<UploadStatus>,  // Track multiple uploads
    status_receiver: mpsc::UnboundedReceiver<UploadEvent>,
    status_sender: mpsc::UnboundedSender<UploadEvent>,
    status_message: String,
    passcode: String,
    passcode_confirmed: bool,
    selected_env: String,
    error_message: Option<String>,
}

impl Default for UploadApp {
    fn default() -> Self {
        let (status_sender, status_receiver) = mpsc::unbounded_channel();
        Self {
            dropped_files: Vec::new(),
            upload_statuses: Vec::new(),
            status_receiver,
            status_sender,
            status_message: "Drop files here to upload".to_string(),
            passcode: String::new(),
            passcode_confirmed: false,
            selected_env: "local".to_string(),
            error_message: None,
        }
    }
}

pub fn verify_passcode(passcode: &str) -> Result<String, String> {
    if passcode == "no" {
        Err("Invalid passcode".to_string())
    } else {
        Ok(format!("Hash of {}", passcode))
    }
}

impl UploadApp {
    // Add a helper method to check if there are active uploads
    fn has_active_uploads(&self) -> bool {
        self.upload_statuses.iter().any(|status| !status.completed)
    }
}

impl eframe::App for UploadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints while uploads are active
        if self.has_active_uploads() {
            ctx.request_repaint();
        }

        // Show passcode modal if not yet confirmed
        if !self.passcode_confirmed {
            // Main password window
            egui::Window::new("Enter SECRET_KEY")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        ui.add_space(20.0);

                        let text_edit = egui::TextEdit::singleline(&mut self.passcode)
                            .password(true)
                            .font(egui::TextStyle::Heading)
                            .desired_width(200.0);
                        let response = ui.add(text_edit);
                        response.request_focus();

                        // Show error message if any
                        if let Some(error) = &self.error_message {
                            ui.add_space(8.0);
                            ui.colored_label(egui::Color32::from_rgb(220, 50, 50), error);
                        }

                        ui.add_space(20.0);

                        let button = ui.add_sized(
                            [120.0, 40.0],
                            egui::Button::new(egui::RichText::new("Submit").size(20.0))
                        );

                        let submit_clicked = button.clicked();
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        
                        if submit_clicked || enter_pressed {
                            match verify_passcode(&self.passcode) {
                                Ok(verified_passcode) => {
                                    self.passcode = verified_passcode;
                                    self.passcode_confirmed = true;
                                    self.error_message = None;
                                }
                                Err(error) => {
                                    self.error_message = Some(error);
                                    self.passcode.clear();
                                }
                            }
                        }
                    });
                });

            // Environment selector window (always visible)
            egui::Window::new("env_selector")
                .frame(egui::Frame::none())
                .fixed_pos(egui::pos2(ctx.available_rect().right() - 100.0, ctx.available_rect().bottom() - 80.0))
                .title_bar(false)
                .show(ctx, |ui| {
                    let environments = ["local", "alpha", "autonomi"];
                    for env in environments.iter().rev() {
                        let is_selected = self.selected_env == *env;
                        if ui.selectable_label(is_selected, *env).clicked() {
                            self.selected_env = env.to_string();
                            println!("Selected environment: {}", env);
                        }
                    }
                });
            return; // Don't show main UI until passcode is confirmed
        }

        // Process any completed uploads
        while let Ok(event) = self.status_receiver.try_recv() {
            match event {
                UploadEvent::Complete { index, duration, filename } => {
                    if let Some(status) = self.upload_statuses.get_mut(index) {
                        status.completed = true;
                        status.success = Some(true);
                        status.message = format!(
                            "{} was successfully uploaded in {:.2} seconds!", 
                            filename, 
                            duration.as_secs_f32()
                        );
                    }
                },
                UploadEvent::Failed { index, filename, error } => {
                    if let Some(status) = self.upload_statuses.get_mut(index) {
                        status.completed = true;
                        status.success = Some(false);
                        status.message = format!("Failed to upload {}: {}", filename, error);
                    }
                }
            }
        }

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    // Calculate the screen dimensions and desired image size
                    let screen_rect = ui.available_rect_before_wrap();
                    let size = screen_rect.size() * 0.5; // Half screen size
                    
                    // Center the image by adding space before it
                    let indent = (screen_rect.width() - size.x) * 0.5;
                    ui.add_space(indent);

                    // Draw the upload icon using egui's painting primitives
                    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let painter = ui.painter();
                    
                    // Draw arrow up icon
                    let stroke = egui::Stroke::new(2.0, ui.style().visuals.text_color());
                    let arrow_size = size.x.min(size.y) * 0.5;
                    let center = rect.center();
                    
                    // Arrow shaft
                    painter.line_segment(
                        [
                            egui::pos2(center.x, center.y + arrow_size * 0.5),
                            egui::pos2(center.x, center.y - arrow_size * 0.5),
                        ],
                        stroke,
                    );
                    
                    // Arrow head
                    painter.line_segment(
                        [
                            egui::pos2(center.x - arrow_size * 0.3, center.y - arrow_size * 0.2),
                            egui::pos2(center.x, center.y - arrow_size * 0.5),
                        ],
                        stroke,
                    );
                    painter.line_segment(
                        [
                            egui::pos2(center.x + arrow_size * 0.3, center.y - arrow_size * 0.2),
                            egui::pos2(center.x, center.y - arrow_size * 0.5),
                        ],
                        stroke,
                    );

                    // Remove the group and use a larger label
                    ui.label(egui::RichText::new(&self.status_message).size(24.0));
                    
                    // Preview dropped files
                    for file in &self.dropped_files {
                        if let Some(path) = &file.path {
                            ui.label(format!("📁 {}", path.display()));
                        } else {
                            ui.label(format!("📁 {}", file.name));
                        }
                    }

                    // Display upload statuses
                    for status in &self.upload_statuses {
                        let color = if !status.completed {
                            egui::Color32::YELLOW
                        } else if status.success.unwrap_or(false) {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };
                        ui.colored_label(color, &status.message);
                    }
                });

                // Handle file drops
                ctx.input(|i| {
                    if !i.raw.dropped_files.is_empty() {
                        self.dropped_files = i.raw.dropped_files.clone();
                        self.status_message = "Processing files...".to_string();
                        
                        // Process each dropped file
                        for file in &self.dropped_files {
                            if let Some(path) = &file.path {
                                match std::fs::read(path) {
                                    Ok(bytes) => {
                                        let filename = path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("unknown")
                                            .to_string();
                                        
                                        // Create new upload status
                                        self.upload_statuses.push(UploadStatus {
                                            filename: filename.clone(),
                                            start_time: std::time::Instant::now(),
                                            completed: false,
                                            success: None,
                                            message: format!("Uploading {}...", filename),
                                        });

                                        let status_index = self.upload_statuses.len() - 1;
                                        let status_sender = self.status_sender.clone();
                                        let filename_clone = filename.clone();
                                        
                                        // Spawn async upload task
                                        tokio::spawn(async move {
                                            let start_time = std::time::Instant::now();
                                            match server::put_data(&bytes).await {
                                                Ok(_) => {
                                                    let _ = status_sender.send(UploadEvent::Complete {
                                                        index: status_index,
                                                        duration: start_time.elapsed(),
                                                        filename: filename_clone,
                                                    });
                                                },
                                                Err(e) => {
                                                    let _ = status_sender.send(UploadEvent::Failed {
                                                        index: status_index,
                                                        filename: filename_clone,
                                                        error: e.to_string(),
                                                    });
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        println!("Error reading file: {}", e);
                                    }
                                }
                            }
                        }
                    }
                });
            });
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "File Uploader",
        options,
        Box::new(|_cc| Box::new(UploadApp::default())),
    )
}