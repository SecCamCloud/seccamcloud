// ============================================================================
// SecCamCloud - GUI Application
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Sender},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::Local;
use clap::Parser;
use eframe::egui;

use seccamcloud::{
    setup_logging, load_points, save_points, ClickPoint, AutomationThread,
    AutomationMessage, is_windows, key_pressed, APP_TITLE, APP_VERSION,
    Telemetry, ScreenshotManager,
};

// ============================================================================
// CLI ARGUMENTS
// ============================================================================

#[derive(Parser)]
#[command(name = APP_TITLE)]
#[command(version = APP_VERSION)]
#[command(about = "Advanced automation tool with GUI, telemetry, and monitoring")]
struct CliArgs {
    /// Enable dry-run mode (simulation without actual clicks)
    #[arg(long, short = 'd')]
    dry_run: bool,

    /// Enable telemetry event logging
    #[arg(long, short = 't')]
    telemetry: bool,

    /// Enable screenshot capture (requires screenshots feature)
    #[arg(long, short = 's')]
    screenshots: bool,
}

// ============================================================================
// HOTKEY MONITOR
// ============================================================================

struct HotkeyMonitor {
    emergency_key: i32,
    last_state: Mutex<bool>,
}

impl HotkeyMonitor {
    fn new() -> Self {
        const VK_DELETE: i32 = 0x2E;
        Self {
            emergency_key: VK_DELETE,
            last_state: Mutex::new(false),
        }
    }

    fn check_emergency_stop(&self) -> bool {
        if !is_windows() {
            return false;
        }

        let pressed = key_pressed(self.emergency_key);
        let mut last = self.last_state.lock().unwrap();

        let triggered = pressed && !*last;
        *last = pressed;

        triggered
    }
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

struct AppState {
    // Thread management
    automation_thread: Option<JoinHandle<()>>,
    stop_sender: Option<Sender<()>>,
    message_receiver: Arc<Mutex<mpsc::Receiver<AutomationMessage>>>,
    stop_flag: Arc<AtomicBool>,

    // Configuration
    points: Vec<ClickPoint>,
    total_hours: i32,
    total_minutes: i32,
    step_delay: i32,
    max_retries: i32,
    step4_wait: i32,
    dry_run: bool,

    // GUI state
    log_messages: Vec<String>,
    status: String,
    time_remaining: i32,
    running: bool,
    edit_mode: bool,

    // Statistics
    iterations: u32,
    start_time: Option<Instant>,

