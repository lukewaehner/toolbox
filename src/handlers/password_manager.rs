use crate::models::app_state::AppState;
use crate::password_manager::{save_password, PasswordEntry};
use crossterm::event::KeyCode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn handle_editing_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = crate::models::app_state::InputMode::Normal;
            app_state.error_message = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Tab => {
            app_state.input_field = (app_state.input_field + 1) % 3;
        }
        KeyCode::BackTab => {
            app_state.input_field = (app_state.input_field + 2) % 3;
        }
        KeyCode::Enter => {
            let entry = PasswordEntry {
                service: app_state.service.clone(),
                username: app_state.username.clone(),
                password: app_state.password.clone(),
            };

            if let Err(e) = save_password(&entry) {
                app_state.error_message = Some(format!("Error saving password: {}", e));
            } else {
                app_state.error_message = Some("Password saved successfully.".to_string());
                app_state.service.clear();
                app_state.username.clear();
                app_state.password.clear();
                app_state.input_mode = crate::models::app_state::InputMode::Normal;
            }
        }
        KeyCode::Char(c) => match app_state.input_field {
            0 => app_state.service.push(c),
            1 => app_state.username.push(c),
            2 => app_state.password.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app_state.input_field {
            0 => {
                app_state.service.pop();
            }
            1 => {
                app_state.username.pop();
            }
            2 => {
                app_state.password.pop();
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

pub fn handle_viewing_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app_state.input_mode = crate::models::app_state::InputMode::Normal;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
    Ok(())
}
