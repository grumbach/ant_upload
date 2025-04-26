mod server;

use server::DEFAULT_ENVIRONMENT;
use server::ENVIRONMENTS;
use server::Server;

use eframe::egui;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct UploadStatus {
    filename: String,
    start_time: std::time::Instant,
    completed: bool,
    success: Option<bool>,
    message: String,
    time_to_complete: Option<f32>, // Store completion time when finished
}

// Define the status update event
enum UploadEvent {
    Complete {
        index: usize,
        address: String,
        cost: String,
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
    server: Option<Server>,
    dropped_files: Vec<egui::DroppedFile>,
    upload_statuses: Vec<UploadStatus>, // Track multiple uploads
    status_receiver: mpsc::UnboundedReceiver<UploadEvent>,
    status_sender: mpsc::UnboundedSender<UploadEvent>,
    passcode: String,
    passcode_confirmed: bool,
    selected_env: String,
    error_message: Option<String>,
    server_init_receiver: Option<mpsc::UnboundedReceiver<Result<Server, String>>>,
    is_connecting: bool,
}

impl Default for UploadApp {
    fn default() -> Self {
        let (status_sender, status_receiver) = mpsc::unbounded_channel();
        Self {
            server: None,
            dropped_files: Vec::new(),
            upload_statuses: Vec::new(),
            status_receiver,
            status_sender,
            passcode: String::new(),
            passcode_confirmed: false,
            selected_env: DEFAULT_ENVIRONMENT.to_string(),
            error_message: None,
            server_init_receiver: None,
            is_connecting: false,
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
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
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
                                ui.horizontal(|ui| {
                                    ui.colored_label(egui::Color32::from_rgb(220, 50, 50), error);
                                    if ui.small_button("📋").clicked() {
                                        ui.output_mut(|o| o.copied_text = error.clone());
                                    }
                                });
                            }

                            // Show loading spinner while connecting
                            if self.is_connecting {
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(
                                        egui::RichText::new(" Connecting...")
                                            .color(egui::Color32::YELLOW),
                                    );
                                });
                            }

                            // Submit button
                            ui.add_space(20.0);
                            let button = ui.add_sized(
                                [120.0, 40.0],
                                egui::Button::new(egui::RichText::new("Submit").size(20.0)),
                            );
                            let submit_clicked = button.clicked();
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                            if submit_clicked || enter_pressed {
                                // Start connection attempt
                                self.is_connecting = true;
                                let (tx, rx) = mpsc::unbounded_channel();
                                self.server_init_receiver = Some(rx);

                                let passcode = self.passcode.clone();
                                let env = self.selected_env.clone();

                                // Spawn async task for server initialization
                                tokio::spawn(async move {
                                    let result = Server::new(&passcode, &env).await;
                                    let _ = tx.send(result);
                                });
                            }

                            // help message
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Please enter a valid EVM hex encoded private key. Your Secret Key is used solely for local transaction signing and is NEVER stored, transmitted, or persisted. The key exists ONLY in memory until the app is closed. You must own ANT and some ETH to upload data. Stop reading and start uploading!").size(10.0));

