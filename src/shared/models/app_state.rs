use crate::network_tools::SpeedTestResult;
use crate::system_utilities::SystemMonitor;
use crate::task_scheduler::{EmailConfig, ReminderType, SmsConfig, TaskPriority, TaskScheduler, TaskStatus};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
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
    ConfiguringSms,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Main,
    PasswordManager,
    NetworkTools,
    SystemUtilities,
    TaskScheduler,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemViewMode {
    Overview,
    CpuDetails,
    MemoryDetails,
    DiskDetails,
    ProcessList,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationDialogue {
    None,
    KillProcess(u32, String), // Process ID and name
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessSortType {
    Pid,
    Name,
    CpuUsage,
    MemoryUsage,
    Runtime,
}

#[derive(Debug)]
pub struct AppState {
    pub active_menu: MenuItem,
    pub input_mode: InputMode,
    pub service: String,
    pub username: String,
    pub password: String,
    pub input_field: usize,
    pub error_message: Option<String>,
    pub address: String,
    pub result: Option<String>,
    pub selected_tool: Option<String>,
    pub speed_test_receiver: Option<Receiver<SpeedTestResult>>,
    pub system_monitor: Option<Arc<Mutex<SystemMonitor>>>,
    pub selected_system_tool: Option<String>,
    pub system_view_mode: SystemViewMode,
    pub system_snapshot: Option<crate::system_utilities::SystemSnapshot>,
    pub selected_process_index: usize,
    pub confirmation_dialogue: ConfirmationDialogue,
    pub status_message: Option<StatusMessage>,
    pub process_sort_type: ProcessSortType,
    pub selected_process_pid: Option<u32>,
    pub task_scheduler: Option<Arc<Mutex<TaskScheduler>>>,
    pub task_filter: Option<String>,
    pub task_title: String,
    pub task_description: String,
    pub task_due_date: String,
    pub task_priority: TaskPriority,
    pub task_tags: String,
    pub selected_task_id: Option<u32>,
    pub email_address: String,
    pub email_smtp_server: String,
    pub email_smtp_port: String,
    pub email_username: String,
    pub email_password: String,
    pub email_config_field: usize,
    pub sms_phone_number: String,
    pub sms_carrier: String,
    pub sms_enabled: bool,
    pub sms_config_field: usize,
    pub reminder_date: String,
    pub reminder_time: String,
    pub reminder_type: ReminderType,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub message_type: StatusMessageType,
    pub created_at: Instant,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusMessageType {
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
            sms_phone_number: String::new(),
            sms_carrier: String::from("att"),
            sms_enabled: false,
            sms_config_field: 0,
            reminder_date: String::new(),
            reminder_time: String::new(),
            reminder_type: ReminderType::Email,
        }
    }
}