    // Components
    telemetry: Arc<Telemetry>,
    screenshots: Arc<ScreenshotManager>,
    hotkeys: HotkeyMonitor,
    gui_sender: Sender<AutomationMessage>,
}

impl AppState {
    fn new(args: CliArgs) -> Self {
        setup_logging();

        let points = load_points();
        let (tx, rx) = mpsc::channel();
        let telemetry = Telemetry::new(args.telemetry);
        let screenshots = ScreenshotManager::new(args.screenshots);

        telemetry.log("Application started");

        Self {
            automation_thread: None,
            stop_sender: None,
            message_receiver: Arc::new(Mutex::new(rx)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            points,
            total_hours: 11,
            total_minutes: 30,
            step_delay: 10,
            max_retries: 3,
            step4_wait: 10,
            dry_run: args.dry_run,
            log_messages: Vec::new(),
            status: "Status: Ready".to_string(),
            time_remaining: 0,
            running: false,
            edit_mode: false,
            iterations: 0,
            start_time: None,
            telemetry,
            screenshots,
            hotkeys: HotkeyMonitor::new(),
            gui_sender: tx,
        }
    }

    fn start_automation(&mut self) {
        if self.running {
            return;
        }

        self.running = true;
        self.iterations = 0;
        self.start_time = Some(Instant::now());
        self.stop_flag.store(false, Ordering::SeqCst);

        let total_seconds = self.total_hours * 3600 + self.total_minutes * 60;

        let (tx_stop, rx_stop) = mpsc::channel();
        self.stop_sender = Some(tx_stop);

        let thread = AutomationThread::new(
            self.points.clone(),
            total_seconds,
            self.step_delay,
            self.max_retries,
            self.step4_wait,
            self.dry_run,
            self.gui_sender.clone(),
            rx_stop,
            self.stop_flag.clone(),
        );

        self.telemetry.log(format!(
            "START: {}h{}m, retries={}, dry_run={}",
            self.total_hours, self.total_minutes, self.max_retries, self.dry_run
        ));

        self.automation_thread = Some(thread::spawn(move || {
            thread.run();
        }));

        self.add_log("Automation started");
    }

    fn stop_automation(&mut self) {
        if !self.running {
            return;
        }

        self.running = false;
        self.stop_flag.store(true, Ordering::SeqCst);

        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(thread) = self.automation_thread.take() {
            let _ = thread.join();
        }

        if let Some(start) = self.start_time {
            let duration = start.elapsed().as_secs_f64();
            self.telemetry.log(format!(
                "COMPLETE: duration={:.1}s, iterations={}",
                duration, self.iterations
            ));
        }

        self.add_log("Automation stopped");
        self.status = "Status: Stopped".to_string();
    }

    fn process_messages(&mut self) {
        let receiver = self.message_receiver.lock().unwrap();

        while let Ok(msg) = receiver.try_recv() {
            match msg {
                AutomationMessage::Log(text) => {
                    self.add_log(&text);
                }
                AutomationMessage::Status(text) => {
                    self.status = text;
                }
                AutomationMessage::UpdateTimer(remaining) => {
                    self.time_remaining = remaining;
                }
                AutomationMessage::ErrorPopup(text) => {
                    self.add_log(&format!("ERROR: {}", text));
                }
                AutomationMessage::Stop => {
                    self.running = false;
                }
            }
        }

        // Check for iteration completion
        if self.running {
            let msg_count = self.log_messages.iter()
                .filter(|m| m.contains("Iteration") && m.contains("complete"))
                .count();
            self.iterations = msg_count as u32;
        }
    }

    fn add_log(&mut self, message: &str) {
        let timestamp = Local::now().format("%H:%M:%S");
        let formatted = format!("[{}] {}", timestamp, message);
        self.log_messages.push(formatted);

        // Limit log size
        if self.log_messages.len() > 500 {
            self.log_messages.drain(0..100);
        }
    }

    fn save_points(&mut self) {
        save_points(&self.points);
        self.telemetry.log("Configuration saved");
        self.add_log("Points saved");
    }
}

// ============================================================================
// MAIN APPLICATION
// ============================================================================

struct AutomationApp {
    state: AppState,
}

impl AutomationApp {
    fn new(_cc: &eframe::CreationContext<'_>, args: CliArgs) -> Self {
        Self {
            state: AppState::new(args),
        }
    }
}

impl eframe::App for AutomationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process messages
        self.state.process_messages();

        // Check emergency stop
        if self.state.running && self.state.hotkeys.check_emergency_stop() {
            self.state.add_log("EMERGENCY STOP TRIGGERED");
            self.state.stop_automation();
        }

