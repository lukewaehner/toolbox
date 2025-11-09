# Toolbox API Documentation

This document provides detailed API documentation for the Toolbox application modules.

## Table of Contents

- [Password Manager](#password-manager)
- [Network Tools](#network-tools)
- [System Utilities](#system-utilities)
- [Task Scheduler](#task-scheduler)

## Password Manager

The password manager module provides secure storage and retrieval of login credentials using AES-256 encryption.

### Data Structures

#### `PasswordEntry`

Represents a single password entry.

```rust
pub struct PasswordEntry {
    pub service: String,   // Service name (e.g., "gmail.com")
    pub username: String,  // Username or email
    pub password: String,  // Password
}
```

### Functions

#### `save_password`

Saves a password entry to encrypted storage.

```rust
pub fn save_password(entry: &PasswordEntry) -> io::Result<()>
```

**Parameters:**
- `entry`: Reference to a `PasswordEntry` to save

**Returns:**
- `Ok(())` on success
- `Err(io::Error)` if saving fails

**Example:**
```rust
let entry = PasswordEntry {
    service: "github.com".to_string(),
    username: "user@example.com".to_string(),
    password: "my_secure_password".to_string(),
};

save_password(&entry).expect("Failed to save password");
```

#### `retrieve_password`

Retrieves all stored password entries.

```rust
pub fn retrieve_password() -> io::Result<Vec<PasswordEntry>>
```

**Returns:**
- `Ok(Vec<PasswordEntry>)` containing all stored passwords
- `Err(io::Error)` if retrieval fails

**Example:**
```rust
let entries = retrieve_password().expect("Failed to retrieve passwords");
for entry in entries {
    println!("Service: {}, Username: {}", entry.service, entry.username);
}
```

### Security Considerations

1. **Encryption Key**: Must be set in the `ENCRYPTION_KEY` environment variable
2. **Key Length**: Must be exactly 32 bytes (256 bits)
3. **Encryption Method**: AES-256-CBC with PKCS7 padding
4. **IV**: A new random IV is generated for each encryption operation
5. **Storage**: Encrypted data is stored in `passwords.json`

### Setup

Create a `.env` file in the project root:

```
ENCRYPTION_KEY=your_32_character_secret_key_here
```

## Network Tools

The network tools module provides utilities for network diagnostics and speed testing.

### Data Structures

#### `PingResult`

Contains statistics from a ping operation.

```rust
pub struct PingResult {
    packets_transmitted: u32,
    packets_received: u32,
    packet_loss: f32,
    time: u32,
    round_trip_min: f32,
    round_trip_avg: f32,
    round_trip_max: f32,
    round_trip_mdev: f32,
}
```

#### `SpeedTestResult`

Contains results from a download speed test.

```rust
pub struct SpeedTestResult {
    pub speed: f64,        // Download speed in bits per second
    pub duration: f64,     // Test duration in seconds
    pub bytes: u64,        // Total bytes downloaded
    pub status: String,    // Status message
    pub test_type: String, // Type of test performed
}
```

### Functions

#### `ping`

Performs a ping test to a specified address.

```rust
pub fn ping(address: &str) -> Result<PingResult, Box<dyn std::error::Error>>
```

**Parameters:**
- `address`: IP address or hostname to ping

**Returns:**
- `Ok(PingResult)` with statistics
- `Err` if the ping fails

**Example:**
```rust
let result = ping("8.8.8.8").expect("Ping failed");
println!("Packet loss: {}%", result.packet_loss);
println!("Average RTT: {} ms", result.round_trip_avg);
```

#### `efficient_speed_test`

Performs a download speed test using an efficient streaming approach.

```rust
pub fn efficient_speed_test() -> Result<SpeedTestResult, Box<dyn std::error::Error>>
```

**Returns:**
- `Ok(SpeedTestResult)` with speed measurements
- `Err` if the test fails

**Example:**
```rust
let result = efficient_speed_test().expect("Speed test failed");
let mbps = result.speed / 1_000_000.0;
println!("Download speed: {:.2} Mbps", mbps);
```

#### `multi_file_speed_test`

Performs a multi-file download test for more accurate results.

```rust
pub fn multi_file_speed_test() -> Result<SpeedTestResult, Box<dyn std::error::Error>>
```

#### `parallel_speed_test`

Performs parallel downloads for comprehensive speed testing.

```rust
pub fn parallel_speed_test() -> Result<SpeedTestResult, Box<dyn std::error::Error>>
```

## System Utilities

The system utilities module provides real-time system monitoring and process management.

### Data Structures

#### `SystemMonitor`

Monitors system resources including CPU, memory, disk, and processes.

```rust
pub struct SystemMonitor {
    // Internal fields for system monitoring
}
```

#### `SystemSnapshot`

A snapshot of current system state.

```rust
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub processes: Vec<ProcessInfo>,
}
```

### Functions

#### `SystemMonitor::new`

Creates a new system monitor instance.

```rust
pub fn new() -> Self
```

**Example:**
```rust
let monitor = SystemMonitor::new();
```

#### `refresh`

Refreshes system information.

```rust
pub fn refresh(&mut self)
```

#### `snapshot`

Gets a snapshot of current system state.

```rust
pub fn snapshot(&self) -> &SystemSnapshot
```

**Example:**
```rust
let mut monitor = SystemMonitor::new();
monitor.refresh();
let snapshot = monitor.snapshot();
println!("CPU Usage: {:.1}%", snapshot.cpu_usage);
```

## Task Scheduler

The task scheduler module provides task management with reminders via email, SMS, and desktop notifications.

### Data Structures

#### `Task`

Represents a scheduled task.

```rust
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<i64>,
    pub tags: Vec<String>,
    pub reminders: Vec<Reminder>,
}
```

#### `TaskPriority`

Priority levels for tasks.

```rust
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}
```

#### `TaskStatus`

Status of a task.

```rust
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}
```

#### `ReminderType`

Types of reminders.

```rust
pub enum ReminderType {
    Email,
    Notification,
    SMS,
    Both,      // Email + Notification
    All,       // Email + SMS + Notification
}
```

### Functions

#### `TaskScheduler::new`

Creates a new task scheduler instance.

```rust
pub fn new(file_path: &str) -> Self
```

**Parameters:**
- `file_path`: Path to the task storage file

**Example:**
```rust
let scheduler = TaskScheduler::new("tasks.json");
```

#### `add_task`

Adds a new task to the scheduler.

```rust
pub fn add_task(&mut self, task: Task) -> Result<u32, SchedulerError>
```

**Parameters:**
- `task`: The task to add

**Returns:**
- `Ok(u32)` with the task ID
- `Err(SchedulerError)` if adding fails

#### `get_task`

Retrieves a task by ID.

```rust
pub fn get_task(&self, id: u32) -> Option<&Task>
```

#### `update_task_status`

Updates the status of a task.

```rust
pub fn update_task_status(&mut self, id: u32, status: TaskStatus) -> Result<(), SchedulerError>
```

#### `add_reminder`

Adds a reminder to a task.

```rust
pub fn add_reminder(
    &mut self,
    task_id: u32,
    reminder_time: i64,
    reminder_type: ReminderType
) -> Result<(), SchedulerError>
```

**Example:**
```rust
use chrono::Utc;

let mut scheduler = TaskScheduler::new("tasks.json");
let task = Task {
    id: 0,
    title: "Complete project".to_string(),
    description: "Finish the Rust project".to_string(),
    status: TaskStatus::Pending,
    priority: TaskPriority::High,
    due_date: Some(Utc::now().timestamp() + 86400), // Due in 24 hours
    tags: vec!["work".to_string()],
    reminders: vec![],
};

let task_id = scheduler.add_task(task).expect("Failed to add task");

// Add an email reminder 1 hour before due date
scheduler.add_reminder(
    task_id,
    Utc::now().timestamp() + 82800,
    ReminderType::Email
).expect("Failed to add reminder");
```

### Email Configuration

Email reminders require SMTP configuration:

```rust
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}
```

Set up email configuration through the UI or by calling:

```rust
scheduler.set_email_config(config);
```

## Error Handling

All modules use Rust's `Result` type for error handling. Common error types include:

- `io::Error` - File system and I/O operations
- `Box<dyn std::error::Error>` - Generic errors
- Module-specific error types (e.g., `SchedulerError`)

Always handle errors appropriately:

```rust
match save_password(&entry) {
    Ok(()) => println!("Password saved successfully"),
    Err(e) => eprintln!("Failed to save password: {}", e),
}
```

## Thread Safety

The application uses `Arc<Mutex<T>>` for shared state across threads:

- `TaskScheduler` is wrapped in `Arc<Mutex<>>` for background processing
- `SystemMonitor` is wrapped in `Arc<Mutex<>>` for real-time updates

Example:

```rust
use std::sync::{Arc, Mutex};

let scheduler = Arc::new(Mutex::new(TaskScheduler::new("tasks.json")));
let scheduler_clone = scheduler.clone();

// Use in background thread
std::thread::spawn(move || {
    let mut sched = scheduler_clone.lock().unwrap();
    sched.check_reminders();
});
```

## Best Practices

1. **Always handle errors**: Don't unwrap unless in example code
2. **Use environment variables**: For sensitive configuration
3. **Close resources**: The application handles cleanup automatically
4. **Thread-safe access**: Use proper locking when accessing shared state
5. **Validate input**: Always validate user input before processing
6. **Test encryption**: Ensure encryption key is properly set before using password manager
