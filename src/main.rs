// Module imports
mod network_tools;
mod password_manager;
mod system_utilities;
mod task_scheduler;

// Crate list
use crate::task_scheduler::{EmailConfig, ReminderType, TaskPriority, TaskScheduler, TaskStatus};
use chrono::{Local, NaiveDateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use network_tools::{ping, SpeedTestResult};
use once_cell::sync::Lazy;
use password_manager::{save_password, PasswordEntry};
use signal_hook::consts::SIGINT;
use signal_hook::flag;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use system_utilities::SystemMonitor;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table},
    Frame, Terminal,
};

// Define the PingResult struct
#[derive(serde::Deserialize, Debug)]
struct PingResult {
    packets_transmitted: u32,
    packets_received: u32,
    packet_loss: f32,
    time: Option<u32>, // Change this to Option<u32> if it can be None
    round_trip_min: f32,
    round_trip_avg: f32,
    round_trip_max: f32,
    round_trip_mdev: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Editing,
    Viewing,
    EnterAddress,
    ViewResults,
    SpeedTestRunning,
    AddingTask,
    EditingTask,
    ViewingTasks,
    AddingReminder,
    ConfiguringEmail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MenuItem {
    Main,
    PasswordManager,
    NetworkTools,
    SystemUtilities,
    TaskScheduler,
}

#[derive(Debug, Clone, PartialEq)]
enum SystemViewMode {
    Overview,
    CpuDetails,
    MemoryDetails,
    DiskDetails,
    ProcessList,
}

#[derive(Debug, Clone, PartialEq)]
enum ConfirmationDialogue {
    None,
    KillProcess(u32, String), // Process ID and name
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessSortType {
    Pid,
    Name,
    CpuUsage,
    MemoryUsage,
    Runtime,
}

#[derive(Debug)]
struct AppState {
    active_menu: MenuItem,
    input_mode: InputMode,
    service: String,
    username: String,
    password: String,
    input_field: usize,
    error_message: Option<String>,
    address: String,
    result: Option<String>,
    selected_tool: Option<String>,
    speed_test_receiver: Option<Receiver<network_tools::SpeedTestResult>>,
    system_monitor: Option<Arc<Mutex<SystemMonitor>>>,
    selected_system_tool: Option<String>,
    system_view_mode: SystemViewMode,
    system_snapshot: Option<system_utilities::SystemSnapshot>,
    selected_process_index: usize,
    confirmation_dialogue: ConfirmationDialogue,
    status_message: Option<StatusMessage>,
    process_sort_type: ProcessSortType,
    selected_process_pid: Option<u32>,
    task_scheduler: Option<Arc<Mutex<TaskScheduler>>>,
    task_filter: Option<String>,
    task_title: String,
    task_description: String,
    task_due_date: String,
    task_priority: TaskPriority,
    task_tags: String,
    selected_task_id: Option<u32>,
    email_address: String,
    email_smtp_server: String,
    email_smtp_port: String,
    email_username: String,
    email_password: String,
    email_config_field: usize,
    reminder_date: String,
    reminder_time: String,
    reminder_type: ReminderType,
}

#[derive(Debug, Clone)]
struct StatusMessage {
    message: String,
    message_type: StatusMessageType,
    created_at: std::time::Instant,
    duration: std::time::Duration,
}

#[derive(Debug, Clone, PartialEq)]
enum StatusMessageType {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_menu: MenuItem::Main,
            input_mode: InputMode::Normal,
            service: String::new(),
            username: String::new(),
            password: String::new(),
            input_field: 0,
            error_message: None,
            address: String::new(),
            result: None,
            selected_tool: None,
            speed_test_receiver: None,
            system_monitor: None,
            selected_system_tool: None,
            system_view_mode: SystemViewMode::Overview,
            system_snapshot: None,
            selected_process_index: 0,
            confirmation_dialogue: ConfirmationDialogue::None,
            status_message: None,
            process_sort_type: ProcessSortType::CpuUsage,
            selected_process_pid: None,
            task_scheduler: None,
            task_filter: None,
            task_title: String::new(),
            task_description: String::new(),
            task_due_date: String::new(),
            task_priority: TaskPriority::Medium,
            task_tags: String::new(),
            selected_task_id: None,
            email_address: String::new(),
            email_smtp_server: String::new(),
            email_smtp_port: String::from("587"),
            email_username: String::new(),
            email_password: String::new(),
            email_config_field: 0,
            reminder_date: String::new(),
            reminder_time: String::new(),
            reminder_type: ReminderType::Email,
        }
    }
}

