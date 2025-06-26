use crate::models::app_state::{AppState, InputMode};
use crate::network_tools::ping;
use crossterm::event::KeyCode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn handle_enter_address_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
            app_state.address.clear();
            app_state.selected_tool = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Enter => {
            if let Some(ref tool) = app_state.selected_tool {
                let address = app_state.address.trim();
                let result = match tool.as_str() {
                    "ping" => ping(address).map(|output| {
                        // Store the PingResult as a JSON string for simplicity
                        serde_json::to_string(&output)
                            .unwrap_or_else(|_| "Failed to serialize ping result.".to_string())
                    }),
                    // Handle other tools if necessary
                    _ => Err("Unsupported tool".into()),
                };

                match result {
                    Ok(json_result) => {
                        app_state.result = Some(json_result);
                        app_state.input_mode = InputMode::ViewResults;
                    }
                    Err(e) => {
                        println!("Failed to parse packet stats: {:?}", e);
                        app_state.error_message = Some(format!("Error: {}", e));
                        app_state.input_mode = InputMode::Normal;
                    }
                }
                app_state.address.clear();
            } else {
                app_state.error_message = Some("No tool selected.".to_string());
                app_state.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app_state.address.push(c);
        }
        KeyCode::Backspace => {
            app_state.address.pop();
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_view_results_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app_state.input_mode = InputMode::Normal;
            app_state.result = None;
            app_state.selected_tool = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_speed_test_running_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        // Allow the user to cancel the live speed test by pressing Esc.
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
            app_state.speed_test_receiver = None;
        }
        // Allow quitting
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
    Ok(())
}
