use once_cell::sync::Lazy;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use tui::style::Color;

// DARK_MODE and text color functions
pub static DARK_MODE: Lazy<bool> = Lazy::new(|| {
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

pub fn get_text_color() -> tui::style::Color {
    if *DARK_MODE {
        tui::style::Color::White
    } else {
        tui::style::Color::Black
    }
}

pub fn is_dark_mode() -> bool {
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

// Helper function to get color based on usage percentage
pub fn get_usage_color(usage: f32) -> Color {
    if usage < 60.0 {
        Color::Green
    } else if usage < 85.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

// Helper function for priority colors
pub fn get_priority_color(priority: &crate::task_scheduler::TaskPriority) -> Color {
    use crate::task_scheduler::TaskPriority;
    match *priority {
        TaskPriority::Low => Color::Blue,
        TaskPriority::Medium => Color::Green,
        TaskPriority::High => Color::Yellow,
        TaskPriority::Urgent => Color::Red,
    }
}

// Helper function for status colors
pub fn get_status_color(status: &crate::task_scheduler::TaskStatus) -> Color {
    use crate::task_scheduler::TaskStatus;
    match *status {
        TaskStatus::Pending => Color::Yellow,
        TaskStatus::InProgress => Color::Blue,
        TaskStatus::Completed => Color::Green,
        TaskStatus::Cancelled => Color::Gray,
    }
}
