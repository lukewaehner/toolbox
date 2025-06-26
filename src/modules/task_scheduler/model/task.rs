use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
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
    All,
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
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<i64>, // Unix timestamp
    pub created_at: i64,
    pub updated_at: i64,
    pub reminders: Vec<Reminder>,
    pub tags: Vec<String>,
}

impl Task {
    pub fn new(id: u32, title: String, description: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id,
            title,
            description,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            due_date: None,
            created_at: now,
            updated_at: now,
            reminders: Vec::new(),
            tags: Vec::new(),
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
        self.update_timestamp();
    }

    pub fn set_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
        self.update_timestamp();
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.update_timestamp();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.update_timestamp();
        }
    }

    pub fn is_due(&self) -> bool {
        if let Some(due_date) = self.due_date {
            due_date <= Utc::now().timestamp()
        } else {
            false
        }
    }

    pub fn has_pending_reminders(&self) -> bool {
        self.reminders.iter().any(|r| !r.sent)
    }

    fn update_timestamp(&mut self) {
        self.updated_at = Utc::now().timestamp();
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl Default for ReminderType {
    fn default() -> Self {
        ReminderType::Email
    }
} 