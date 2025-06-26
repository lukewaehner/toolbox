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

pub mod task;
pub mod scheduler;

// Re-export main types for convenience
pub use task::{Task, TaskPriority, TaskStatus, ReminderType, Reminder};
pub use scheduler::{TaskScheduler, SchedulerError, run_scheduler_background_thread};

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
    Sms,
    Both,
    All, // Email + SMS + Notification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub phone_number: String,
    pub carrier: String, // att, verizon, tmobile, sprint, etc.
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub email: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            email: String::new(),
            smtp_server: String::new(),
            smtp_port: 587,
            username: String::new(),
            password: String::new(),
            retry_attempts: 3,
            retry_delay_seconds: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub reminder_time: i64, // Unix timestamp
    pub reminder_type: ReminderType,
    pub sent: bool,
    pub retry_count: u32,
    pub last_attempt: Option<i64>,
    pub error_message: Option<String>,
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
            retry_count: 0,
            last_attempt: None,
            error_message: None,
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
    sms_config: Option<SmsConfig>,
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
            // Validate email configuration
            if config.email.is_empty() || config.smtp_server.is_empty() || config.username.is_empty() {
                return Err("Email configuration is incomplete. Please check your settings.".to_string());
            }
            
            // Create a test email with proper error handling
            println!("==== TEST EMAIL DEBUG INFO ====");
            println!("SMTP Server: {}", &config.smtp_server);
            println!("SMTP Port: {}", config.smtp_port);
            println!("Username: {}", &config.username);
            println!("Password: {}", if config.password.is_empty() { "EMPTY!" } else { "******" });
            println!("From: Task Scheduler <{}>", &config.email);
            println!("To: {}", &config.email);
            
            let email = Message::builder()
                .from(format!("Task Scheduler <{}>", &config.email).parse().map_err(|e| format!("Invalid sender email: {}", e))?)
                .to(config.email.parse().map_err(|e| format!("Invalid recipient email: {}", e))?)
                .subject("Test Email from Task Scheduler")
                .body("This is a test email to verify your email configuration is working correctly.".to_string())
                .map_err(|e| format!("Failed to create email: {}", e))?;

            // Create SMTP transport
            let creds = Credentials::new(config.username.clone(), config.password.clone());

            // Use proper transport settings for different email providers
            let mailer = if config.smtp_server.contains("gmail") {
                println!("Using Gmail-specific settings with STARTTLS");
                
                // For Gmail, we need to enable TLS and use specific authentication settings
                use lettre::transport::smtp::authentication::Mechanism;
                
                SmtpTransport::starttls_relay(&config.smtp_server)
                    .map_err(|e| format!("Failed to create Gmail relay: {}", e))?
                    .credentials(creds)
                    .port(config.smtp_port)
                    .authentication(vec![Mechanism::Plain])
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build()
            } else if config.smtp_server.contains("outlook") || config.smtp_server.contains("hotmail") {
                println!("Using Outlook-specific settings with STARTTLS");
                
                SmtpTransport::starttls_relay(&config.smtp_server)
                    .map_err(|e| format!("Failed to create Outlook relay: {}", e))?
                    .credentials(creds)
                    .port(config.smtp_port)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build()
            } else if config.smtp_server.contains("yahoo") {
                println!("Using Yahoo-specific settings with TLS wrapper");
                
                SmtpTransport::relay(&config.smtp_server)
                    .map_err(|e| format!("Failed to create Yahoo relay: {}", e))?
                    .credentials(creds)
                    .port(config.smtp_port)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build()
            } else {
                SmtpTransport::relay(&config.smtp_server)
                    .map_err(|e| format!("Failed to create mailer: {}", e))?
                    .credentials(creds)
                    .port(config.smtp_port)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build()
            };

            // Send email
            println!("Attempting to connect to SMTP server...");
            println!("Sending test email to {}", &config.email);
            
            match mailer.send(&email) {
                Ok(_) => {
                    println!("✅ Test email sent successfully!");
                    println!("==== END TEST EMAIL DEBUG ====");
                    return Ok(());
                }
                Err(e) => {
                    println!("❌ TEST EMAIL SENDING FAILED");
                    let error_msg = format!("Failed to send test email: {}", e);
                    eprintln!("{}", error_msg);
                    
                    if config.smtp_server.contains("gmail") {
                        eprintln!("=== GMAIL TROUBLESHOOTING ===");
                        eprintln!("1. App Password: If using 2FA, you MUST use an App Password");
                        eprintln!("   - Go to https://myaccount.google.com/apppasswords to generate one");
                        eprintln!("2. Less secure app access: This setting must be ON if not using 2FA");
                        eprintln!("   - Go to https://myaccount.google.com/lesssecureapps");
                        eprintln!("3. Try testing with another email provider like Outlook or Yahoo");
                        eprintln!("4. Gmail often blocks 'unusual sign-in attempts'");
                        eprintln!("   - Check your Gmail inbox for security alerts");
                    } else if config.smtp_server.contains("outlook") || config.smtp_server.contains("hotmail") {
                        eprintln!("=== OUTLOOK TROUBLESHOOTING ===");
                        eprintln!("1. Make sure you're using your full Outlook/Hotmail email address as username");
                        eprintln!("2. Check if you have 2FA enabled - you might need an app password");
                        eprintln!("3. Try smtp-mail.outlook.com with port 587");
                    } else if config.smtp_server.contains("yahoo") {
                        eprintln!("=== YAHOO TROUBLESHOOTING ===");
                        eprintln!("1. Make sure you're using your full Yahoo email address as username");
                        eprintln!("2. You might need to create an app password if 2FA is enabled");
                        eprintln!("3. Try smtp.mail.yahoo.com with port 465");
                    }
                    
                    println!("==== END TEST EMAIL DEBUG ====");
                    return Err(error_msg);
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

        println!("Attempting to load email config from: {:?}", config_path);

        if config_path.exists() {
            println!("Email config file exists!");
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => {
                    if contents.trim().is_empty() {
                        eprintln!("Email config file is empty");
                        return;
                    }
                    
                    println!("Read email config file contents: {}", contents);
                    match serde_json::from_str::<EmailConfig>(&contents) {
                        Ok(config) => {
                            // Validate that we have the minimum required fields
                            if config.email.is_empty() || config.smtp_server.is_empty() {
                                eprintln!("Email config is missing required fields");
                                return;
                            }
                            
                            println!("Successfully parsed email config");
                            self.email_config = Some(config);
                        }
                        Err(e) => {
                            eprintln!("Error parsing email config: {}", e);
                            eprintln!("Config content: {}", contents);
                        }
                    }
                }
                Err(e) => eprintln!("Error reading email config file: {}", e),
            }
        } else {
            println!("Email config file does not exist at path: {:?}", config_path);
        }
    }

    pub fn set_sms_config(&mut self, config: SmsConfig) {
        self.sms_config = Some(config.clone());
        self.save_sms_config();
    }

    pub fn get_sms_gateway_email(&self, phone_number: &str, carrier: &str) -> Option<String> {
        let clean_number = phone_number.chars().filter(|c| c.is_numeric()).collect::<String>();
        
        match carrier.to_lowercase().as_str() {
            "att" | "at&t" => Some(format!("{}@txt.att.net", clean_number)),
            "verizon" => Some(format!("{}@vtext.com", clean_number)),
            "tmobile" | "t-mobile" => Some(format!("{}@tmomail.net", clean_number)),
            "sprint" => Some(format!("{}@messaging.sprintpcs.com", clean_number)),
            "boost" => Some(format!("{}@myboostmobile.com", clean_number)),
            "cricket" => Some(format!("{}@sms.cricketwireless.net", clean_number)),
            "metropcs" => Some(format!("{}@mymetropcs.com", clean_number)),
            "virgin" => Some(format!("{}@vmobl.com", clean_number)),
            "uscellular" => Some(format!("{}@email.uscc.net", clean_number)),
            _ => None,
        }
    }

    pub fn send_sms_reminder(&self, task_id: u32, message: &str) -> Result<(), String> {
        let sms_config = self.sms_config.as_ref()
            .ok_or_else(|| "SMS configuration not set".to_string())?;

        if !sms_config.enabled {
            return Err("SMS is disabled in configuration".to_string());
        }

        let email_config = self.email_config.as_ref()
            .ok_or_else(|| "Email configuration required for SMS (uses email-to-SMS gateway)".to_string())?;

        let gateway_email = self.get_sms_gateway_email(&sms_config.phone_number, &sms_config.carrier)
            .ok_or_else(|| format!("Unsupported carrier: {}", sms_config.carrier))?;

        println!("Sending SMS via email gateway to: {}", gateway_email);

        // Create SMS email (keep it short for SMS)
        let sms_message = if message.len() > 160 {
            format!("{}...", &message[..157])
        } else {
            message.to_string()
        };

        let email = Message::builder()
            .from(format!("Task Reminder <{}>", &email_config.email).parse()
                .map_err(|e| format!("Invalid sender email: {}", e))?)
            .to(gateway_email.parse()
                .map_err(|e| format!("Invalid gateway email: {}", e))?)
            .subject("") // SMS gateways often ignore subject
            .body(sms_message)
            .map_err(|e| format!("Failed to create SMS email: {}", e))?;

        // Use the same SMTP transport as email
        let creds = Credentials::new(email_config.username.clone(), email_config.password.clone());
        let mailer = self.create_smtp_transport(&email_config, &creds)?;

        match mailer.send(&email) {
            Ok(_) => {
                println!("✅ SMS sent successfully via {} gateway", sms_config.carrier);
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to send SMS: {}", e);
                eprintln!("❌ SMS SENDING FAILED: {}", error_msg);
                Err(error_msg)
            }
        }
    }

    fn create_smtp_transport(&self, config: &EmailConfig, creds: &Credentials) -> Result<SmtpTransport, String> {
        // Create a properly configured transport with improved settings based on the provider
        if config.smtp_server.contains("gmail") {
            println!("Using Gmail-specific settings with STARTTLS");
            
            use lettre::transport::smtp::authentication::Mechanism;
            
            Ok(SmtpTransport::starttls_relay(&config.smtp_server)
                .map_err(|e| format!("Failed to create Gmail relay: {}", e))?
                .credentials(creds.clone())
                .port(config.smtp_port)
                .authentication(vec![Mechanism::Plain])
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build())
        } else if config.smtp_server.contains("outlook") || config.smtp_server.contains("hotmail") {
            println!("Using Outlook-specific settings with STARTTLS");
            
            Ok(SmtpTransport::starttls_relay(&config.smtp_server)
                .map_err(|e| format!("Failed to create Outlook relay: {}", e))?
                .credentials(creds.clone())
                .port(config.smtp_port)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build())
        } else if config.smtp_server.contains("yahoo") {
            println!("Using Yahoo-specific settings with TLS wrapper");
            
            Ok(SmtpTransport::relay(&config.smtp_server)
                .map_err(|e| format!("Failed to create Yahoo relay: {}", e))?
                .credentials(creds.clone())
                .port(config.smtp_port)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build())
        } else {
            println!("Using standard SMTP relay settings");
            
            Ok(SmtpTransport::relay(&config.smtp_server)
                .map_err(|e| format!("Failed to create relay: {}", e))?
                .credentials(creds.clone())
                .port(config.smtp_port)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build())
        }
    }

    fn save_sms_config(&self) {
        if let Some(ref config) = self.sms_config {
            let config_dir = std::path::Path::new(&self.file_path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let config_path = config_dir.join("sms_config.json");

            match serde_json::to_string_pretty(config) {
                Ok(contents) => {
                    match std::fs::File::create(&config_path) {
                        Ok(mut file) => {
                            if let Err(e) = std::io::Write::write_all(&mut file, contents.as_bytes()) {
                                eprintln!("Error writing SMS config file: {}", e);
                            } else {
                                println!("Successfully saved SMS config");
                            }
                        }
                        Err(e) => eprintln!("Error creating SMS config file: {}", e),
                    }
                }
                Err(e) => eprintln!("Error serializing SMS config: {}", e),
            }
        }
    }

    fn load_sms_config(&mut self) {
        let config_dir = std::path::Path::new(&self.file_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let config_path = config_dir.join("sms_config.json");

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => {
                    if !contents.trim().is_empty() {
                        match serde_json::from_str::<SmsConfig>(&contents) {
                            Ok(config) => {
                                println!("Successfully loaded SMS config");
                                self.sms_config = Some(config);
                            }
                            Err(e) => eprintln!("Error parsing SMS config: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Error reading SMS config file: {}", e),
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
            sms_config: None,
        };

        // Load existing tasks if file exists
        scheduler.load_tasks();

        // Load email configuration
        scheduler.load_email_config();

        // Load SMS configuration
        scheduler.load_sms_config();

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

    pub fn check_reminders(&mut self) -> Vec<(u32, String, ReminderType, Option<usize>)> {
        let mut triggered_reminders = Vec::new();
        let now = Utc::now().timestamp();
        
        println!("Current time (UTC): {}", format_timestamp(now));
        println!("Checking for reminders that should trigger...");

        // First, collect tasks and reminders to process
        let mut notification_tasks = Vec::new();
        
        // Scan tasks to find reminders that need to be triggered
        for (id, task) in &mut self.tasks {
            println!("Checking task ID {}: '{}' with {} reminder(s)", id, task.title, task.reminders.len());
            
            for (i, reminder) in task.reminders.iter_mut().enumerate() {
                println!("  Reminder #{}: scheduled for {} (sent: {}, retries: {})", 
                         i+1, 
                         format_timestamp(reminder.reminder_time),
                         reminder.sent,
                         reminder.retry_count);
                
                let should_retry = !reminder.sent && 
                    reminder.retry_count < 3 &&
                    (reminder.last_attempt.is_none() || 
                     now - reminder.last_attempt.unwrap_or(0) > 300); // 5 minute retry delay
                
                if !reminder.sent && reminder.reminder_time <= now && should_retry {
                    println!("  --> Reminder #{} is due and not yet sent - triggering now", i+1);
                    
                    // Update attempt tracking
                    reminder.last_attempt = Some(now);
                    reminder.retry_count += 1;
                    
                    match &reminder.reminder_type {
                        ReminderType::Notification => {
                            reminder.sent = true;
                            notification_tasks.push((task.title.clone(), task.description.clone()));
                            
                            triggered_reminders.push((
                                *id, 
                                task.title.clone(), 
                                ReminderType::Notification,
                                None
                            ));
                        },
                        ReminderType::Email => {
                            triggered_reminders.push((
                                *id,
                                task.title.clone(),
                                ReminderType::Email,
                                Some(i)
                            ));
                        },
                        ReminderType::Sms => {
                            triggered_reminders.push((
                                *id,
                                task.title.clone(),
                                ReminderType::Sms,
                                Some(i)
                            ));
                        },
                        ReminderType::Both => {
                            // Desktop notification
                            notification_tasks.push((task.title.clone(), task.description.clone()));
                            
                            // Email
                            triggered_reminders.push((
                                *id,
                                task.title.clone(),
                                ReminderType::Email,
                                Some(i)
                            ));
                        },
                        ReminderType::All => {
                            // Desktop notification
                            notification_tasks.push((task.title.clone(), task.description.clone()));
                            
                            // Email and SMS
                            triggered_reminders.push((
                                *id,
                                task.title.clone(),
                                ReminderType::Email,
                                Some(i)
                            ));
                            triggered_reminders.push((
                                *id,
                                task.title.clone(),
                                ReminderType::Sms,
                                Some(i)
                            ));
                        }
                    }
                } else if reminder.sent {
                    println!("  --> Reminder #{} already sent, skipping", i+1);
                } else if reminder.retry_count >= 3 {
                    println!("  --> Reminder #{} exceeded retry limit", i+1);
                } else {
                    let time_diff = reminder.reminder_time - now;
                    let hours = time_diff / 3600;
                    let minutes = (time_diff % 3600) / 60;
                    println!("  --> Reminder #{} not yet due, will trigger in {}h {}m", 
                             i+1, hours, minutes);
                }
            }
        }

        // Send desktop notifications now
        for (title, description) in notification_tasks {
            if let Err(e) = self.send_desktop_notification(&title, &description) {
                eprintln!("Failed to send notification: {}", e);
            }
        }

        // Save task changes to file
        if !triggered_reminders.is_empty() {
            println!("Saving tasks after processing reminders");
            self.save_tasks();
        }

        triggered_reminders
    }
    
    // Helper method to mark a reminder as sent after email is successfully delivered
    pub fn mark_reminder_as_sent(&mut self, task_id: u32, reminder_index: usize) -> Result<(), String> {
        // First get the task title before modifying anything
        let task_title = match self.tasks.get(&task_id) {
            Some(task) => task.title.clone(),
            None => return Err(format!("Task with ID {} not found", task_id)),
        };
        
        // Now modify the task's reminder
        if let Some(task) = self.tasks.get_mut(&task_id) {
            if reminder_index < task.reminders.len() {
                task.reminders[reminder_index].sent = true;
                
                // Save tasks after modification
                self.save_tasks();
                
                println!("Marked reminder #{} for task '{}' as sent", reminder_index + 1, task_title);
                Ok(())
            } else {
                Err(format!("Reminder index {} out of bounds for task {}", reminder_index, task_id))
            }
        } else {
            Err(format!("Task with ID {} not found", task_id))
        }
    }

    pub fn send_reminder_email(&self, task_id: u32) -> Result<(), String> {
        let task = self
            .get_task(task_id)
            .ok_or_else(|| format!("Task with ID {} not found", task_id))?;

        let config = self
            .email_config
            .as_ref()
            .ok_or_else(|| "Email configuration not set".to_string())?;

        // Verify email configuration is valid
        if config.email.is_empty() || config.smtp_server.is_empty() || config.username.is_empty() {
            return Err("Email configuration is incomplete. Please check your settings.".to_string());
        }

        println!("Creating email with the following values:");
        println!("From: Task Scheduler <{}>", &config.email);
        println!("To: {}", &config.email);
        println!("Subject: Reminder: {}", task.title);
        
        // Create email with proper formatting and complete body
        let email = Message::builder()
            .from(format!("Task Scheduler <{}>", &config.email).parse().map_err(|e| format!("Invalid sender email format: {}", e))?)
            .to(config.email.parse().map_err(|e| format!("Invalid recipient email format: {}", e))?)
            .subject(format!("Reminder: {}", task.title))
            .body(format!(
                "This is a reminder for your task: {}\n\nDescription: {}\n\nDue: {}\n\nPriority: {:?}",
                task.title,
                task.description,
                format_timestamp(task.due_date),
                task.priority
            ))
            .map_err(|e| format!("Failed to create email: {}", e))?;

        // Create SMTP transport with proper debugging
        println!("==== EMAIL DEBUG INFO ====");
        println!("SMTP Server: {}", &config.smtp_server);
        println!("SMTP Port: {}", config.smtp_port);
        println!("Username: {}", &config.username);
        println!("Password: {}", if config.password.is_empty() { "EMPTY!" } else { "******" });
        
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        // Create a properly configured transport with improved settings based on the provider
        let mailer = self.create_smtp_transport(&config, &creds)?;

        // Send email with better error handling and don't mark as sent until successful
        println!("Attempting to connect to SMTP server...");
        println!("Sending email to {} for task: {}", &config.email, task.title);
        
        let send_result = mailer.send(&email);
        
        match send_result {
            Ok(_) => {
                println!("✅ Email sent successfully to {} for task: {}", &config.email, task.title);
                println!("==== END EMAIL DEBUG ====");
                Ok(())
            },
            Err(e) => {
                println!("❌ EMAIL SENDING FAILED");
                let error_msg = format!("Failed to send email: {}", e);
                eprintln!("{}", error_msg);
                
                if config.smtp_server.contains("gmail") {
                    eprintln!("=== GMAIL TROUBLESHOOTING ===");
                    eprintln!("1. App Password: If using 2FA, you MUST use an App Password");
                    eprintln!("   - Go to https://myaccount.google.com/apppasswords to generate one");
                    eprintln!("2. Less secure app access: This setting must be ON if not using 2FA");
                    eprintln!("   - Go to https://myaccount.google.com/lesssecureapps");
                    eprintln!("3. Try testing with another email provider like Outlook or Yahoo");
                    eprintln!("4. Gmail often blocks 'unusual sign-in attempts'");
                    eprintln!("   - Check your Gmail inbox for security alerts");
                } else if config.smtp_server.contains("outlook") || config.smtp_server.contains("hotmail") {
                    eprintln!("=== OUTLOOK TROUBLESHOOTING ===");
                    eprintln!("1. Make sure you're using your full Outlook/Hotmail email address as username");
                    eprintln!("2. Check if you have 2FA enabled - you might need an app password");
                    eprintln!("3. Try smtp-mail.outlook.com with port 587");
                } else if config.smtp_server.contains("yahoo") {
                    eprintln!("=== YAHOO TROUBLESHOOTING ===");
                    eprintln!("1. Make sure you're using your full Yahoo email address as username");
                    eprintln!("2. You might need to create an app password if 2FA is enabled");
                    eprintln!("3. Try smtp.mail.yahoo.com with port 465");
                }
                
                println!("==== END EMAIL DEBUG ====");
                Err(error_msg)
            }
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
        println!("Starting task scheduler background thread");
        loop {
            thread::sleep(Duration::from_secs(30)); // Check every 30 seconds (for testing)
            
            println!("Checking for due reminders...");
            let mut triggered_reminders = Vec::new();

            // Check for reminders in a separate scope to release the lock
            {
                if let Ok(mut sched) = scheduler.lock() {
                    triggered_reminders = sched.check_reminders();
                }
            }

            // Log the number of triggered reminders for debugging
            println!("Found {} triggered reminders", triggered_reminders.len());

            // Send notifications for triggered reminders
            for (task_id, task_title, reminder_type, reminder_index) in triggered_reminders {
                println!("Processing reminder for task {}: {}", task_id, task_title);
                
                let mut reminder_success = false;
                let mut error_message = None;
                
                match reminder_type {
                    ReminderType::Email => {
                        println!("Sending email reminder for task {}", task_id);
                        if let Ok(sched) = scheduler.lock() {
                            match sched.send_reminder_email(task_id) {
                                Ok(_) => {
                                    println!("Successfully sent email reminder for task {}", task_id);
                                    reminder_success = true;
                                },
                                Err(e) => {
                                    eprintln!("Failed to send email for task {}: {}", task_id, e);
                                    error_message = Some(e);
                                }
                            }
                        }
                    },
                    ReminderType::Sms => {
                        println!("Sending SMS reminder for task {}", task_id);
                        if let Ok(sched) = scheduler.lock() {
                            let sms_message = format!("Task Reminder: {}", task_title);
                            match sched.send_sms_reminder(task_id, &sms_message) {
                                Ok(_) => {
                                    println!("Successfully sent SMS reminder for task {}", task_id);
                                    reminder_success = true;
                                },
                                Err(e) => {
                                    eprintln!("Failed to send SMS for task {}: {}", task_id, e);
                                    error_message = Some(e);
                                }
                            }
                        }
                    },
                    ReminderType::Notification => {
                        println!("Desktop notification already sent for task {}", task_id);
                        reminder_success = true;
                    },
                    _ => {
                        println!("Unsupported reminder type for task {}", task_id);
                    }
                }
                
                // Update reminder status
                if let Some(index) = reminder_index {
                    if let Ok(mut sched) = scheduler.lock() {
                        if reminder_success {
                            if let Err(e) = sched.mark_reminder_as_sent(task_id, index) {
                                eprintln!("Failed to mark reminder as sent: {}", e);
                            }
                        } else {
                            // Update error message for failed reminder
                            if let Some(task) = sched.tasks.get_mut(&task_id) {
                                if index < task.reminders.len() {
                                    task.reminders[index].error_message = error_message;
                                    sched.save_tasks();
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
