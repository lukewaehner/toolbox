use crate::app::prepare_status_message;
use crate::models::app_state::{
    AppState, ConfirmationDialogue, ProcessSortType, StatusMessageType, SystemViewMode,
};
use crossterm::event::KeyCode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn handle_system_utilities_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    // Handle confirmation dialog if active
    if app_state.confirmation_dialogue != ConfirmationDialogue::None {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let ConfirmationDialogue::KillProcess(pid, ref name) =
                    app_state.confirmation_dialogue.clone()
                {
                    // Actually kill the process
                    let kill_result = if let Some(ref monitor) = app_state.system_monitor {
                        if let Ok(mut monitor) = monitor.lock() {
                            let result = monitor.kill_process(pid);

                            // Refresh process list
                            app_state.system_snapshot = Some(monitor.refresh_and_get());

                            result
                        } else {
                            Err(format!("Failed to access system monitor"))
                        }
                    } else {
                        Err(format!("System monitor not available"))
                    };

                    // Set appropriate status message
                    match kill_result {
                        Ok(_) => {
                            // Success message
                            app_state.status_message = Some(prepare_status_message(
                                &format!(
                                    "Process '{}' (PID: {}) terminated successfully",
                                    name, pid
                                ),
                                StatusMessageType::Success,
                                3,
                            ));
                        }
                        Err(e) => {
                            // Error message
                            app_state.status_message = Some(prepare_status_message(
                                &format!("Failed to kill process: {}", e),
                                StatusMessageType::Error,
                                5,
                            ));
                        }
                    }
                }
                // Close the dialog
                app_state.confirmation_dialogue = ConfirmationDialogue::None;
                return Ok(());
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // Cancel and close the dialog
                app_state.confirmation_dialogue = ConfirmationDialogue::None;
                return Ok(());
            }
            _ => return Ok(()), // Ignore other keys while dialog is active
        }
    }

    // Regular key handling when no dialog is active
    match code {
        KeyCode::Esc => {
            // If we're in a tool view, go back to the system utilities menu
            if app_state.selected_system_tool.is_some() {
                app_state.selected_system_tool = None;
            } else {
                // Otherwise, go back to the main menu
                app_state.active_menu = crate::models::app_state::MenuItem::Main;
                // No need to free up resources when just navigating back to main menu
            }
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Char('r') => {
            // In the menu, 'r' selects resource monitor
            if app_state.selected_system_tool.is_none() {
                app_state.selected_system_tool = Some("resource_monitor".to_string());
            } else {
                // In a tool view, 'r' refreshes data
                let refresh_result = if let Some(ref monitor) = app_state.system_monitor {
                    if let Ok(mut monitor) = monitor.lock() {
                        app_state.system_snapshot = Some(monitor.refresh_and_get());
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // If in process manager and refresh was successful, show message
                if refresh_result
                    && app_state.selected_system_tool == Some("process_manager".to_string())
                {
                    app_state.status_message = Some(prepare_status_message(
                        "Process list refreshed",
                        StatusMessageType::Info,
                        2,
                    ));
                }
            }
        }
        KeyCode::Char('p') => {
            // Handle different behaviors for 'p'
            if app_state.selected_system_tool.is_none() {
                // Open Process Manager
                app_state.selected_system_tool = Some("process_manager".to_string());
                app_state.selected_process_pid = None; // Reset selected process when opening

                // Initialize process selection
                crate::ui::system_utilities::initialize_process_selection(app_state);
            } else if app_state.selected_system_tool == Some("process_manager".to_string()) {
                // Sort by PID in Process Manager
                app_state.process_sort_type = ProcessSortType::Pid;
                app_state.status_message = Some(prepare_status_message(
                    "Sorted by Process ID",
                    StatusMessageType::Info,
                    2,
                ));
            }
        }
        KeyCode::Char('d') => {
            // Disk analyzer
            if app_state.selected_system_tool.is_none() {
                app_state.selected_system_tool = Some("disk_analyzer".to_string());
            }
        }
        KeyCode::Char('c') => {
            // Handle different behaviors for 'c'
            if app_state.selected_system_tool == Some("resource_monitor".to_string()) {
                // CPU details view in resource monitor
                app_state.system_view_mode = SystemViewMode::CpuDetails;
            } else if app_state.selected_system_tool == Some("process_manager".to_string()) {
                // Sort by CPU in Process Manager
                app_state.process_sort_type = ProcessSortType::CpuUsage;
                app_state.status_message = Some(prepare_status_message(
                    "Sorted by CPU usage",
                    StatusMessageType::Info,
                    2,
                ));
            }
        }
        KeyCode::Char('m') => {
            // Handle different behaviors for 'm'
            if app_state.selected_system_tool == Some("resource_monitor".to_string()) {
                // Memory details view in resource monitor
                app_state.system_view_mode = SystemViewMode::MemoryDetails;
            } else if app_state.selected_system_tool == Some("process_manager".to_string()) {
                // Sort by Memory in Process Manager
                app_state.process_sort_type = ProcessSortType::MemoryUsage;
                app_state.status_message = Some(prepare_status_message(
                    "Sorted by memory usage",
                    StatusMessageType::Info,
                    2,
                ));
            }
        }
        KeyCode::Char('k') => {
            // Kill selected process in Process Manager
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                if let Some(pid) = app_state.selected_process_pid {
                    // Get the process name for confirmation dialog
                    if let Some(ref system_snapshot) = app_state.system_snapshot {
                        if let Some(process) = system_snapshot.top_processes.iter().find(|p| p.pid == pid) {
                            app_state.confirmation_dialogue =
                                ConfirmationDialogue::KillProcess(pid, process.name.clone());
                        }
                    }
                } else {
                    app_state.status_message = Some(prepare_status_message(
                        "No process selected",
                        StatusMessageType::Error,
                        2,
                    ));
                }
            }
        }
        KeyCode::Up => {
            // Navigate to previous item in the list (in Process Manager)
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                if let Some(ref system_snapshot) = app_state.system_snapshot {
                    if !system_snapshot.top_processes.is_empty() {
                        if app_state.selected_process_index > 0 {
                            app_state.selected_process_index -= 1;
                        } else {
                            // Wrap to bottom
                            app_state.selected_process_index = system_snapshot.top_processes.len().saturating_sub(1);
                        }
                        
                        // Update the selected PID
                        if app_state.selected_process_index < system_snapshot.top_processes.len() {
                            app_state.selected_process_pid = Some(system_snapshot.top_processes[app_state.selected_process_index].pid);
                        }
                    }
                }
            }
        }
        KeyCode::Down => {
            // Navigate to next item in the list (in Process Manager)
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                if let Some(ref system_snapshot) = app_state.system_snapshot {
                    if !system_snapshot.top_processes.is_empty() {
                        app_state.selected_process_index = (app_state.selected_process_index + 1) % system_snapshot.top_processes.len();
                        
                        // Update the selected PID
                        app_state.selected_process_pid = Some(system_snapshot.top_processes[app_state.selected_process_index].pid);
                    }
                }
            }
        }
        // ... more key handling for system utilities ...
        _ => {}
    }
    Ok(())
}