// Struct to handle terminal cleanup
struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        // Ensure terminal is reset
        if let Err(e) = disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {:?}", e);
        }
        if let Err(e) = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture // Ensure mouse capture is disabled
        ) {
            eprintln!(
                "Failed to leave alternate screen or disable mouse capture: {:?}",
                e
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Craete task scheduler
    let task_scheduler = Arc::new(Mutex::new(TaskScheduler::new("tasks.json")));

    let _scheduler_thread = task_scheduler::run_scheduler_background_thread(task_scheduler.clone());

    // Instantiate TerminalCleanup to ensure it is used
    let _cleanup = TerminalCleanup;

    // Setup signal handling for Ctrl+C
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    flag::register(SIGINT, r)?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    let mut app_state = AppState::default();
    app_state.task_scheduler = Some(task_scheduler.clone());

    let res = run_app(&mut terminal, running, app_state);

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    running: Arc<AtomicBool>,
    mut app_state: AppState,
) -> io::Result<()> {
    while running.load(Ordering::Relaxed) {
        if app_state.system_monitor.is_some() {
            if let Some(ref monitor) = app_state.system_monitor {
                if let Ok(mut monitor) = monitor.lock() {
                    if monitor.refresh_if_needed() {
                        app_state.system_snapshot = Some(monitor.snapshot().clone());
                    }
                }
            }
        }

        check_expired_status(&mut app_state);

        terminal.draw(|f| match app_state.active_menu {
            MenuItem::Main => draw_main_menu(f),
            MenuItem::PasswordManager => match app_state.input_mode {
                InputMode::Normal => draw_password_manager_menu(f),
                InputMode::Editing => draw_input_modal(f, &app_state),
                InputMode::Viewing => draw_password_list(f),
                _ => {}
            },
            MenuItem::NetworkTools => match app_state.input_mode {
                InputMode::Normal => draw_network_tools_menu(f),
                InputMode::EnterAddress => draw_address_input(f, &app_state),
                InputMode::ViewResults => draw_view_results(f, &app_state),
                InputMode::SpeedTestRunning => draw_speed_test(f, &app_state),
                _ => {}
            },
            MenuItem::SystemUtilities => {
                if let Some(ref tool) = app_state.selected_system_tool {
                    match tool.as_str() {
                        "resource_monitor" => draw_resource_monitor(f, &app_state),
                        "process_manager" => draw_process_list_detailed(f, &app_state),
                        "disk_analyzer" => draw_disk_analyzer(f, &app_state),
                        _ => draw_system_utilities_menu(f),
                    }
                } else {
                    draw_system_utilities_menu(f);
                }
            }
            MenuItem::TaskScheduler => match app_state.input_mode {
                InputMode::Normal => draw_task_scheduler_menu(f),
                InputMode::AddingTask => draw_add_task(f, &app_state),
                InputMode::ViewingTasks => draw_view_tasks(f, &app_state),
                InputMode::ConfiguringEmail => draw_email_config(f, &app_state),
                InputMode::AddingReminder => draw_add_reminder(f, &app_state),
                InputMode::Editing => {}
                InputMode::Viewing => {}
                InputMode::EnterAddress => {}
                InputMode::ViewResults => {}
                InputMode::SpeedTestRunning => {}
                InputMode::EditingTask => {}
            },
        })?;

        // Refresh system data if needed
        if app_state.active_menu == MenuItem::SystemUtilities && app_state.system_monitor.is_some()
        {
            if let Some(ref monitor) = app_state.system_monitor {
                if let Ok(mut monitor) = monitor.lock() {
                    if monitor.refresh_if_needed() {
                        app_state.system_snapshot = Some(monitor.snapshot().clone());
                    }
                }
            }
        }

        if event::poll(Duration::from_millis(10))? {
            if let Ok(event) = event::read() {
                match event {
                    event::Event::Key(KeyEvent { code, .. }) => match app_state.active_menu {
                        MenuItem::Main => handle_normal_mode(&mut app_state, code, &running)?,
                        MenuItem::PasswordManager => match app_state.input_mode {
                            InputMode::Normal => {
                                handle_normal_mode(&mut app_state, code, &running)?
                            }
                            InputMode::Editing => {
                                handle_editing_mode(&mut app_state, code, &running)?
                            }
                            InputMode::Viewing => {
                                handle_viewing_mode(&mut app_state, code, &running)?
                            }
                            _ => {}
                        },
                        MenuItem::NetworkTools => match app_state.input_mode {
                            InputMode::Normal => {
                                handle_normal_mode(&mut app_state, code, &running)?
                            }
                            InputMode::EnterAddress => {
                                handle_enter_address_mode(&mut app_state, code, &running)?
                            }
                            InputMode::ViewResults => {
                                handle_view_results_mode(&mut app_state, code, &running)?
                            }
                            InputMode::SpeedTestRunning => {
                                handle_speed_test_running_mode(&mut app_state, code, &running)?
                            }
                            _ => {}
                        },
                        MenuItem::SystemUtilities => {
                            handle_system_utilities_mode(&mut app_state, code, &running)?
                        }
                        MenuItem::TaskScheduler => match app_state.input_mode {
                            InputMode::Normal => {
                                handle_normal_mode(&mut app_state, code, &running)?
                            }
                            InputMode::AddingTask => {
                                handle_adding_task_mode(&mut app_state, code, &running)?
                            }
                            InputMode::ViewingTasks => {
                                handle_viewing_tasks_mode(&mut app_state, code, &running)?
                            }
                            InputMode::ConfiguringEmail => {
                                handle_email_config_mode(&mut app_state, code, &running)?
                            }
                            InputMode::AddingReminder => {
                                handle_adding_reminder_mode(&mut app_state, code, &running)?
                            }
                            // The below modes don't apply to TaskScheduler, so just do nothing
                            InputMode::Editing
                            | InputMode::Viewing
                            | InputMode::EnterAddress
                            | InputMode::ViewResults
                            | InputMode::SpeedTestRunning
                            | InputMode::EditingTask => {}
                        },
                    },
                    _ => {}
                }
            }
        }

        if let Some(ref task_scheduler) = app_state.task_scheduler {
            static LAST_REMINDER_CHECK: Lazy<Mutex<std::time::Instant>> =
                Lazy::new(|| Mutex::new(std::time::Instant::now()));

            if let Ok(mut last_check) = LAST_REMINDER_CHECK.lock() {
                if last_check.elapsed() >= Duration::from_secs(5) {
                    let mut triggered_reminders = Vec::new();

                    if let Ok(mut sched) = task_scheduler.lock() {
                        triggered_reminders = sched.check_reminders();
                    }

                    if !triggered_reminders.is_empty() {
                        let reminder_count = triggered_reminders.len();
                        app_state.status_message = Some(prepare_status_message(
                            &format!(
                                "{} reminder{} triggered",
                                reminder_count,
                                if reminder_count == 1 { "" } else { "s" }
                            ),
                            StatusMessageType::Info,
                            5,
                        ));
                    }
                    *last_check = std::time::Instant::now();
                }
            }
        }

        // the existing speed test receiver handling block
        if let Some(ref rx) = app_state.speed_test_receiver {
            loop {
                match rx.try_recv() {
                    Ok(speed_result) => {
                        // Format the result based on the test type
                        if speed_result.test_type == "error" {
                            app_state.result =
                                Some(format!("Speed test error: {}", speed_result.status_message));
                        } else if speed_result.test_type == "status" {
                            app_state.result = Some(speed_result.status_message);
                        } else if speed_result.download_speed_bps > 0.0 {
                            // Format the full result with proper units
                            let mb_downloaded =
                                speed_result.file_size_bytes as f64 / (1024.0 * 1024.0);
                            let mbps = speed_result.download_speed_bps / 1_000_000.0;

                            app_state.result = Some(format!(
                                "Download speed: {:.2} Mbps\n\n\
                                 Downloaded: {:.2} MB in {:.2} sec\n\
                                 {}",
                                mbps,
                                mb_downloaded,
                                speed_result.duration_secs,
                                speed_result.status_message
                            ));
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // When the sender is dropped, reset the mode.
                        if app_state.result.is_none()
                            || app_state.result.as_ref().unwrap().contains("Starting")
                        {
                            app_state.result =
                                Some("Speed test failed or was cancelled.".to_string());
                        }
                        app_state.speed_test_receiver = None;
                        break;
                    }
                }
            }
        }
    }
    cleanup_resources(&mut app_state);

    Ok(())
}

// === Input Handling Functions ===

fn handle_normal_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match (code, &app_state.active_menu) {
        (KeyCode::Char('q'), _) => {
            cleanup_resources(app_state);
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
                    60, // Keep 60 data points for history (1 minute at 1 second refresh)
                    std::time::Duration::from_millis(1000), // Refresh every second
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
            app_state.system_view_mode = SystemViewMode::Overview;
        }
        (KeyCode::Char('p'), MenuItem::SystemUtilities) => {
            app_state.selected_system_tool = Some("process_manager".to_string());
            app_state.system_view_mode = SystemViewMode::ProcessList;
        }
        (KeyCode::Char('d'), MenuItem::SystemUtilities) => {
            app_state.selected_system_tool = Some("disk_analyzer".to_string());
            app_state.system_view_mode = SystemViewMode::DiskDetails;
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
                let _ = tx.send(network_tools::SpeedTestResult::status(
                    "Connecting to speed test servers...",
                ));

                // Wait briefly to show the message
                thread::sleep(Duration::from_millis(500));

                // Try single file test first
                match network_tools::measure_speed() {
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
                            match network_tools::parallel_speed_test() {
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

fn handle_editing_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
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
                app_state.input_mode = InputMode::Normal;
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

fn handle_viewing_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app_state.input_mode = InputMode::Normal;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
    Ok(())
}

fn handle_email_config_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
        }
        KeyCode::Char('t') if app_state.email_config_field == 99 => {
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(scheduler) = task_scheduler.lock() {
                    match scheduler.test_email_config() {
                        Ok(_) => {
                            app_state.status_message = Some(prepare_status_message(
                                "Test email sent successfully!",
                                StatusMessageType::Success,
                                5,
                            ));
                        }
                        Err(e) => {
                            app_state.status_message = Some(prepare_status_message(
                                &format!("Failed to send test email: {}", e),
                                StatusMessageType::Error,
                                10, // Longer display time for errors
                            ));
                        }
                    }
                }
            }
        }
        KeyCode::Tab => {
            app_state.email_config_field = (app_state.email_config_field + 1) % 5;
        }
        KeyCode::BackTab => {
            app_state.email_config_field = (app_state.email_config_field + 4) % 5;
        }
        KeyCode::Char(c) => match app_state.email_config_field {
            0 => app_state.email_address.push(c),
            1 => app_state.email_smtp_server.push(c),
            2 => app_state.email_smtp_port.push(c),
            3 => app_state.email_username.push(c),
            4 => app_state.email_password.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app_state.email_config_field {
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
        },
        KeyCode::Enter => {
            println!("Saving email config...");
            // Validate and save email config
            if app_state.email_address.is_empty()
                || app_state.email_smtp_server.is_empty()
                || app_state.email_smtp_port.is_empty()
                || app_state.email_username.is_empty()
                || app_state.email_password.is_empty()
            {
                println!("Email config validation failed - empty fields");
                app_state.status_message = Some(prepare_status_message(
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
                    app_state.status_message = Some(prepare_status_message(
                        "Invalid port number",
                        StatusMessageType::Error,
                        3,
                    ));
                    return Ok(());
                }
            };

            // Create config
            let config = EmailConfig {
                email: app_state.email_address.clone(),
                smtp_server: app_state.email_smtp_server.clone(),
                smtp_port: port,
                username: app_state.email_username.clone(),
                password: app_state.email_password.clone(),
            };

            // Set config
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(mut scheduler) = task_scheduler.lock() {
                    scheduler.set_email_config(config);
                    app_state.status_message = Some(prepare_status_message(
                        "Email configuration saved",
                        StatusMessageType::Success,
                        3,
                    ));
                    app_state.input_mode = InputMode::Normal;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_adding_reminder_mode(
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
            app_state.input_field = (app_state.input_field + 1) % 2;
        }
        KeyCode::BackTab => {
            app_state.input_field = (app_state.input_field + 1) % 2;
        }
        KeyCode::Char(c) => match app_state.input_field {
            0 => app_state.reminder_date.push(c),
            1 => app_state.reminder_time.push(c),
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
        KeyCode::Up | KeyCode::Down => {
            // Cycle through reminder types
            app_state.reminder_type = match app_state.reminder_type {
                ReminderType::Email => ReminderType::Notification,
                ReminderType::Notification => ReminderType::Both,
                ReminderType::Both => ReminderType::Email,
            };
        }
        KeyCode::Enter => {
            if let Some(task_id) = app_state.selected_task_id {
                // Parse date and time
                let reminder_datetime =
                    format!("{} {}", app_state.reminder_date, app_state.reminder_time);

                if let Ok(dt) = NaiveDateTime::parse_from_str(
                    &format!("{} 00", reminder_datetime),
                    "%Y-%m-%d %H:%M %S",
                ) {
                    let timestamp = dt.timestamp();

                    // Add reminder
                    if let Some(ref task_scheduler) = app_state.task_scheduler {
                        if let Ok(mut scheduler) = task_scheduler.lock() {
                            if let Err(e) = scheduler.add_reminder_to_task(
                                task_id,
                                timestamp,
                                app_state.reminder_type.clone(),
                            ) {
                                app_state.status_message = Some(prepare_status_message(
                                    &format!("Error: {}", e),
                                    StatusMessageType::Error,
                                    3,
                                ));
                            } else {
                                app_state.status_message = Some(prepare_status_message(
                                    "Reminder added successfully",
                                    StatusMessageType::Success,
                                    3,
                                ));
                                app_state.input_mode = InputMode::ViewingTasks;
                            }
                        }
                    }
                } else {
                    app_state.status_message = Some(prepare_status_message(
                        "Invalid date or time format",
                        StatusMessageType::Error,
                        3,
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_enter_address_mode(
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

fn handle_view_results_mode(
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

fn handle_adding_task_mode(
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
            app_state.task_priority = match app_state.task_priority {
                TaskPriority::Low => TaskPriority::Urgent,
                TaskPriority::Medium => TaskPriority::Low,
                TaskPriority::High => TaskPriority::Medium,
                TaskPriority::Urgent => TaskPriority::High,
            };
        }
        KeyCode::Down => {
            // Cycle through priorities
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

fn handle_viewing_tasks_mode(
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
        KeyCode::Char('d') => {
            // Delete selected task
            if let Some(task_id) = app_state.selected_task_id {
                if let Some(ref task_scheduler) = app_state.task_scheduler {
                    if let Ok(mut scheduler) = task_scheduler.lock() {
                        if let Err(e) = scheduler.delete_task(task_id) {
                            app_state.status_message = Some(prepare_status_message(
                                &format!("Error: {}", e),
                                StatusMessageType::Error,
                                3,
                            ));
                        } else {
                            app_state.status_message = Some(prepare_status_message(
                                "Task deleted successfully",
                                StatusMessageType::Success,
                                3,
                            ));
                            app_state.selected_task_id = None;
                        }
                    }
                }
            }
        }
        KeyCode::Char('c') => {
            // Complete selected task
            if let Some(task_id) = app_state.selected_task_id {
                if let Some(ref task_scheduler) = app_state.task_scheduler {
                    if let Ok(mut scheduler) = task_scheduler.lock() {
                        if let Some(task) = scheduler.get_task(task_id) {
                            let mut updated_task = task.clone();
                            updated_task.status = TaskStatus::Completed;

                            if let Err(e) = scheduler.update_task(task_id, updated_task) {
                                app_state.status_message = Some(prepare_status_message(
                                    &format!("Error: {}", e),
                                    StatusMessageType::Error,
                                    3,
                                ));
                            } else {
                                app_state.status_message = Some(prepare_status_message(
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
        KeyCode::Up => {
            // Navigate tasks up
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(scheduler) = task_scheduler.lock() {
                    let tasks = scheduler.get_all_tasks();
                    if tasks.is_empty() {
                        return Ok(());
                    }

                    // Sort tasks by due date
                    let mut task_ids: Vec<u32> = tasks.iter().map(|task| task.id).collect();
                    task_ids.sort_by(|a, b| {
                        let task_a = scheduler.get_task(*a).unwrap();
                        let task_b = scheduler.get_task(*b).unwrap();
                        task_a.due_date.cmp(&task_b.due_date)
                    });

                    if let Some(selected_id) = app_state.selected_task_id {
                        if let Some(pos) = task_ids.iter().position(|id| *id == selected_id) {
                            if pos > 0 {
                                app_state.selected_task_id = Some(task_ids[pos - 1]);
                            }
                        }
                    } else if !task_ids.is_empty() {
                        app_state.selected_task_id = Some(task_ids[0]);
                    }
                }
            }
        }
        KeyCode::Down => {
            // Navigate tasks down
            if let Some(ref task_scheduler) = app_state.task_scheduler {
                if let Ok(scheduler) = task_scheduler.lock() {
                    let tasks = scheduler.get_all_tasks();
                    if tasks.is_empty() {
                        return Ok(());
                    }

                    // Sort tasks by due date
                    let mut task_ids: Vec<u32> = tasks.iter().map(|task| task.id).collect();
                    task_ids.sort_by(|a, b| {
                        let task_a = scheduler.get_task(*a).unwrap();
                        let task_b = scheduler.get_task(*b).unwrap();
                        task_a.due_date.cmp(&task_b.due_date)
                    });

                    if let Some(selected_id) = app_state.selected_task_id {
                        if let Some(pos) = task_ids.iter().position(|id| *id == selected_id) {
                            if pos < task_ids.len() - 1 {
                                app_state.selected_task_id = Some(task_ids[pos + 1]);
                            }
                        }
                    } else if !task_ids.is_empty() {
                        app_state.selected_task_id = Some(task_ids[0]);
                    }
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

fn draw_task_scheduler_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("a. Add Task")),
        Spans::from(Span::raw("v. View Tasks")),
        Spans::from(Span::raw("e. Email Configuration")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Task Scheduler")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(text_color));
    f.render_widget(paragraph, chunks[0]);
}

fn draw_add_task<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    let priority_display = format!("{:?}", app_state.task_priority);

    let fields = [
        ("Title: ", &app_state.task_title, app_state.input_field == 0),
        (
            "Description: ",
            &app_state.task_description,
            app_state.input_field == 1,
        ),
        (
            "Due Date (YYYY-MM-DD): ",
            &app_state.task_due_date,
            app_state.input_field == 2,
        ),
        (
            "Tags (comma-separated): ",
            &app_state.task_tags,
            app_state.input_field == 3,
        ),
    ];

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                (*value).clone(),
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);

        // Highlight the entire block if this field is selected
        let block_style = if *is_selected {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, layout[i + 1]);
    }

    // Similarly update your priority display with clearer highlighting
    // Add a blinking cursor or indicator for the active field
    if app_state.input_field < 4 {
        let field_layout = layout[app_state.input_field + 1];
        let cursor_x =
            fields[app_state.input_field].0.len() + fields[app_state.input_field].1.len();

        // You could render a cursor indicator here
        if cursor_x < field_layout.width as usize {
            let cursor_rect = Rect::new(
                field_layout.x + cursor_x as u16 + 1, // +1 for border
                field_layout.y + 1,                   // +1 for border
                1,
                1,
            );
            f.render_widget(
                Block::default().style(Style::default().bg(Color::White)),
                cursor_rect,
            );
        }
    }

    // Priority field for the add task screen
    let priority_text = Spans::from(vec![
        Span::raw("Priority: "),
        Span::styled(
            priority_display,
            Style::default().fg(get_priority_color(&app_state.task_priority)),
        ),
        Span::raw(" (Use ↑↓ to change)"),
    ]);
    let priority_paragraph =
        Paragraph::new(priority_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(priority_paragraph, layout[5]);

    // Instructions
    let instructions = Paragraph::new("Press 'Enter' to Save, 'Esc' to Cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[6]);

    // Display status message if present
    if let Some(ref msg) = app_state.status_message {
        let status_block = Paragraph::new(msg.message.clone())
            .style(Style::default().fg(match msg.message_type {
                StatusMessageType::Info => Color::Blue,
                StatusMessageType::Success => Color::Green,
                StatusMessageType::Warning => Color::Yellow,
                StatusMessageType::Error => Color::Red,
            }))
            .block(Block::default().borders(Borders::ALL).title("Status"));

        // Create a centered popup for the status message
        let area = f.size();
        let status_width = 60.min(area.width.saturating_sub(4));
        let status_height = 5.min(area.height.saturating_sub(4));

        let status_area = Rect::new(
            ((area.width - status_width) / 2).max(0),
            ((area.height - status_height) / 2).max(0),
            status_width,
            status_height,
        );

        f.render_widget(Clear, status_area); // Clear the area
        f.render_widget(status_block, status_area);
    }
}

fn draw_view_tasks<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create layout with a status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Task table
            Constraint::Length(3), // Task details
            Constraint::Length(1), // Status message
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![Span::styled(
            "TASK MANAGER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(
            "Press ↑↓ to navigate, 'r' to add reminder, 'c' to complete, 'd' to delete, 'Esc' to go back",
        )]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Task list
    if let Some(ref task_scheduler) = app_state.task_scheduler {
        if let Ok(scheduler) = task_scheduler.lock() {
            let all_tasks = scheduler.get_all_tasks();

            if all_tasks.is_empty() {
                let no_tasks = Paragraph::new("No tasks found. Press 'a' to add a new task.")
                    .style(Style::default().fg(text_color))
                    .block(Block::default().borders(Borders::ALL).title("Tasks"));
                f.render_widget(no_tasks, chunks[1]);
            } else {
                // Sort tasks by due date
                let mut sorted_tasks = all_tasks.clone();
                sorted_tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));

                // Table headers
                let header_cells = vec![
                    Cell::from("ID").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Title").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Due Date").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Priority").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Status").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Reminders").style(Style::default().fg(Color::Yellow)),
                ];

                let header = Row::new(header_cells)
                    .style(Style::default().add_modifier(Modifier::BOLD))
                    .height(1);

                // Task rows
                let rows = sorted_tasks.iter().map(|task| {
                    let id = task.id.to_string();
                    let title = task.title.clone();
                    let due_date = task_scheduler::format_timestamp(task.due_date);
                    let priority = format!("{:?}", task.priority);
                    let status = format!("{:?}", task.status);
                    let reminders = task.reminders.len().to_string();

                    // Calculate color based on due date
                    let now = Utc::now().timestamp();
                    let row_color = if task.status == TaskStatus::Completed {
                        Color::DarkGray
                    } else if task.due_date < now {
                        Color::Red
                    } else if task.due_date - now < 86400 {
                        // Within 24 hours
                        Color::Yellow
                    } else {
                        text_color
                    };

                    Row::new(vec![
                        Cell::from(id),
                        Cell::from(title),
                        Cell::from(due_date),
                        Cell::from(priority)
                            .style(Style::default().fg(get_priority_color(&task.priority))),
                        Cell::from(status)
                            .style(Style::default().fg(get_status_color(&task.status))),
                        Cell::from(reminders),
                    ])
                    .style(Style::default().fg(row_color))
                });

                // Create a stateful table
                let mut state = tui::widgets::TableState::default();

                // Find the selected task index
                let selected_index = if let Some(selected_id) = app_state.selected_task_id {
                    sorted_tasks.iter().position(|task| task.id == selected_id)
                } else {
                    None
                };

                // Set the selected index
                if let Some(idx) = selected_index {
                    state.select(Some(idx));
                } else if !sorted_tasks.is_empty() {
                    state.select(Some(0));
                }

                let table = Table::new(rows)
                    .header(header)
                    .block(Block::default().title("Tasks").borders(Borders::ALL))
                    .widths(&[
                        Constraint::Length(5),
                        Constraint::Percentage(35),
                        Constraint::Length(19),
                        Constraint::Length(10),
                        Constraint::Length(12),
                        Constraint::Length(10),
                    ])
                    .column_spacing(1)
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );

                // Render stateful table
                f.render_stateful_widget(table, chunks[1], &mut state);

                // Show task details if a task is selected
                if let Some(selected_id) = app_state.selected_task_id {
                    if let Some(task) = scheduler.get_task(selected_id) {
                        let details = vec![
                            Spans::from(vec![
                                Span::styled(
                                    "Title: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(&task.title),
                            ]),
                            Spans::from(vec![
                                Span::styled(
                                    "Description: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(&task.description),
                            ]),
                            Spans::from(vec![
                                Span::styled(
                                    "Tags: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(task.tags.join(", ")),
                            ]),
                        ];

                        let details_block = Paragraph::new(details)
                            .block(Block::default().title("Task Details").borders(Borders::ALL));
                        f.render_widget(details_block, chunks[2]);
                    }
                }
            }
        }
    } else {
        // If no scheduler is available
        let no_data = Paragraph::new("Task scheduler not initialized")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }

    // Status message
    if let Some(ref status) = app_state.status_message {
        let message_color = match status.message_type {
            StatusMessageType::Info => Color::Blue,
            StatusMessageType::Success => Color::Green,
            StatusMessageType::Warning => Color::Yellow,
            StatusMessageType::Error => Color::Red,
        };

        let status_text = Spans::from(vec![
            Span::styled("◆ ", Style::default().fg(message_color)),
            Span::styled(&status.message, Style::default().fg(message_color)),
        ]);

        let status_bar = Paragraph::new(status_text);
        f.render_widget(status_bar, chunks[3]);
    }

    // Controls
    let controls = Paragraph::new(vec![Spans::from(vec![Span::raw(
        "Actions: [r]Add Reminder [c]Complete [d]Delete | [↑↓]Navigate",
    )])])
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(controls, chunks[4]);
}

fn draw_email_config<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .bg(Color::DarkGray) // More visible background
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    let fields = [
        (
            "Email Address: ",
            &app_state.email_address,
            app_state.email_config_field == 0,
        ),
        (
            "SMTP Server: ",
            &app_state.email_smtp_server,
            app_state.email_config_field == 1,
        ),
        (
            "SMTP Port: ",
            &app_state.email_smtp_port,
            app_state.email_config_field == 2,
        ),
        (
            "Username: ",
            &app_state.email_username,
            app_state.email_config_field == 3,
        ),
        (
            "Password: ",
            &app_state.email_password,
            app_state.email_config_field == 4,
        ),
    ];

    // Title
    let title = Paragraph::new("Email Configuration")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let display_value = if i == 4 && !value.is_empty() {
            "*".repeat(value.len()) // Mask password
        } else {
            value.to_string()
        };

        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                display_value,
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);

        // Highlight the entire block if this field is selected
        let block_style = if *is_selected {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).style(block_style));

        f.render_widget(paragraph, layout[i + 1]);
    }

    // Add instructions
    let instructions = Paragraph::new(vec![
        Spans::from("Press 'Enter' to Save, 'Esc' to Cancel"),
        Spans::from("Press 't' to Test Configuration"),
    ])
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(instructions, layout[6]);

    // Optional: Add a blinking cursor for better visual feedback
    if app_state.email_config_field < 5 {
        let field_layout = layout[app_state.email_config_field + 1];
        let label_len = fields[app_state.email_config_field].0.len();
        let value_len = if app_state.email_config_field == 4 {
            // For password field
            "*".repeat(fields[app_state.email_config_field].1.len())
                .len()
        } else {
            fields[app_state.email_config_field].1.len()
        };

        // You can add a blinking cursor here if needed
    }
}

fn draw_add_reminder<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default().fg(Color::Yellow).bg(Color::Blue);
    let normal_style = Style::default().fg(Color::White);

    // Title with selected task info
    let mut title_text = "Add Reminder".to_string();
    if let Some(task_id) = app_state.selected_task_id {
        if let Some(ref task_scheduler) = app_state.task_scheduler {
            if let Ok(scheduler) = task_scheduler.lock() {
                if let Some(task) = scheduler.get_task(task_id) {
                    title_text = format!("Add Reminder for: {}", task.title);
                }
            }
        }
    }

    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    let reminder_type_display = format!("{:?}", app_state.reminder_type);

    let fields = [
        (
            "Date (YYYY-MM-DD): ",
            &app_state.reminder_date,
            app_state.input_field == 0,
        ),
        (
            "Time (HH:MM): ",
            &app_state.reminder_time,
            app_state.input_field == 1,
        ),
    ];

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                (*value).clone(),
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, layout[i + 1]);
    }

    // Reminder type selection
    let reminder_type_text = Spans::from(vec![
        Span::raw("Reminder Type: "),
        Span::styled(reminder_type_display, Style::default().fg(Color::Cyan)),
        Span::raw(" (Use ↑↓ to change)"),
    ]);
    let reminder_type_paragraph =
        Paragraph::new(reminder_type_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(reminder_type_paragraph, layout[3]);

    // Instructions
    let instructions = Paragraph::new("Press 'Enter' to Save, 'Esc' to Cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[4]);
}

// Helper function for priority colors
fn get_priority_color(priority: &TaskPriority) -> Color {
    match priority {
        TaskPriority::Low => Color::Blue,
        TaskPriority::Medium => Color::Green,
        TaskPriority::High => Color::Yellow,
        TaskPriority::Urgent => Color::Red,
    }
}

// Helper function for status colors
fn get_status_color(status: &TaskStatus) -> Color {
    match status {
        TaskStatus::Pending => Color::Yellow,
        TaskStatus::InProgress => Color::Blue,
        TaskStatus::Completed => Color::Green,
        TaskStatus::Cancelled => Color::Gray,
    }
}

fn handle_speed_test_running_mode(
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

fn handle_system_utilities_mode(
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
                app_state.active_menu = MenuItem::Main;
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
                initialize_process_selection(app_state);
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
                // Sort by memory in Process Manager
                app_state.process_sort_type = ProcessSortType::MemoryUsage;
                app_state.status_message = Some(prepare_status_message(
                    "Sorted by memory usage",
                    StatusMessageType::Info,
                    2,
                ));
            }
        }
        KeyCode::Char('o') => {
            // Overview in resource monitor
            if app_state.selected_system_tool == Some("resource_monitor".to_string()) {
                app_state.system_view_mode = SystemViewMode::Overview;
            }
        }
        KeyCode::Char('n') => {
            // Sort by name
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                app_state.process_sort_type = ProcessSortType::Name;
                app_state.status_message = Some(prepare_status_message(
                    "Sorted by process name",
                    StatusMessageType::Info,
                    2,
                ));
            }
        }
        KeyCode::Char('t') => {
            // Sort by runtime
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                app_state.process_sort_type = ProcessSortType::Runtime;
                app_state.status_message = Some(prepare_status_message(
                    "Sorted by runtime",
                    StatusMessageType::Info,
                    2,
                ));
            }
        }
        KeyCode::Char('k') => {
            // Show kill confirmation for selected process
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                if let Some(ref snapshot) = app_state.system_snapshot {
                    if let Some(selected_pid) = app_state.selected_process_pid {
                        // Find the process in the snapshot
                        if let Some(process) = snapshot
                            .top_processes
                            .iter()
                            .find(|p| p.pid == selected_pid)
                        {
                            let pid = process.pid;
                            let name = process.name.clone();

                            // Show confirmation dialog - ensure we use the same name!
                            app_state.confirmation_dialogue =
                                ConfirmationDialogue::KillProcess(pid, name);
                        }
                    }
                }
            }
        }
        // Add navigation for process list
        KeyCode::Up => {
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                if let Some(ref snapshot) = app_state.system_snapshot {
                    if snapshot.top_processes.is_empty() {
                        return Ok(());
                    }

                    // Get a sorted copy of processes
                    let mut sorted_processes = snapshot.top_processes.clone();
                    match app_state.process_sort_type {
                        ProcessSortType::Pid => {
                            sorted_processes.sort_by_key(|p| p.pid);
                        }
                        ProcessSortType::Name => {
                            sorted_processes
                                .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                        }
                        ProcessSortType::CpuUsage => {
                            sorted_processes.sort_by(|a, b| {
                                b.cpu_usage
                                    .partial_cmp(&a.cpu_usage)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }
                        ProcessSortType::MemoryUsage => {
                            sorted_processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
                        }
                        ProcessSortType::Runtime => {
                            sorted_processes.sort_by(|a, b| b.run_time.cmp(&a.run_time));
                        }
                    }

                    if let Some(selected_pid) = app_state.selected_process_pid {
                        // Find the current process in the sorted list
                        if let Some(current_idx) =
                            sorted_processes.iter().position(|p| p.pid == selected_pid)
                        {
                            if current_idx > 0 {
                                // Select the previous process
                                app_state.selected_process_pid =
                                    Some(sorted_processes[current_idx - 1].pid);
                            }
                        }
                    } else if !sorted_processes.is_empty() {
                        // Select the first process if none is selected
                        app_state.selected_process_pid = Some(sorted_processes[0].pid);
                    }
                }
            }
        }
        KeyCode::Down => {
            if app_state.selected_system_tool == Some("process_manager".to_string()) {
                if let Some(ref snapshot) = app_state.system_snapshot {
                    if snapshot.top_processes.is_empty() {
                        return Ok(());
                    }

                    // Get a sorted copy of processes
                    let mut sorted_processes = snapshot.top_processes.clone();
                    match app_state.process_sort_type {
                        ProcessSortType::Pid => {
                            sorted_processes.sort_by_key(|p| p.pid);
                        }
                        ProcessSortType::Name => {
                            sorted_processes
                                .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                        }
                        ProcessSortType::CpuUsage => {
                            sorted_processes.sort_by(|a, b| {
                                b.cpu_usage
                                    .partial_cmp(&a.cpu_usage)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }
                        ProcessSortType::MemoryUsage => {
                            sorted_processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
                        }
                        ProcessSortType::Runtime => {
                            sorted_processes.sort_by(|a, b| b.run_time.cmp(&a.run_time));
                        }
                    }

                    if let Some(selected_pid) = app_state.selected_process_pid {
                        // Find the current process in the sorted list
                        if let Some(current_idx) =
                            sorted_processes.iter().position(|p| p.pid == selected_pid)
                        {
                            if current_idx < sorted_processes.len() - 1 {
                                // Select the next process
                                app_state.selected_process_pid =
                                    Some(sorted_processes[current_idx + 1].pid);
                            }
                        }
                    } else if !sorted_processes.is_empty() {
                        // Select the first process if none is selected
                        app_state.selected_process_pid = Some(sorted_processes[0].pid);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// === Drawing Functions ===

static DARK_MODE: Lazy<bool> = Lazy::new(|| {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(
            "tell application \"System Events\" to tell appearance preferences to return dark mode",
        )
        .output()
        .unwrap_or_else(|_| {
            eprintln!("Failed to execute osascript");
            std::process::Output {
                status: std::process::ExitStatus::from_raw(0),
                stdout: b"false".to_vec(),
                stderr: Vec::new(),
            }
        });
    String::from_utf8_lossy(&output.stdout).trim() == "true"
});

fn get_text_color() -> tui::style::Color {
    if *DARK_MODE {
        tui::style::Color::White
    } else {
        tui::style::Color::Black
    }
}

fn is_dark_mode() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(
            "tell application \"System Events\" to tell appearance preferences to return dark mode",
        )
        .output()
        .expect("Failed to execute osascript");

    let result = String::from_utf8_lossy(&output.stdout);
    result.trim() == "true"
}

fn draw_main_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("1. Password Manager")),
        Spans::from(Span::raw("2. Network Tools")),
        Spans::from(Span::raw("3. System Utilities")),
        Spans::from(Span::raw("4. Task Manager")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Main Menu").borders(Borders::ALL))
        .style(Style::default().fg(text_color)); // Use dynamic text color
    f.render_widget(paragraph, chunks[0]);
}

fn draw_password_manager_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("a. Add Password")),
        Spans::from(Span::raw("v. View Passwords")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Password Manager")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(text_color)); // Set text color to black
    f.render_widget(paragraph, chunks[0]);
}

fn draw_input_modal<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default().fg(Color::Yellow).bg(Color::Blue);
    let normal_style = Style::default().fg(Color::White);

    let fields = [
        ("Service: ", &app_state.service, app_state.input_field == 0),
        (
            "Username: ",
            &app_state.username,
            app_state.input_field == 1,
        ),
        (
            "Password: ",
            &app_state.password,
            app_state.input_field == 2,
        ),
    ];

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                (*value).clone(),
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, layout[i + 1]);
    }

    // Display error or success message if present
    if let Some(ref msg) = app_state.error_message {
        let status_block = Paragraph::new(msg.clone())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status_block, layout[5]); // Adjust index based on your layout
    }

    let instructions = Paragraph::new("Press 'Enter' to Save, 'Esc' to Cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[4]);
}

fn draw_password_list<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    match password_manager::retrieve_password() {
        Ok(passwords) => {
            if passwords.is_empty() {
                let paragraph = Paragraph::new("No passwords found.")
                    .block(
                        Block::default()
                            .title("Stored Passwords")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::Black)); // Set text color to black
                f.render_widget(paragraph, f.size());
            } else {
                let text: Vec<Spans> = passwords
                    .iter()
                    .map(|entry| {
                        Spans::from(vec![
                            Span::raw(format!("Service: {}, ", entry.service)),
                            Span::raw(format!("Username: {}, ", entry.username)),
                            Span::raw(format!("Password: {}", entry.password)),
                        ])
                    })
                    .collect();

                let paragraph = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title("Stored Passwords")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(text_color)); // Set text color to black
                f.render_widget(paragraph, f.size());
            }
        }
        Err(e) => {
            let paragraph = Paragraph::new(format!("Error retrieving passwords: {:?}", e))
                .block(Block::default().title("Error").borders(Borders::ALL))
                .style(Style::default().fg(Color::Red)); // Keep error text red
            f.render_widget(paragraph, f.size());
        }
    }
}

fn draw_network_tools_menu<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("p. Ping")),
        Spans::from(Span::raw("t. Traceroute")),
        Spans::from(Span::raw("s. Speed Test")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Network Tools")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Black)); // Set text color to black
    f.render_widget(paragraph, chunks[0]);
}

fn draw_address_input<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let tool_name = app_state.selected_tool.as_deref().unwrap_or("Unknown Tool");
    let text = vec![
        Spans::from(Span::raw(format!("Enter address for {}:", tool_name))),
        Spans::from(Span::raw(&app_state.address)),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Address Input")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White)); // Remove bg setting
    f.render_widget(paragraph, chunks[0]);
}

fn draw_speed_test<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let display_text = if let Some(ref result) = app_state.result {
        result.clone()
    } else {
        "Initializing speed test...".to_string()
    };

    // Extract speed value to determine color
    let speed_color = if let Some(ref result) = app_state.result {
        if result.contains("Mbps") {
            // Try to extract the speed value
            if let Some(speed_str) =
                result
                    .lines()
                    .find(|line| line.contains("Mbps"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .find(|word| word.parse::<f64>().is_ok())
                    })
            {
                if let Ok(speed) = speed_str.parse::<f64>() {
                    // Color-code based on speed
                    if speed > 100.0 {
                        Color::Green // Fast connection
                    } else if speed > 25.0 {
                        Color::Yellow // Medium speed
                    } else {
                        Color::Red // Slow connection
                    }
                } else {
                    Color::White // Default if parsing fails
                }
            } else {
                Color::White // Default if no speed found
            }
        } else if result.contains("error") || result.contains("failed") {
            Color::Red // Error message
        } else {
            Color::White // Status message
        }
    } else {
        Color::White // Default when no result
    };

    // Format display text with instructions
    let display_text_with_instructions = format!("{}\n\nPress Esc to return to menu", display_text);

    let paragraph = Paragraph::new(display_text_with_instructions)
        .block(
            Block::default()
                .title("Network Speed Test")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(speed_color));

    f.render_widget(paragraph, f.size());

    // If no receiver but we have a result, we're done
    if app_state.speed_test_receiver.is_none() && app_state.result.is_some() {
        // Add a footer with more instructions
        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        let footer_text = "Test complete. Press Esc to return to the menu";
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(footer, footer_layout[1]);
    }
}

fn draw_confirmation_dialogue<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    if let ConfirmationDialogue::KillProcess(pid, ref name) = &app_state.confirmation_dialogue {
        // Create a centered box for the dialog
        let area = f.size();
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 10.min(area.height.saturating_sub(4));

        let dialog_area = Rect::new(
            ((area.width - dialog_width) / 2).max(0),
            ((area.height - dialog_height) / 2).max(0),
            dialog_width,
            dialog_height,
        );

        // Draw dialog box
        let dialog = Block::default()
            .title("Confirm Process Termination")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        f.render_widget(Clear, dialog_area); // Clear the area
        f.render_widget(dialog, dialog_area);

        // Dialog content
        let content_area = Rect::new(
            dialog_area.x + 2,
            dialog_area.y + 2,
            dialog_area.width.saturating_sub(4),
            dialog_area.height.saturating_sub(4),
        );

        let message = vec![
            Spans::from(vec![
                Span::raw("Are you sure you want to terminate process "),
                Span::styled(name.clone(), Style::default().fg(Color::Yellow)),
                Span::raw(" (PID: "),
                Span::styled(pid.to_string(), Style::default().fg(Color::Yellow)),
                Span::raw(")?"),
            ]),
            Spans::from(Span::raw("")),
            Spans::from(Span::styled(
                "This action cannot be undone.",
                Style::default().fg(Color::Red),
            )),
            Spans::from(Span::raw("")),
            Spans::from(vec![
                Span::styled("y", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Yes, terminate the process"),
            ]),
            Spans::from(vec![
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - No, cancel"),
            ]),
        ];

        let content = Paragraph::new(message).alignment(tui::layout::Alignment::Center);

        f.render_widget(content, content_area);
    }
}

fn draw_view_results<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    if let Some(ref result) = app_state.result {
        // Deserialize the PingResult
        if let Ok(ping_result) = serde_json::from_str::<PingResult>(result) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let mut rows = vec![
                Row::new(vec![
                    Cell::from("Packets Transmitted"),
                    Cell::from(ping_result.packets_transmitted.to_string()),
                ]),
                Row::new(vec![
                    Cell::from("Packets Received"),
                    Cell::from(ping_result.packets_received.to_string()),
                ]),
                Row::new(vec![
                    Cell::from("Packet Loss"),
                    Cell::from(format!("{}%", ping_result.packet_loss)),
                ]),
            ];

            if let Some(time) = ping_result.time {
                rows.push(Row::new(vec![
                    Cell::from("Total Time"),
                    Cell::from(format!("{} ms", time)),
                ]));
            }

            rows.extend(vec![
                Row::new(vec![
                    Cell::from("Round-Trip Min"),
                    Cell::from(format!("{} ms", ping_result.round_trip_min)),
                ]),
                Row::new(vec![
                    Cell::from("Round-Trip Avg"),
                    Cell::from(format!("{} ms", ping_result.round_trip_avg)),
                ]),
                Row::new(vec![
                    Cell::from("Round-Trip Max"),
                    Cell::from(format!("{} ms", ping_result.round_trip_max)),
                ]),
                Row::new(vec![
                    Cell::from("Round-Trip Mdev"),
                    Cell::from(format!("{} ms", ping_result.round_trip_mdev)),
                ]),
            ]);

            let table = Table::new(rows)
                .header(Row::new(vec!["Metric", "Value"]).style(Style::default().fg(Color::Yellow)))
                .block(Block::default().title("Ping Results").borders(Borders::ALL))
                .widths(&[Constraint::Length(20), Constraint::Length(20)]);

            f.render_widget(table, chunks[0]);
        } else {
            // If deserialization fails, display the raw output
            let paragraph = Paragraph::new(result.clone())
                .block(Block::default().title("Results").borders(Borders::ALL))
                .style(Style::default().fg(Color::White)); // Remove bg setting
            f.render_widget(paragraph, f.size());
        }
    } else {
        let paragraph = Paragraph::new("No results available.")
            .block(Block::default().title("Results").borders(Borders::ALL))
            .style(Style::default().fg(Color::White)); // Remove bg setting
        f.render_widget(paragraph, f.size());
    }
}

fn draw_system_utilities_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("r. Resource Monitor")),
        Spans::from(Span::raw("p. Process Manager")),
        Spans::from(Span::raw("d. Disk Space Analyzer")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("System Utilities")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(text_color));

    f.render_widget(paragraph, chunks[0]);
}

// Add a function to draw the resource monitor
fn draw_resource_monitor<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create a layout for different sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(9), // CPU
            Constraint::Length(9), // Memory
            Constraint::Min(10),   // Processes
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![
            Span::styled("SYSTEM RESOURCE MONITOR", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::raw("Press 'c' for CPU details, 'm' for Memory details, 'p' for Process list, 'Esc' to go back"),
        ]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // If we have system data, show it
    if let Some(ref snapshot) = app_state.system_snapshot {
        // CPU Usage section
        let cpu_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // CPU usage gauge
        let cpu_percent = snapshot.cpu_usage as u16;
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU Usage").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(get_usage_color(cpu_percent as f32))
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(cpu_percent);

        f.render_widget(cpu_gauge, cpu_layout[0]);

        // CPU info
        let cpu_info = Paragraph::new(vec![
            Spans::from(vec![Span::raw(format!(
                "Cores: {}",
                snapshot.cpu_cores_count
            ))]),
            Spans::from(vec![Span::raw(format!("Model: {}", snapshot.cpu_name))]),
        ])
        .block(Block::default().title("CPU Info").borders(Borders::ALL));

        f.render_widget(cpu_info, cpu_layout[1]);

        // Memory section
        let mem_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[2]);

        // Memory usage gauge
        let mem_percent = snapshot.memory_usage_percent as u16;
        let mem_gauge = Gauge::default()
            .block(Block::default().title("Memory Usage").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(get_usage_color(mem_percent as f32))
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(mem_percent);

        f.render_widget(mem_gauge, mem_layout[0]);

        // Memory info
        let mem_info = Paragraph::new(vec![
            Spans::from(vec![Span::raw(format!(
                "Used: {} MB",
                snapshot.memory_used / 1024 / 1024
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Total: {} MB",
                snapshot.memory_total / 1024 / 1024
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Swap Used: {} MB",
                snapshot.swap_used / 1024 / 1024
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Swap Total: {} MB",
                snapshot.swap_total / 1024 / 1024
            ))]),
        ])
        .block(Block::default().title("Memory Info").borders(Borders::ALL));

        f.render_widget(mem_info, mem_layout[1]);

        // Process list
        let process_chunk = chunks[3];
        draw_process_list(f, snapshot, process_chunk);
    } else {
        // If no snapshot is available
        let no_data = Paragraph::new("Loading system data...")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }
}

// Helper function to draw process list
fn draw_process_list<B: Backend>(
    f: &mut Frame<B>,
    snapshot: &system_utilities::SystemSnapshot,
    area: Rect,
) {
    // Table headers
    let header_cells = ["PID", "Name", "CPU %", "Memory", "Mem %", "Runtime"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    // Process rows
    let rows = snapshot.top_processes.iter().map(|process| {
        let pid = process.pid.to_string();
        let name = process.name.clone();
        let cpu = format!("{:.1}%", process.cpu_usage);
        let mem = format!("{} MB", process.memory_usage / 1024 / 1024);
        let mem_percent = format!("{:.1}%", process.memory_usage_percent);

        // Format runtime
        let hours = process.run_time / 3600;
        let minutes = (process.run_time % 3600) / 60;
        let seconds = process.run_time % 60;
        let runtime = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

        Row::new(vec![
            Cell::from(pid),
            Cell::from(name),
            Cell::from(cpu),
            Cell::from(mem),
            Cell::from(mem_percent),
            Cell::from(runtime),
        ])
    });

    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Top Processes")
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(10),
        ])
        .column_spacing(1);

    f.render_widget(table, area);
}

// Helper function to get color based on usage percentage
fn get_usage_color(usage: f32) -> Color {
    if usage < 60.0 {
        Color::Green
    } else if usage < 85.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

// Detailed process list view
fn draw_process_list_detailed<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create layout with a status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Process table
            Constraint::Length(1), // Status message
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![Span::styled(
            "PROCESS MANAGER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(
            "Press ↑↓ to navigate, 'k' to kill selected process, 'r' to refresh, 'Esc' to go back",
        )]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Process list
    if let Some(ref snapshot) = app_state.system_snapshot {
        // Table headers with highlighting for current sort
        let sort_highlight = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let normal_header = Style::default().fg(Color::Yellow);

        let pid_style = if app_state.process_sort_type == ProcessSortType::Pid {
            sort_highlight
        } else {
            normal_header
        };
        let name_style = if app_state.process_sort_type == ProcessSortType::Name {
            sort_highlight
        } else {
            normal_header
        };
        let cpu_style = if app_state.process_sort_type == ProcessSortType::CpuUsage {
            sort_highlight
        } else {
            normal_header
        };
        let mem_style = if app_state.process_sort_type == ProcessSortType::MemoryUsage {
            sort_highlight
        } else {
            normal_header
        };
        let runtime_style = if app_state.process_sort_type == ProcessSortType::Runtime {
            sort_highlight
        } else {
            normal_header
        };

        let header_cells = vec![
            Cell::from("PID").style(pid_style),
            Cell::from("Name").style(name_style),
            Cell::from("CPU %").style(cpu_style),
            Cell::from("Memory").style(mem_style),
            Cell::from("Mem %").style(mem_style),
            Cell::from("Disk I/O").style(normal_header),
            Cell::from("Start Time").style(normal_header),
            Cell::from("Runtime").style(runtime_style),
        ];

        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        // Get a sorted copy of processes
        let mut sorted_processes = snapshot.top_processes.clone();
        match app_state.process_sort_type {
            ProcessSortType::Pid => {
                sorted_processes.sort_by_key(|p| p.pid);
            }
            ProcessSortType::Name => {
                sorted_processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            ProcessSortType::CpuUsage => {
                sorted_processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProcessSortType::MemoryUsage => {
                sorted_processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
            }
            ProcessSortType::Runtime => {
                sorted_processes.sort_by(|a, b| b.run_time.cmp(&a.run_time));
            }
        }

        // Process rows
        let rows = sorted_processes.iter().map(|process| {
            let pid = process.pid.to_string();
            let name = process.name.clone();
            let cpu = format!("{:.1}%", process.cpu_usage);
            let mem = format!("{} MB", process.memory_usage / 1024 / 1024);
            let mem_percent = format!("{:.1}%", process.memory_usage_percent);
            let disk_io = format!("{} KB", process.disk_usage / 1024);

            // Format start time
            let start_datetime = chrono::DateTime::from_timestamp(process.start_time as i64, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Format runtime
            let hours = process.run_time / 3600;
            let minutes = (process.run_time % 3600) / 60;
            let seconds = process.run_time % 60;
            let runtime = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

            Row::new(vec![
                Cell::from(pid),
                Cell::from(name),
                Cell::from(cpu),
                Cell::from(mem),
                Cell::from(mem_percent),
                Cell::from(disk_io),
                Cell::from(start_datetime),
                Cell::from(runtime),
            ])
        });

        // Create a stateful table
        let mut state = tui::widgets::TableState::default();

        // Find the currently selected process by PID
        let selected_index = if let Some(selected_pid) = app_state.selected_process_pid {
            sorted_processes.iter().position(|p| p.pid == selected_pid)
        } else {
            None
        };

        // Set the selected index
        if let Some(idx) = selected_index {
            state.select(Some(idx));
        } else if !sorted_processes.is_empty() {
            // Default to first item if nothing is selected, but we can't update app_state here
            state.select(Some(0));
            // We can't do this here since app_state is borrowed immutably:
            // app_state.selected_process_pid = Some(sorted_processes[0].pid);
        }

        let table = Table::new(rows)
            .header(header)
            .block(Block::default().title("Processes").borders(Borders::ALL))
            .widths(&[
                Constraint::Length(8),
                Constraint::Percentage(25),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ])
            .column_spacing(1)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        // Render stateful table
        f.render_stateful_widget(table, chunks[1], &mut state);
    } else {
        // If no snapshot is available
        let no_data = Paragraph::new("Loading process data...")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }

    // Status message (between table and controls)
    if let Some(ref status) = app_state.status_message {
        let message_color = match status.message_type {
            StatusMessageType::Info => Color::Blue,
            StatusMessageType::Success => Color::Green,
            StatusMessageType::Warning => Color::Yellow,
            StatusMessageType::Error => Color::Red,
        };

        let status_text = Spans::from(vec![
            Span::styled("◆ ", Style::default().fg(message_color)),
            Span::styled(&status.message, Style::default().fg(message_color)),
        ]);

        let status_bar = Paragraph::new(status_text);
        f.render_widget(status_bar, chunks[2]);
    }

    // Controls (now at index 3)
    let controls = Paragraph::new(vec![Spans::from(vec![Span::raw(
        "Sort: [p]PID [n]Name [c]CPU [m]Memory [t]Time | [r]Refresh [k]Kill [↑↓]Navigate",
    )])])
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(controls, chunks[3]);

    // Draw the confirmation dialog if active
    if app_state.confirmation_dialogue != ConfirmationDialogue::None {
        draw_confirmation_dialogue(f, app_state);
    }
}

// Disk space analyzer view
fn draw_disk_analyzer<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Disk info
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![Span::styled(
            "DISK SPACE ANALYZER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw("Press 'r' to refresh, 'Esc' to go back")]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Disk info
    if let Some(ref snapshot) = app_state.system_snapshot {
        if snapshot.disks.is_empty() {
            let no_disks = Paragraph::new("No disks found")
                .style(Style::default().fg(text_color))
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(no_disks, chunks[1]);
        } else {
            // Calculate layout for disks
            let disk_count = snapshot.disks.len();
            let disk_constraints = vec![Constraint::Length(4); disk_count];

            let disk_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(disk_constraints)
                .split(chunks[1]);

            // Render each disk
            for (i, disk) in snapshot.disks.iter().enumerate() {
                let disk_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(disk_chunks[i]);

                // Disk usage gauge
                let disk_percent = disk.usage_percent as u16;
                let disk_gauge = Gauge::default()
                    .block(
                        Block::default()
                            .title(format!("{} ({})", disk.name, disk.mount_point))
                            .borders(Borders::ALL),
                    )
                    .gauge_style(
                        Style::default()
                            .fg(get_usage_color(disk_percent as f32))
                            .bg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                    .percent(disk_percent);

                f.render_widget(disk_gauge, disk_layout[0]);

                // Disk details
                let used_gb = (disk.total_space - disk.available_space) as f64 / 1_073_741_824.0; // Convert to GB
                let total_gb = disk.total_space as f64 / 1_073_741_824.0;

                let disk_details = Paragraph::new(vec![
                    Spans::from(vec![Span::raw(format!("Used: {:.1} GB", used_gb))]),
                    Spans::from(vec![Span::raw(format!(
                        "Free: {:.1} GB",
                        disk.available_space as f64 / 1_073_741_824.0
                    ))]),
                    Spans::from(vec![Span::raw(format!("Total: {:.1} GB", total_gb))]),
                    Spans::from(vec![Span::raw(format!("Type: {}", disk.filesystem_type))]),
                ])
                .block(Block::default().title("Details").borders(Borders::ALL));

                f.render_widget(disk_details, disk_layout[1]);
            }
        }
    } else {
        // If no snapshot is available
        let no_data = Paragraph::new("Loading disk data...")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }
}

// Helper function to clean up resources
fn cleanup_resources(app_state: &mut AppState) {
    // Clean up system monitor resources
    app_state.system_monitor = None;
    app_state.system_snapshot = None;

    // Clean up network tools resources
    app_state.speed_test_receiver = None;
    app_state.selected_tool = None;
    app_state.result = None;

    // Reset state
    app_state.input_mode = InputMode::Normal;
    app_state.error_message = None;
}

fn initialize_process_selection(app_state: &mut AppState) {
    // Only initialize if we don't have a process selected yet
    if app_state.selected_process_pid.is_none() {
        if let Some(ref snapshot) = app_state.system_snapshot {
            if !snapshot.top_processes.is_empty() {
                // Sort processes based on current sort type
                let mut sorted_processes = snapshot.top_processes.clone();
                match app_state.process_sort_type {
                    ProcessSortType::Pid => {
                        sorted_processes.sort_by_key(|p| p.pid);
                    }
                    ProcessSortType::Name => {
                        sorted_processes
                            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    }
                    ProcessSortType::CpuUsage => {
                        sorted_processes.sort_by(|a, b| {
                            b.cpu_usage
                                .partial_cmp(&a.cpu_usage)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                    ProcessSortType::MemoryUsage => {
                        sorted_processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
                    }
                    ProcessSortType::Runtime => {
                        sorted_processes.sort_by(|a, b| b.run_time.cmp(&a.run_time));
                    }
                }

                // Select the first process in the sorted list
                if !sorted_processes.is_empty() {
                    app_state.selected_process_pid = Some(sorted_processes[0].pid);
                }
            }
        }
    }
}

fn set_status_message(
    app_state: &mut AppState,
    message: &str,
    message_type: StatusMessageType,
    duration_secs: u64,
) {
    app_state.status_message = Some(StatusMessage {
        message: message.to_string(),
        message_type,
        created_at: std::time::Instant::now(),
        duration: std::time::Duration::from_secs(duration_secs),
    });
}

fn prepare_status_message(
    message: &str,
    message_type: StatusMessageType,
    duration_secs: u64,
) -> StatusMessage {
    StatusMessage {
        message: message.to_string(),
        message_type,
        created_at: std::time::Instant::now(),
        duration: std::time::Duration::from_secs(duration_secs),
    }
}

fn check_expired_status(app_state: &mut AppState) {
    // Make a copy of the relevant data
    let should_clear = if let Some(status) = &app_state.status_message {
        let current_time = std::time::Instant::now();
        let duration_elapsed = current_time.duration_since(status.created_at);
        duration_elapsed >= status.duration
    } else {
        false
    };

    // Modify based on the copy
    if should_clear {
        app_state.status_message = None;
    }
}

// Allows
#[allow(dead_code)]
fn update_ui_with_ping_result(result: String) {
    // Placeholder function to handle the ping result
    // You can update the app_state or UI here as needed
    println!("Ping result: {}", result);
}

#[allow(dead_code)]
fn update_ui_with_error(error_message: String) {
    // Placeholder function to handle errors
    // You can update the app_state or UI here as needed
    println!("Error: {}", error_message);
}