                            // Check for server init result
                            if let Some(receiver) = &mut self.server_init_receiver {
                                if let Ok(result) = receiver.try_recv() {
                                    self.is_connecting = false;
                                    self.server_init_receiver = None;

                                    match result {
                                        Ok(server) => {
                                            self.server = Some(server);
                                            self.passcode_confirmed = true;
                                            self.error_message = None;
                                        }
                                        Err(error) => {
                                            self.error_message = Some(error);
                                            self.passcode.clear();
                                        }
                                    }
                                }
                            }
                        },
                    );
                });

            // Environment selector window (always visible)
            egui::Window::new("env_selector")
                .frame(egui::Frame::none())
                .fixed_pos(egui::pos2(
                    ctx.available_rect().right() - 100.0,
                    ctx.available_rect().bottom() - 80.0,
                ))
                .title_bar(false)
                .show(ctx, |ui| {
                    let environments = ENVIRONMENTS;
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
                UploadEvent::Complete {
                    index,
                    address,
                    cost,
                    duration,
                    filename,
                } => {
                    if let Some(status) = self.upload_statuses.get_mut(index) {
                        status.completed = true;
                        status.success = Some(true);
                        status.time_to_complete = Some(duration.as_secs_f32());
                        status.message = format!(
                            "{filename} was successfully uploaded in {} seconds for {cost}! At address: {address}",
                            duration.as_secs_f32(),
                        );
                    }
                }
                UploadEvent::Failed {
                    index,
                    filename,
                    duration,
                    error,
                } => {
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

                ui.add_space(10.0);
                // Show error message if any
                if let Some(error) = &self.error_message {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(220, 50, 50), error);
                        if ui.small_button("📋").clicked() {
                            ui.output_mut(|o| o.copied_text = error.clone());
                        }
                    });
                }
            });

            // Bottom half - Scrollable status area
            egui::ScrollArea::vertical()
                .max_height(half_height)
                .show(ui, |ui| {
                    // Display upload statuses
                    for status in &self.upload_statuses {
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center)
                                    .with_main_wrap(true),
                                |ui| {
                                    ui.set_max_width(ui.available_width() - 30.0);

                                    // Filename in grey
                                    ui.label(
                                        egui::RichText::new(format!("📁 {}", status.filename))
                                            .color(egui::Color32::from_gray(180)),
                                    );

                                    // Status text and duration
                                    ui.label(
                                        egui::RichText::new(if !status.completed {
                                            " uploading... "
                                        } else {
                                            " uploaded in "
                                        })
                                        .color(
                                            if !status.completed {
                                                egui::Color32::YELLOW
                                            } else {
                                                egui::Color32::from_gray(180)
                                            },
                                        ),
                                    );

                                    // Duration in green
                                    let duration = if let Some(time) = status.time_to_complete {
                                        time // Use stored completion time
                                    } else {
                                        status.start_time.elapsed().as_secs_f32() // Live counter
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("{:.2}", duration))
                                            .color(egui::Color32::from_rgb(100, 200, 100)),
                                    );

                                    // Only show address for completed successful uploads
                                    if status.completed && status.success == Some(true) {
                                        // "seconds! At address:" in grey
                                        ui.label(
                                            egui::RichText::new(" seconds! At address: ")
                                                .color(egui::Color32::from_gray(180)),
                                        );

                                        // Address in purple, make sure to match the case in the status message
                                        if let Some(address) =
                                            status.message.split("At address: ").nth(1)
                                        {
                                            ui.label(
                                                egui::RichText::new(address.trim())
                                                    .color(egui::Color32::from_rgb(180, 120, 255)),
                                            );
                                        }
                                    } else if status.success == Some(false) {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Failed to upload: {}",
                                                status.message
                                            ))
                                            .color(egui::Color32::RED),
                                        );
                                    } else {
                                        // Just "seconds..." for in-progress or failed uploads
                                        ui.label(
                                            egui::RichText::new(" seconds...")
                                                .color(egui::Color32::from_gray(180)),
                                        );
                                    }
                                },
                            );

                            // Only show copy button for completed successful uploads with an address
                            if status.completed && status.success == Some(true) {
                                if let Some(address) = status.message.split("At address: ").nth(1) {
                                    if !address.trim().is_empty() && ui.small_button("📋").clicked()
                                    {
                                        ui.output_mut(|o| {
                                            o.copied_text = address.trim().to_string()
                                        });
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
                                let filename = path
                                    .file_name()
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

                                let server_clone = self.server.as_ref().unwrap().clone();
                                // Spawn async upload task
                                tokio::spawn(async move {
                                    let start_time = std::time::Instant::now();
                                    match server_clone.put_data(&bytes).await {
                                        Ok((address, cost)) => {
                                            let _ = status_sender.send(UploadEvent::Complete {
                                                index: status_index,
                                                address: address.clone(),
                                                cost: cost.clone(),
                                                duration: start_time.elapsed(),
                                                filename: filename_clone,
                                            });
                                        }
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
                                println!("Error reading file {}: {e}", path.display());
                                self.error_message =
                                    Some(format!("Error reading file {}: {e}", path.display()));
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
        "Ant Upload",
        options,
        Box::new(|_cc| Box::new(UploadApp::default())),
    )
}
