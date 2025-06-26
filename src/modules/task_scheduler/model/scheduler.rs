use super::task::{Task, TaskPriority, TaskStatus, ReminderType};
use crate::core::config::get_config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Task with ID {id} not found")]
    TaskNotFound { id: u32 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Email configuration error: {0}")]
    EmailConfig(String),
    
    #[error("SMS configuration error: {0}")]
    SmsConfig(String),
}

#[derive(Debug)]
pub struct TaskScheduler {
    tasks: HashMap<u32, Task>,
    next_id: u32,
    file_path: String,
}

impl TaskScheduler {
    pub fn new(file_path: &str) -> Self {
        let mut scheduler = Self {
            tasks: HashMap::new(),
            next_id: 1,
            file_path: file_path.to_string(),
        };
        
        if let Err(e) = scheduler.load_tasks() {
            eprintln!("Failed to load tasks: {}", e);
        }
        
        scheduler
    }

    pub fn add_task(&mut self, mut task: Task) -> Result<u32, SchedulerError> {
        let id = self.next_id;
        task.id = id;
        self.tasks.insert(id, task);
        self.next_id += 1;
        self.save_tasks()?;
        Ok(id)
    }

    pub fn get_task(&self, id: u32) -> Option<&Task> {
        self.tasks.get(&id)
    }

    pub fn get_task_mut(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.get_mut(&id)
    }

    pub fn update_task(&mut self, id: u32, task: Task) -> Result<(), SchedulerError> {
        if self.tasks.contains_key(&id) {
            self.tasks.insert(id, task);
            self.save_tasks()?;
            Ok(())
        } else {
            Err(SchedulerError::TaskNotFound { id })
        }
    }

    pub fn delete_task(&mut self, id: u32) -> Result<(), SchedulerError> {
        if self.tasks.remove(&id).is_some() {
            self.save_tasks()?;
            Ok(())
        } else {
            Err(SchedulerError::TaskNotFound { id })
        }
    }

    pub fn list_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status == TaskStatus::Pending)
            .collect()
    }

    pub fn get_due_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.is_due())
            .collect()
    }

    pub fn get_tasks_with_pending_reminders(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.has_pending_reminders())
            .collect()
    }

    pub fn mark_reminder_as_sent(&mut self, task_id: u32, reminder_index: usize) -> Result<(), SchedulerError> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            if let Some(reminder) = task.reminders.get_mut(reminder_index) {
                reminder.sent = true;
                reminder.error_message = None;
                self.save_tasks()?;
                Ok(())
            } else {
                Err(SchedulerError::TaskNotFound { id: task_id })
            }
        } else {
            Err(SchedulerError::TaskNotFound { id: task_id })
        }
    }

    pub fn get_tasks_by_priority(&self, priority: TaskPriority) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.priority == priority)
            .collect()
    }

    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status == status)
            .collect()
    }

    pub fn search_tasks(&self, query: &str) -> Vec<&Task> {
        let query_lower = query.to_lowercase();
        self.tasks
            .values()
            .filter(|task| {
                task.title.to_lowercase().contains(&query_lower)
                    || task.description.to_lowercase().contains(&query_lower)
                    || task.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    fn save_tasks(&self) -> Result<(), SchedulerError> {
        let json = serde_json::to_string_pretty(&self.tasks)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }

    fn load_tasks(&mut self) -> Result<(), SchedulerError> {
        if !std::path::Path::new(&self.file_path).exists() {
            return Ok(()); // No file exists yet, start fresh
        }

        let contents = fs::read_to_string(&self.file_path)?;
        if contents.trim().is_empty() {
            return Ok(()); // Empty file, start fresh
        }

        self.tasks = serde_json::from_str(&contents)?;
        
        // Update next_id to be higher than any existing ID
        self.next_id = self.tasks.keys().max().unwrap_or(&0) + 1;
        
        Ok(())
    }
}

pub fn run_scheduler_background_thread(
    scheduler: Arc<Mutex<TaskScheduler>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        println!("Starting task scheduler background thread");
        
        // Get check interval from config
        let check_interval = get_config(|config| config.reminder.check_interval_seconds)
            .unwrap_or(30);
        
        loop {
            thread::sleep(Duration::from_secs(check_interval));
            
            println!("Checking for due reminders...");
            
            // This would integrate with the reminder system
            // For now, it's just a placeholder for the background processing
            if let Ok(scheduler) = scheduler.lock() {
                let tasks_with_reminders = scheduler.get_tasks_with_pending_reminders();
                if !tasks_with_reminders.is_empty() {
                    println!("Found {} tasks with pending reminders", tasks_with_reminders.len());
                    // Here we would trigger the reminder processing
                }
            }
        }
    })
} 