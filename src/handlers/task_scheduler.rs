use crate::app::prepare_status_message;
use crate::models::app_state::{AppState, InputMode, StatusMessageType};
use crate::task_scheduler::{EmailConfig, ReminderType, SmsConfig};
use chrono::{Local, NaiveDateTime};
use crossterm::event::KeyCode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use lettre::transport::smtp::authentication::Credentials;
use lettre::SmtpTransport;

pub fn handle_adding_task_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
            app_state.task_title.clear();
            app_state.task_description.clear();
            app_state.task_due_date.clear();
            app_state.task_tags.clear();
        }
        KeyCode::Tab => {
            app_state.input_field = (app_state.input_field + 1) % 4; // Cycle through title, description, due date, tags
        }
        KeyCode::BackTab => {
            app_state.input_field = (app_state.input_field + 3) % 4;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Char(c) => match app_state.input_field {
            0 => app_state.task_title.push(c),
            1 => app_state.task_description.push(c),
            2 => app_state.task_due_date.push(c),
            3 => app_state.task_tags.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app_state.input_field {
            0 => {
                app_state.task_title.pop();
            }
            1 => {
                app_state.task_description.pop();
            }
            2 => {
                app_state.task_due_date.pop();
            }
            3 => {
                app_state.task_tags.pop();
            }
            _ => {}
        },
        KeyCode::Enter => {
            // Parse due date
            let due_date = parse_due_date(&app_state.task_due_date);

            if let Some(timestamp) = due_date {
                // Parse tags
                let tags: Vec<String> = app_state
                    .task_tags
                    .split(',')
                    .map(|tag| tag.trim().to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect();

                // Add task
                if let Some(ref task_scheduler) = app_state.task_scheduler {
                    if let Ok(mut scheduler) = task_scheduler.lock() {
                        scheduler.add_task(
                            app_state.task_title.clone(),
                            app_state.task_description.clone(),
                            timestamp,
                            app_state.task_priority.clone(),
                            tags,
                        );

                        // Success message
                        app_state.status_message = Some(prepare_status_message(
                            "Task added successfully",
                            StatusMessageType::Success,
                            3,
                        ));

                        // Clear fields and return to normal mode
                        app_state.task_title.clear();
                        app_state.task_description.clear();
                        app_state.task_due_date.clear();
                        app_state.task_tags.clear();
                        app_state.input_mode = InputMode::Normal;
                    }
                }
            } else {
                // Error message for invalid date
                app_state.status_message = Some(prepare_status_message(
                    "Invalid due date format (use YYYY-MM-DD)",
                    StatusMessageType::Error,
                    3,
                ));
            }
        }
        KeyCode::Up => {
            // Cycle through priorities
            use crate::task_scheduler::TaskPriority;
            app_state.task_priority = match app_state.task_priority {
                TaskPriority::Low => TaskPriority::Urgent,
                TaskPriority::Medium => TaskPriority::Low,
                TaskPriority::High => TaskPriority::Medium,
                TaskPriority::Urgent => TaskPriority::High,
            };
        }
        KeyCode::Down => {
            // Cycle through priorities
            use crate::task_scheduler::TaskPriority;
            app_state.task_priority = match app_state.task_priority {
                TaskPriority::Low => TaskPriority::Medium,
                TaskPriority::Medium => TaskPriority::High,
                TaskPriority::High => TaskPriority::Urgent,
                TaskPriority::Urgent => TaskPriority::Low,
            };
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_viewing_tasks_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
            app_state.selected_task_id = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Char('r') => {
            // Add reminder to selected task
            if let Some(task_id) = app_state.selected_task_id {
                app_state.input_mode = InputMode::AddingReminder;
                app_state.reminder_date = Local::now().format("%Y-%m-%d").to_string();
                app_state.reminder_time = Local::now().format("%H:%M").to_string();
            }
        }
        KeyCode::Char('c') => {
            // Mark selected task as completed
            if let Some(task_id) = app_state.selected_task_id {
                if let Some(ref scheduler) = app_state.task_scheduler {
                    if let Ok(mut scheduler) = scheduler.lock() {
                        if let Some(task) = scheduler.get_task(task_id).cloned() {
                            let mut updated_task = task.clone();
                            updated_task.status = crate::task_scheduler::TaskStatus::Completed;
                            if let Err(e) = scheduler.update_task(task_id, updated_task) {
                                app_state.status_message = Some(crate::app::prepare_status_message(
                                    &format!("Failed to update task: {}", e),
                                    StatusMessageType::Error,
                                    3,
                                ));
                            } else {
                                app_state.status_message = Some(crate::app::prepare_status_message(
                                    "Task marked as completed",
                                    StatusMessageType::Success,
                                    3,
                                ));
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            // Delete selected task
            if let Some(task_id) = app_state.selected_task_id {
                if let Some(ref scheduler) = app_state.task_scheduler {
                    if let Ok(mut scheduler) = scheduler.lock() {
                        if let Err(e) = scheduler.delete_task(task_id) {
                            app_state.status_message = Some(crate::app::prepare_status_message(
                                &format!("Failed to delete task: {}", e),
                                StatusMessageType::Error,
                                3,
                            ));
                        } else {
                            app_state.status_message = Some(crate::app::prepare_status_message(
                                "Task deleted",
                                StatusMessageType::Success,
                                3,
                            ));
                            app_state.selected_task_id = None;
                        }
                    }
                }
            }
        }
        KeyCode::Up => {
            // Navigate to previous task
            if let Some(ref scheduler) = app_state.task_scheduler {
                if let Ok(scheduler) = scheduler.lock() {
                    let tasks = scheduler.get_all_tasks();
                    if !tasks.is_empty() {
                        if let Some(current_id) = app_state.selected_task_id {
                            // Find current index
                            let current_index = tasks.iter().position(|t| t.id == current_id);
                            if let Some(idx) = current_index {
                                // Move to previous task or wrap to last
                                let new_idx = if idx > 0 { idx - 1 } else { tasks.len() - 1 };
                                app_state.selected_task_id = Some(tasks[new_idx].id);
                            }
                        } else {
                            // No selection, select last task
                            app_state.selected_task_id = Some(tasks.last().unwrap().id);
                        }
                    }
                }
            }
        }
        KeyCode::Down => {
            // Navigate to next task
            if let Some(ref scheduler) = app_state.task_scheduler {
                if let Ok(scheduler) = scheduler.lock() {
                    let tasks = scheduler.get_all_tasks();
                    if !tasks.is_empty() {
                        if let Some(current_id) = app_state.selected_task_id {
                            // Find current index
                            let current_index = tasks.iter().position(|t| t.id == current_id);
                            if let Some(idx) = current_index {
                                // Move to next task or wrap to first
                                let new_idx = if idx < tasks.len() - 1 { idx + 1 } else { 0 };
                                app_state.selected_task_id = Some(tasks[new_idx].id);
                            }
                        } else {
                            // No selection, select first task
                            app_state.selected_task_id = Some(tasks[0].id);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_email_config_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
        }
        KeyCode::Tab => {
            // Cycle through: email fields (0-4) and test button (5)
            app_state.email_config_field = (app_state.email_config_field + 1) % 6;
        }
        KeyCode::BackTab => {
            app_state.email_config_field = (app_state.email_config_field + 5) % 6;
        }
        // Test email when on test button (field 5) or when pressing 't'
        KeyCode::Char('t') | KeyCode::Enter if app_state.email_config_field == 5 => {
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(scheduler) = task_scheduler.lock() {
                    match scheduler.test_email_config() {
                        Ok(_) => {
                            app_state.status_message = Some(crate::app::prepare_status_message(
                                "Test email sent successfully!",
                                StatusMessageType::Success,
                                5,
                            ));
                        }
                        Err(e) => {
                            app_state.status_message = Some(crate::app::prepare_status_message(
                                &format!("Failed to send test email: {}", e),
                                StatusMessageType::Error,
                                10, // Longer display time for errors
                            ));
                        }
                    }
                }
            }
        }
        KeyCode::Char(c) if app_state.email_config_field < 5 => {
            match app_state.email_config_field {
                0 => app_state.email_address.push(c),
                1 => app_state.email_smtp_server.push(c),
                2 => app_state.email_smtp_port.push(c),
                3 => app_state.email_username.push(c),
                4 => app_state.email_password.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace if app_state.email_config_field < 5 => {
            match app_state.email_config_field {
                0 => {
                    app_state.email_address.pop();
                }
                1 => {
                    app_state.email_smtp_server.pop();
                }
                2 => {
                    app_state.email_smtp_port.pop();
                }
                3 => {
                    app_state.email_username.pop();
                }
                4 => {
                    app_state.email_password.pop();
                }
                _ => {}
            }
        }
        KeyCode::Enter if app_state.email_config_field < 5 => {
            println!("Saving email config...");
            // Validate and save email config
            if app_state.email_address.is_empty()
                || app_state.email_smtp_server.is_empty()
                || app_state.email_smtp_port.is_empty()
                || app_state.email_username.is_empty()
                || app_state.email_password.is_empty()
            {
                println!("Email config validation failed - empty fields");
                app_state.status_message = Some(crate::app::prepare_status_message(
                    "All fields are required",
                    StatusMessageType::Error,
                    3,
                ));
                return Ok(());
            }

            // Parse port
            let port = match app_state.email_smtp_port.parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    app_state.status_message = Some(crate::app::prepare_status_message(
                        "Invalid port number",
                        StatusMessageType::Error,
                        3,
                    ));
                    return Ok(());
                }
            };

            // Create config
            let config = crate::task_scheduler::EmailConfig {
                email: app_state.email_address.clone(),
                smtp_server: app_state.email_smtp_server.clone(),
                smtp_port: port,
                username: app_state.email_username.clone(),
                password: app_state.email_password.clone(),
                retry_attempts: 3,
                retry_delay_seconds: 300,
            };

            // Set config
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(mut scheduler) = task_scheduler.lock() {
                    scheduler.set_email_config(config);
                    app_state.status_message = Some(crate::app::prepare_status_message(
                        "Email configuration saved",
                        StatusMessageType::Success,
                        3,
                    ));
                    // Do NOT return to normal mode here
                    // app_state.input_mode = InputMode::Normal;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_adding_reminder_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::ViewingTasks;
            app_state.reminder_date.clear();
            app_state.reminder_time.clear();
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Tab => {
            app_state.input_field = (app_state.input_field + 1) % 3; // Cycle through date, time, and type
        }
        KeyCode::BackTab => {
            app_state.input_field = (app_state.input_field + 2) % 3;
        }
        KeyCode::Char(c) => match app_state.input_field {
            0 => app_state.reminder_date.push(c), // Date field
            1 => app_state.reminder_time.push(c), // Time field
            _ => {}
        },
        KeyCode::Backspace => match app_state.input_field {
            0 => {
                app_state.reminder_date.pop();
            }
            1 => {
                app_state.reminder_time.pop();
            }
            _ => {}
        },
        KeyCode::Up | KeyCode::Down if app_state.input_field == 2 => {
            // Toggle reminder type when on the type field
            app_state.reminder_type = match app_state.reminder_type {
                crate::task_scheduler::ReminderType::Email => crate::task_scheduler::ReminderType::Notification,
                crate::task_scheduler::ReminderType::Notification => crate::task_scheduler::ReminderType::Sms,
                crate::task_scheduler::ReminderType::Sms => crate::task_scheduler::ReminderType::Both,
                crate::task_scheduler::ReminderType::Both => crate::task_scheduler::ReminderType::All,
                crate::task_scheduler::ReminderType::All => crate::task_scheduler::ReminderType::Email,
            };
        },
        KeyCode::Char('n') => {
            // Special shortcut for testing - add reminder for 1 minute from now
            if let Some(task_id) = app_state.selected_task_id {
                let now = chrono::Utc::now();
                let one_minute_from_now = now + chrono::Duration::minutes(1);
                
                app_state.reminder_date = one_minute_from_now.format("%Y-%m-%d").to_string();
                app_state.reminder_time = one_minute_from_now.format("%H:%M").to_string();
                
                app_state.status_message = Some(prepare_status_message(
                    "Set reminder time to 1 minute from now. Press Enter to save.",
                    StatusMessageType::Info,
                    3,
                ));
            }
        },
        KeyCode::Char('t') => {
            // Special shortcut for testing - add immediate reminder (for testing)
            if let Some(task_id) = app_state.selected_task_id {
                // Set to current time
                let now = chrono::Utc::now();
                
                // Add reminder to task with current timestamp
                if let Some(ref task_scheduler) = app_state.task_scheduler {
                    if let Ok(mut scheduler) = task_scheduler.lock() {
                        match scheduler.add_reminder_to_task(
                            task_id,
                            now.timestamp(),
                            app_state.reminder_type.clone(),
                        ) {
                            Ok(_) => {
                                app_state.status_message = Some(prepare_status_message(
                                    "Immediate reminder added successfully - will trigger within 30 seconds",
                                    StatusMessageType::Success,
                                    5,
                                ));
                                app_state.input_mode = InputMode::ViewingTasks;
                                app_state.reminder_date.clear();
                                app_state.reminder_time.clear();
                            }
                            Err(e) => {
                                app_state.status_message = Some(prepare_status_message(
                                    &format!("Failed to add reminder: {}", e),
                                    StatusMessageType::Error,
                                    5,
                                ));
                            }
                        }
                    }
                }
            }
        },
        KeyCode::Enter => {
            // Parse date and time
            if let Some(task_id) = app_state.selected_task_id {
                if app_state.reminder_date.is_empty() || app_state.reminder_time.is_empty() {
                    app_state.status_message = Some(prepare_status_message(
                        "Date and time are required",
                        StatusMessageType::Error,
                        3,
                    ));
                    return Ok(());
                }

                // Parse datetime
                let datetime_str = format!("{} {}", app_state.reminder_date, app_state.reminder_time);
                let parsed_datetime = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M");
                
                if let Ok(datetime) = parsed_datetime {
                    let timestamp = datetime.timestamp();
                    
                    // Add reminder to task
                    if let Some(ref task_scheduler) = app_state.task_scheduler {
                        if let Ok(mut scheduler) = task_scheduler.lock() {
                            match scheduler.add_reminder_to_task(
                                task_id,
                                timestamp,
                                app_state.reminder_type.clone(),
                            ) {
                                Ok(_) => {
                                    app_state.status_message = Some(prepare_status_message(
                                        "Reminder added successfully",
                                        StatusMessageType::Success,
                                        3,
                                    ));
                                    app_state.input_mode = InputMode::ViewingTasks;
                                    app_state.reminder_date.clear();
                                    app_state.reminder_time.clear();
                                }
                                Err(e) => {
                                    app_state.status_message = Some(prepare_status_message(
                                        &format!("Failed to add reminder: {}", e),
                                        StatusMessageType::Error,
                                        5,
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    app_state.status_message = Some(prepare_status_message(
                        "Invalid date/time format. Use YYYY-MM-DD for date and HH:MM for time",
                        StatusMessageType::Error,
                        5,
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_sms_config_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
        }
        KeyCode::Tab => {
            // Cycle through: phone number (0), carrier (1), enabled toggle (2)
            app_state.sms_config_field = (app_state.sms_config_field + 1) % 3;
        }
        KeyCode::BackTab => {
            app_state.sms_config_field = (app_state.sms_config_field + 2) % 3;
        }
        KeyCode::Char(c) if app_state.sms_config_field < 2 => {
            match app_state.sms_config_field {
                0 => {
                    // Only allow digits, spaces, dashes, and parentheses for phone number
                    if c.is_ascii_digit() || matches!(c, ' ' | '-' | '(' | ')' | '+') {
                        app_state.sms_phone_number.push(c);
                    }
                }
                1 => app_state.sms_carrier.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace if app_state.sms_config_field < 2 => {
            match app_state.sms_config_field {
                0 => {
                    app_state.sms_phone_number.pop();
                }
                1 => {
                    app_state.sms_carrier.pop();
                }
                _ => {}
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') if app_state.sms_config_field == 2 => {
            // Toggle enabled status
            app_state.sms_enabled = !app_state.sms_enabled;
        }
        KeyCode::Up | KeyCode::Down if app_state.sms_config_field == 1 => {
            // Cycle through common carriers
            let carriers = ["att", "verizon", "tmobile", "sprint", "boost", "cricket", "metropcs"];
            if let Some(current_index) = carriers.iter().position(|&c| c == app_state.sms_carrier) {
                let next_index = if code == KeyCode::Up {
                    (current_index + carriers.len() - 1) % carriers.len()
                } else {
                    (current_index + 1) % carriers.len()
                };
                app_state.sms_carrier = carriers[next_index].to_string();
            } else {
                app_state.sms_carrier = "att".to_string();
            }
        }
        KeyCode::Enter if app_state.sms_config_field < 2 => {
            // Save SMS configuration
            if app_state.sms_phone_number.is_empty() {
                app_state.status_message = Some(prepare_status_message(
                    "Phone number is required",
                    StatusMessageType::Error,
                    3,
                ));
                return Ok(());
            }

            if app_state.sms_carrier.is_empty() {
                app_state.status_message = Some(prepare_status_message(
                    "Carrier is required",
                    StatusMessageType::Error,
                    3,
                ));
                return Ok(());
            }

            // Create SMS config
            let config = SmsConfig {
                phone_number: app_state.sms_phone_number.clone(),
                carrier: app_state.sms_carrier.clone(),
                enabled: app_state.sms_enabled,
            };

            // Set config
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(mut scheduler) = task_scheduler.lock() {
                    scheduler.set_sms_config(config);
                    app_state.status_message = Some(prepare_status_message(
                        "SMS configuration saved",
                        StatusMessageType::Success,
                        3,
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// Helper function to parse due date
fn parse_due_date(date_str: &str) -> Option<i64> {
    // Try to parse as YYYY-MM-DD
    if let Ok(date) =
        NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
    {
        return Some(date.timestamp());
    }

    None
}
