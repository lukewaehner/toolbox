// task_scheduler.rs
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use notify_rust::{Notification, Timeout};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReminderType {
    Email,
    Notification,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub reminder_time: i64, // Unix timestamp
    pub reminder_type: ReminderType,
    pub sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub due_date: i64, // Unix timestamp
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: i64, // Unix timestamp
    pub reminders: Vec<Reminder>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub email: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
}

impl Task {
    pub fn new(
        id: u32,
        title: String,
        description: String,
        due_date: i64,
        priority: TaskPriority,
        tags: Vec<String>,
    ) -> Self {
        let now = Utc::now().timestamp();

        Self {
            id,
            title,
            description,
            due_date,
            priority,
            status: TaskStatus::Pending,
            created_at: now,
            reminders: Vec::new(),
            tags,
        }
    }

    pub fn add_reminder(&mut self, reminder_time: i64, reminder_type: ReminderType) {
        self.reminders.push(Reminder {
            reminder_time,
            reminder_type,
            sent: false,
        });
    }

    pub fn is_due(&self) -> bool {
        let now = Utc::now().timestamp();
        self.due_date <= now
    }

    pub fn has_pending_reminders(&self) -> bool {
        let now = Utc::now().timestamp();
        self.reminders
            .iter()
            .any(|r| !r.sent && r.reminder_time <= now)
    }
}

#[derive(Debug)]
pub struct TaskScheduler {
    tasks: HashMap<u32, Task>,
    next_id: u32,
    file_path: String,
    email_config: Option<EmailConfig>,
}

impl TaskScheduler {
    fn send_desktop_notification(
        &self,
        title: &str,
        body: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Notification::new()
            .summary(&format!("Task Reminder: {}", title))
            .body(body)
            .icon("calendar")
            .timeout(notify_rust::Timeout::Milliseconds(5000))
            .show()?;

        Ok(())
    }

    pub fn set_email_config(&mut self, config: EmailConfig) {
        self.email_config = Some(config.clone());
        self.save_email_config();
    }

