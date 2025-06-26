use crate::app::prepare_status_message;
use crate::models::app_state::{AppState, InputMode, MenuItem, StatusMessageType};
use crate::network_tools::{ping, SpeedTestResult};
use crate::system_utilities::SystemMonitor;
use chrono::Local;
use crossterm::event::KeyCode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn handle_normal_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match (code, &app_state.active_menu) {
        (KeyCode::Char('q'), _) => {
            crate::app::cleanup_resources(app_state);
            running.store(false, Ordering::Relaxed);
        }
        (KeyCode::Char('1'), MenuItem::Main) => {
            app_state.active_menu = MenuItem::PasswordManager;
        }
        (KeyCode::Char('2'), MenuItem::Main) => {
            app_state.active_menu = MenuItem::NetworkTools;
        }
        (KeyCode::Char('3'), MenuItem::Main) => {
            app_state.active_menu = MenuItem::SystemUtilities;

            // Initialize the system monitor if it doesn't exist yet
            if app_state.system_monitor.is_none() {
                let monitor = SystemMonitor::new(
                    60,                          // Keep 60 data points for history (1 minute at 1 second refresh)
                    Duration::from_millis(1000), // Refresh every second
                );
                app_state.system_monitor = Some(Arc::new(Mutex::new(monitor)));
            }

            // Get an initial snapshot
            if let Some(ref monitor) = app_state.system_monitor {
                if let Ok(mut monitor) = monitor.lock() {
                    app_state.system_snapshot = Some(monitor.refresh_and_get());
                }
            }
        }
        (KeyCode::Char('4'), MenuItem::Main) => {
            app_state.active_menu = MenuItem::TaskScheduler;
        }
        (KeyCode::Char('a'), MenuItem::TaskScheduler) => {
            app_state.input_mode = InputMode::AddingTask;
            // Initialize with today's date
            let today = Local::now();
            app_state.task_due_date = today.format("%Y-%m-%d").to_string();
        }
        (KeyCode::Char('v'), MenuItem::TaskScheduler) => {
            app_state.input_mode = InputMode::ViewingTasks;
        }
        (KeyCode::Char('e'), MenuItem::TaskScheduler) => {
            app_state.input_mode = InputMode::ConfiguringEmail;
        }
        (KeyCode::Char('r'), MenuItem::SystemUtilities) => {
            app_state.selected_system_tool = Some("resource_monitor".to_string());
            app_state.system_view_mode = crate::models::app_state::SystemViewMode::Overview;
        }
        (KeyCode::Char('p'), MenuItem::SystemUtilities) => {
            app_state.selected_system_tool = Some("process_manager".to_string());
            app_state.system_view_mode = crate::models::app_state::SystemViewMode::ProcessList;
        }
        (KeyCode::Char('d'), MenuItem::SystemUtilities) => {
            app_state.selected_system_tool = Some("disk_analyzer".to_string());
            app_state.system_view_mode = crate::models::app_state::SystemViewMode::DiskDetails;
        }
        (KeyCode::Char('a'), MenuItem::PasswordManager) => {
            app_state.input_mode = InputMode::Editing;
        }
        (KeyCode::Char('v'), MenuItem::PasswordManager) => {
            app_state.input_mode = InputMode::Viewing;
        }
        (KeyCode::Char('p'), MenuItem::NetworkTools) => {
            app_state.selected_tool = Some("ping".to_string());
            app_state.input_mode = InputMode::EnterAddress;
        }
        (KeyCode::Char('t'), MenuItem::NetworkTools) => {
            app_state.selected_tool = Some("traceroute".to_string());
            app_state.input_mode = InputMode::EnterAddress;
        }
        (KeyCode::Char('s'), MenuItem::NetworkTools) => {
            app_state.input_mode = InputMode::SpeedTestRunning;
            let (tx, rx) = mpsc::channel();
            app_state.speed_test_receiver = Some(rx);

            // Create a status message right away
            app_state.result = Some("Starting speed test. This may take a moment...\n\nConnecting to speed test servers...".to_string());

            thread::spawn(move || {
                // Send a status update
                let _ = tx.send(SpeedTestResult::status(
                    "Connecting to speed test servers...",
                ));

                // Wait briefly to show the message
                thread::sleep(Duration::from_millis(500));

                // Try single file test first
                match crate::network_tools::measure_speed() {
                    // ... rest of the speed test code ...
                    Ok(single_result) => {
                        // Send the initial result
                        let _ = tx.send(SpeedTestResult {
                            status_message: format!(
                                "Initial test: {:.2} Mbps",
                                single_result.download_speed_bps / 1_000_000.0
                            ),
                            ..single_result.clone()
                        });

                        // If the speed is reasonably fast, try a multi-connection test
                        if single_result.download_speed_bps > 10_000_000.0 {
                            // > 10 Mbps
                            // Update status
                            let _ = tx.send(SpeedTestResult::status(
                                "Fast connection detected. Running multi-connection test...",
                            ));

                            // Try parallel test
                            match crate::network_tools::parallel_speed_test() {
                                Ok(parallel_result) => {
                                    // Send the final result
                                    let _ = tx.send(parallel_result);
                                }
                                Err(e) => {
                                    eprintln!("Parallel test error: {}", e);
                                    // Send a status update
                                    let _ = tx.send(SpeedTestResult::status(
                                        &format!("Multi-connection test failed. Using single-connection result: {:.2} Mbps", 
                                            single_result.download_speed_bps / 1_000_000.0)
                                    ));
                                }
                            }
                        } else {
                            // For slower connections, just use the single file result
                            let _ = tx.send(SpeedTestResult::status(&format!(
                                "Test complete. Download speed: {:.2} Mbps",
                                single_result.download_speed_bps / 1_000_000.0
                            )));
                        }
                    }
                    Err(e) => {
                        eprintln!("Speed test error: {}", e);
                        let _ = tx.send(SpeedTestResult::error(&e.to_string()));
                    }
                }
                // Thread will exit after test is complete
            });
        }
        (KeyCode::Esc, menu_item) if *menu_item != MenuItem::Main => {
            app_state.active_menu = MenuItem::Main;
            app_state.input_mode = InputMode::Normal;
            app_state.selected_tool = None;
            app_state.address.clear();
            app_state.result = None;
            app_state.error_message = None;
        }
        (KeyCode::Esc, MenuItem::SystemUtilities) => {
            // Going back from the SystemUtilities menu to Main menu
            app_state.active_menu = MenuItem::Main;
        }
        _ => {}
    }
    Ok(())
}