        // Request repaint for timer updates
        if self.state.running {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(APP_TITLE).size(20.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("v{}", APP_VERSION));
                });
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(&self.state.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Iterations: {}", self.state.iterations));
                });
            });

            if self.state.running {
                ui.separator();

                ui.horizontal(|ui| {
                    let hours = self.state.time_remaining / 3600;
                    let minutes = (self.state.time_remaining % 3600) / 60;
                    let seconds = self.state.time_remaining % 60;
                    ui.label(format!("Timer: {:02}:{:02}:{:02}", hours, minutes, seconds));

                    let total = (self.state.total_hours * 3600 + self.state.total_minutes * 60) as f32;
                    let progress = if total > 0.0 {
                        1.0 - (self.state.time_remaining as f32 / total)
                    } else {
                        0.0
                    };

                    ui.add(egui::ProgressBar::new(progress).show_percentage().animate(true));
                });

                if let Some(start) = self.state.start_time {
                    ui.separator();
                    ui.label(format!("Elapsed: {:.0}s", start.elapsed().as_secs()));
                }
            }
            ui.add_space(4.0);
        });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left panel - Controls
                ui.vertical(|ui| {
                    ui.set_width(320.0);

                    // Start/Stop buttons
                    ui.group(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .add_enabled(
                                        !self.state.running,
                                        egui::Button::new("▶ Start").min_size([140.0, 30.0].into()),
                                    )
                                    .clicked()
                                {
                                    self.state.start_automation();
                                }

                                if ui
                                    .add_enabled(
                                        self.state.running,
                                        egui::Button::new("⏹ Stop").min_size([140.0, 30.0].into()),
                                    )
                                    .clicked()
                                {
                                    self.state.stop_automation();
                                }
                            });
                        });
                    });

                    ui.add_space(8.0);

                    // Settings
                    ui.group(|ui| {
                        ui.heading("⚙ Settings");
                        ui.separator();

                        ui.label("Total Wait:");
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            ui.add(
                                egui::DragValue::new(&mut self.state.total_hours)
                                    .clamp_range(0..=24)
                                    .suffix(" h")
                                    .speed(0.1),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            ui.add(
                                egui::DragValue::new(&mut self.state.total_minutes)
                                    .clamp_range(0..=59)
                                    .suffix(" m")
                                    .speed(0.1),
                            );
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Step Delay:");
                            ui.add(
                                egui::DragValue::new(&mut self.state.step_delay)
                                    .clamp_range(0..=60)
                                    .suffix(" s")
                                    .speed(0.1),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Max Retries:");
                            ui.add(
                                egui::DragValue::new(&mut self.state.max_retries)
                                    .clamp_range(1..=10)
                                    .speed(0.1),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Step 4 Wait:");
                            ui.add(
                                egui::DragValue::new(&mut self.state.step4_wait)
                                    .clamp_range(0..=300)
                                    .suffix(" s")
                                    .speed(0.1),
                            );
                        });

                        ui.add_space(4.0);
                        ui.checkbox(&mut self.state.dry_run, "🧪 Dry Run");
                    });

                    ui.add_space(8.0);

                    // Click points
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading("📍 Points");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.checkbox(&mut self.state.edit_mode, "✏ Edit");
                            });
                        });
                        ui.separator();

                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for (i, point) in self.state.points.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}.", i + 1));

                                    if self.state.edit_mode {
                                        ui.text_edit_singleline(&mut point.name);
                                        ui.add(egui::DragValue::new(&mut point.x).prefix("x:"));
                                        ui.add(egui::DragValue::new(&mut point.y).prefix("y:"));
                                    } else {
                                        ui.label(&point.name);
                                        ui.label(
                                            egui::RichText::new(format!("({}, {})", point.x, point.y))
                                                .weak(),
                                        );
                                    }
                                });
                            }
                        });

                        if self.state.edit_mode {
                            ui.separator();
                            if ui.button("💾 Save").clicked() {
                                self.state.save_points();
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Info
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("ℹ Info").strong());
                        ui.separator();

                        if is_windows() {
                            ui.label("🔴 DELETE = Emergency Stop");
                        } else {
                            ui.label("⚠ Hotkeys: Windows only");
                        }

                        ui.label("📄 automation_log.txt");

                        if self.state.telemetry.enabled {
                            ui.label("📊 logs/telemetry.log");
                        }

                        if self.state.screenshots.is_enabled() {
                            ui.label("📸 screenshots/");
                        }
                    });
                });

                ui.separator();

                // Right panel - Log
                ui.vertical(|ui| {
                    ui.heading("📋 Activity Log");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            if self.state.log_messages.is_empty() {
                                ui.label(
                                    egui::RichText::new("No activity yet...")
                                        .weak()
                                        .italics(),
                                );
                            } else {
                                for msg in &self.state.log_messages {
                                    ui.label(egui::RichText::new(msg).monospace());
                                }
                            }
                        });
                });
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.state.running {
            self.state.stop_automation();
            thread::sleep(Duration::from_millis(500));
        }
        self.state.telemetry.log("Application exiting");
    }
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

fn main() -> Result<(), eframe::Error> {
    let args = CliArgs::parse();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 650.0])
            .with_min_inner_size([900.0, 550.0])
            .with_title(APP_TITLE),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(AutomationApp::new(cc, args)))),
    )
}