    pub fn test_email_config(&self) -> Result<(), String> {
        if let Some(ref config) = self.email_config {
            // Create a test email
            let email = Message::builder()
                .from(
                    format!("Task Scheduler <{}>", &config.email)
                        .parse()
                        .unwrap(),
                )
                .to(config.email.parse().unwrap())
                .subject("Test Email from Task Scheduler")
                .body(
                    "This is a test email to verify your email configuration is working correctly."
                        .to_string(),
                )
                .map_err(|e| format!("Failed to create email: {}", e))?;

            // Create SMTP transport
            let creds = Credentials::new(config.username.clone(), config.password.clone());

            let mailer = SmtpTransport::relay(&config.smtp_server)
                .map_err(|e| format!("Failed to create mailer: {}", e))?
                .credentials(creds)
                .port(config.smtp_port)
                .build();

            // Send email
            match mailer.send(&email) {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(format!("Failed to send test email: {}", e));
                }
            }
        } else {
            return Err("Email configuration not set".to_string());
        }
    }

    fn save_email_config(&self) {
        if let Some(ref config) = self.email_config {
            // Create a file path based on your application's config directory
            let config_dir = std::path::Path::new(&self.file_path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let config_path = config_dir.join("email_config.json");

            println!("Saving email config to: {:?}", config_path); // Debug print

            // Serialize the config
            match serde_json::to_string(config) {
                Ok(contents) => {
                    println!("Serialized config: {}", contents); // Debug print
                    match std::fs::File::create(&config_path) {
                        Ok(mut file) => {
                            if let Err(e) =
                                std::io::Write::write_all(&mut file, contents.as_bytes())
                            {
                                eprintln!("Error writing to email config file: {}", e);
                            } else {
                                println!("Successfully wrote email config file");
                                // Debug print
                            }
                        }
                        Err(e) => eprintln!("Error creating email config file: {}", e),
                    }
                }
                Err(e) => eprintln!("Error serializing email config: {}", e),
            }
        } else {
            println!("No email config to save"); // Debug print
        }
    }

    fn load_email_config(&mut self) {
        // Create a file path based on your application's config directory
        let config_dir = std::path::Path::new(&self.file_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let config_path = config_dir.join("email_config.json");

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match serde_json::from_str::<EmailConfig>(&contents) {
                    Ok(config) => {
                        self.email_config = Some(config);
                    }
                    Err(e) => eprintln!("Error parsing email config: {}", e),
                },
                Err(e) => eprintln!("Error reading email config file: {}", e),
            }
        }
    }

    // Modify your new function to load email config too
    pub fn new(file_path: &str) -> Self {
        let mut scheduler = Self {
            tasks: HashMap::new(),
            next_id: 1,
            file_path: file_path.to_string(),
            email_config: None,
        };

        // Load existing tasks if file exists
        scheduler.load_tasks();

        // Load email configuration
        scheduler.load_email_config();

        scheduler
    }

    pub fn add_task(
        &mut self,
        title: String,
        description: String,
        due_date: i64,
        priority: TaskPriority,
        tags: Vec<String>,
    ) -> u32 {
        let id = self.next_id;
        let task = Task::new(id, title, description, due_date, priority, tags);

        self.tasks.insert(id, task);
        self.next_id += 1;
        self.save_tasks();

        id
    }

    pub fn update_task(&mut self, id: u32, updated_task: Task) -> Result<(), String> {
        if !self.tasks.contains_key(&id) {
            return Err(format!("Task with ID {} not found", id));
        }

        self.tasks.insert(id, updated_task);
        self.save_tasks();

        Ok(())
    }

    pub fn delete_task(&mut self, id: u32) -> Result<(), String> {
        if !self.tasks.contains_key(&id) {
            return Err(format!("Task with ID {} not found", id));
        }

        self.tasks.remove(&id);
        self.save_tasks();

        Ok(())
    }

    pub fn get_task(&self, id: u32) -> Option<&Task> {
        self.tasks.get(&id)
    }

    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status == TaskStatus::Pending)
            .collect()
    }

    pub fn add_reminder_to_task(
        &mut self,
        task_id: u32,
        reminder_time: i64,
        reminder_type: ReminderType,
    ) -> Result<(), String> {
        let task = self
            .tasks
            .get_mut(&task_id)
            .ok_or_else(|| format!("Task with ID {} not found", task_id))?;
        task.add_reminder(reminder_time, reminder_type);
        self.save_tasks();

        Ok(())
    }

    pub fn check_reminders(&mut self) -> Vec<(u32, String, ReminderType)> {
        let mut triggered_reminders = Vec::new();
        let now = Utc::now().timestamp();

        // First, collect all the tasks that need notifications
        let mut notification_tasks = Vec::new();

        for (id, task) in &mut self.tasks {
            for reminder in &mut task.reminders {
                if !reminder.sent && reminder.reminder_time <= now {
                    // Mark reminder as sent
                    reminder.sent = true;

                    // If it needs notification, collect the info
                    if reminder.reminder_type == ReminderType::Notification
                        || reminder.reminder_type == ReminderType::Both
                    {
                        notification_tasks.push((task.title.clone(), task.description.clone()));
                    }

                    // Add to triggered reminders
                    triggered_reminders.push((
                        *id,
                        task.title.clone(),
                        reminder.reminder_type.clone(),
                    ));
                }
            }
        }

        // Save changes to reminders
        if !triggered_reminders.is_empty() {
            self.save_tasks();
        }

        // Now send notifications after the mutable borrow has ended
        for (title, description) in notification_tasks {
            if let Err(e) = self.send_desktop_notification(&title, &description) {
                eprintln!("Failed to send notification: {}", e);
            }
        }

        triggered_reminders
    }

    pub fn send_reminder_email(&self, task_id: u32) -> Result<(), String> {
        let task = self
            .get_task(task_id)
            .ok_or_else(|| format!("Task with ID {} not found", task_id))?;

        let config = self
            .email_config
            .as_ref()
            .ok_or_else(|| "Email configuration not set".to_string())?;

        // Create email
        let email = Message::builder()
            .from(format!("Task Scheduler <{}>", &config.email).parse().unwrap())
            .to(config.email.parse().unwrap())
            .subject(format!("Reminder: {}", task.title))
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(format!(
                                "This is a reminder for your task: {}\n\nDescription: {}\n\nDue: {}\n\nPriority: {:?}",
                                task.title,
                                task.description,
                                format_timestamp(task.due_date),
                                task.priority
                            ))
                    )
            )
            .map_err(|e| format!("Failed to create email: {}", e))?;

        // Create SMTP transport
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let mailer = SmtpTransport::relay(&config.smtp_server)
            .map_err(|e| format!("Failed to create mailer: {}", e))?
            .credentials(creds)
            .port(config.smtp_port)
            .build();

        // Send email
        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to send email: {}", e)),
        }
    }

    fn load_tasks(&mut self) {
        if !Path::new(&self.file_path).exists() {
            return;
        }

        match fs::read_to_string(&self.file_path) {
            Ok(contents) => match serde_json::from_str::<(HashMap<u32, Task>, u32)>(&contents) {
                Ok((tasks, next_id)) => {
                    self.tasks = tasks;
                    self.next_id = next_id;
                }
                Err(e) => eprintln!("Error parsing tasks: {}", e),
            },
            Err(e) => eprintln!("Error reading tasks file: {}", e),
        }
    }

    fn save_tasks(&self) {
        let contents = match serde_json::to_string(&(self.tasks.clone(), self.next_id)) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error serializing tasks: {}", e);
                return;
            }
        };

        match File::create(&self.file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(contents.as_bytes()) {
                    eprintln!("Error writing to tasks file: {}", e);
                }
            }
            Err(e) => eprintln!("Error creating tasks file: {}", e),
        }
    }
}

pub fn format_timestamp(timestamp: i64) -> String {
    if let Some(dt) = NaiveDateTime::from_timestamp_opt(timestamp, 0) {
        let local_time = DateTime::<Local>::from_naive_utc_and_offset(dt, *Local::now().offset());
        local_time.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid date".to_string()
    }
}

pub fn run_scheduler_background_thread(
    scheduler: Arc<Mutex<TaskScheduler>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60)); // Check every minute

            let mut triggered_reminders = Vec::new();

            // Check for reminders in a separate scope to release the lock
            {
                if let Ok(mut sched) = scheduler.lock() {
                    triggered_reminders = sched.check_reminders();
                }
            }

            // Send notifications for triggered reminders
            for (task_id, task_title, reminder_type) in triggered_reminders {
                match reminder_type {
                    ReminderType::Email | ReminderType::Both => {
                        if let Ok(sched) = scheduler.lock() {
                            if let Err(e) = sched.send_reminder_email(task_id) {
                                eprintln!("Failed to send email for task {}: {}", task_id, e);
                            }
                        }
                    }
                    ReminderType::Notification | ReminderType::Both => {
                        // Desktop notifications can be implemented with the notify-rust crate
                        println!("Notification for task {}: {}", task_id, task_title);
                    }
                }
            }
        }
    })
}
