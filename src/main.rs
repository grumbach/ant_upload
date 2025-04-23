// Mock server implementation
mod server {
    pub async fn put_data(bytes: &[u8]) -> Result<String, String> {
        println!("Uploading {} bytes...", bytes.len());
        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        println!("Upload complete!");
        // Err("test error".to_string())
        Ok("Upload successful".to_string())
    }
}

use eframe::egui;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct UploadStatus {
    filename: String,
    start_time: std::time::Instant,
    completed: bool,
    success: Option<bool>,
    message: String,
    time_to_complete: Option<f32>,  // Store completion time when finished
}

// Define the status update event
enum UploadEvent {
    Complete {
        index: usize,
        address: String,
        duration: std::time::Duration,
        filename: String,
    },
    Failed {
        index: usize,
        filename: String,
        duration: std::time::Duration,
        error: String,
    },
}

struct UploadApp {
    dropped_files: Vec<egui::DroppedFile>,
    upload_statuses: Vec<UploadStatus>,  // Track multiple uploads
    status_receiver: mpsc::UnboundedReceiver<UploadEvent>,
    status_sender: mpsc::UnboundedSender<UploadEvent>,
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
                UploadEvent::Complete { index, address, duration, filename } => {
                    if let Some(status) = self.upload_statuses.get_mut(index) {
                        status.completed = true;
                        status.success = Some(true);
                        status.time_to_complete = Some(duration.as_secs_f32());
                        status.message = format!(
                            "{} was successfully uploaded in {} seconds! At address: {}", 
                            filename, 
                            duration.as_secs_f32(),
                            address
                        );
                    }
                },
                UploadEvent::Failed { index, filename, duration, error } => {
                    if let Some(status) = self.upload_statuses.get_mut(index) {
                        status.completed = true;
                        status.success = Some(false);
                        status.time_to_complete = Some(duration.as_secs_f32());
                        status.message = format!("Failed to upload {}: {}", filename, error);
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate half screen height
            let available_height = ui.available_height();
            let half_height = available_height / 2.0;

            // Top half - Drop zone with arrow
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Calculate the screen dimensions and desired image size
                let screen_rect = ui.available_rect_before_wrap();
                let size = screen_rect.size() * 0.5;
                
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

                ui.label(egui::RichText::new("Drop files here to upload").size(24.0));

                // Preview dropped files
                for file in &self.dropped_files {
                    if let Some(path) = &file.path {
                        ui.label(format!("📁 {}", path.display()));
                    } else {
                        ui.label(format!("📁 {}", file.name));
                    }
                }
            });

            ui.add_space(10.0);

            // Bottom half - Scrollable status area
            egui::ScrollArea::vertical()
                .max_height(half_height)
                .show(ui, |ui| {
                    // Display upload statuses
                    for status in &self.upload_statuses {
                        let color = if !status.completed {
                            egui::Color32::YELLOW
                        } else if status.success.unwrap_or(false) {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };
                        
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true), |ui| {
                                ui.set_max_width(ui.available_width() - 30.0);
                                
                                // Filename in grey
                                ui.label(egui::RichText::new(format!("📁 {}", status.filename)).color(egui::Color32::from_gray(180)));
                                
                                // Status text and duration
                                ui.label(egui::RichText::new(
                                    if !status.completed {
                                        " uploading... "
                                    } else {
                                        " uploaded in "
                                    }
                                ).color(
                                    if !status.completed {
                                        egui::Color32::YELLOW
                                    } else {
                                        egui::Color32::from_gray(180)
                                    }
                                ));

                                // Duration in green
                                let duration = if let Some(time) = status.time_to_complete {
                                    time  // Use stored completion time
                                } else {
                                    status.start_time.elapsed().as_secs_f32()  // Live counter
                                };
                                ui.label(egui::RichText::new(format!("{:.2}", duration))
                                    .color(egui::Color32::from_rgb(100, 200, 100)));
                                
                                // Only show address for completed successful uploads
                                if status.completed && status.success == Some(true) {
                                    // "seconds! At address:" in grey
                                    ui.label(egui::RichText::new(" seconds! At address: ").color(egui::Color32::from_gray(180)));
                                    
                                    // Address in purple, make sure to match the case in the status message
                                    if let Some(address) = status.message.split("At address: ").nth(1) {
                                        ui.label(egui::RichText::new(address.trim())
                                            .color(egui::Color32::from_rgb(180, 120, 255)));
                                    }
                                } else if status.success == Some(false) {
                                    ui.label(egui::RichText::new(format!("Failed to upload: {}", status.message)).color(egui::Color32::RED));
                                } else {
                                    // Just "seconds..." for in-progress or failed uploads
                                    ui.label(egui::RichText::new(" seconds...").color(egui::Color32::from_gray(180)));
                                }
                            });

                            // Only show copy button for completed successful uploads with an address
                            if status.completed && status.success == Some(true) {
                                if let Some(address) = status.message.split("At address: ").nth(1) {
                                    if !address.trim().is_empty() && ui.small_button("📋").clicked() {
                                        ui.output_mut(|o| o.copied_text = address.trim().to_string());
                                    }
                                }
                            }
                        });
                    }
                });
        });

        // Handle file drops
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();

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
                                    time_to_complete: None,
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
                                                address: "81a54669727374939400dc0020ccf8cc9d5ecc87cce9cc94ccbfcc87ccc5ccceccea5cccb501ccddcc9221cca46acc8b6bccc0cc93cc84cccd41ccaf27cccbcca9ccfeccacdc002054cc902f3941cccf0561cce37cccdbcca742ccaecc9cccddcc9635cc95ccb1cc977fccb4ccb9ccf80b13ccd757ccf934cc86cd10769401dc0020ccf1742dccc5515140cce9cce816cc85ccc2cce552cc8d244ecc9872cc84cceb1e67ccdecca3675d5c0b57ccdd4adc0020cc9428ccb5cce7cc87ccc0cca016ccd4ccd155ccf24f6acced1acca44c4e41ccd5595119cca6cc9069ccc9ccf3ccd171ccd0cd10769402dc0020cc84451e5d7bccf72e76766d3f5e690accef5942cc91010ecc89cc94ccab0e3a73cce229ccf70c5eccc3dc0020cca9ccb9ccf44dccd1ccfeccbc58cc8c3eccd9ccc108165acce4ccf525ccc9cc8126ccd8116955cccdccb0ccc3ccd837cc85ccabcd1077".to_string(),
                                                duration: start_time.elapsed(),
                                                filename: filename_clone,
                                            });
                                        },
                                        Err(e) => {
                                            let _ = status_sender.send(UploadEvent::Failed {
                                                index: status_index,
                                                filename: filename_clone,
                                                duration: start_time.elapsed(),
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