use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct Logger {
    log_file_path: String,
    console_output: bool,
    min_level: LogLevel,
}

impl Logger {
    pub fn new(log_file_path: &str, console_output: bool, min_level: LogLevel) -> Self {
        Self {
            log_file_path: log_file_path.to_string(),
            console_output,
            min_level,
        }
    }

    pub fn log(&self, level: LogLevel, module: &str, message: &str, metadata: Option<serde_json::Value>) {
        if !self.should_log(&level) {
            return;
        }

        let entry = LogEntry {
            timestamp: Utc::now().timestamp(),
            level: level.clone(),
            module: module.to_string(),
            message: message.to_string(),
            metadata,
        };

        // Console output
        if self.console_output {
            self.print_to_console(&entry);
        }

        // File output
        if let Err(e) = self.write_to_file(&entry) {
            eprintln!("Failed to write to log file: {}", e);
        }
    }

    pub fn debug(&self, module: &str, message: &str) {
        self.log(LogLevel::Debug, module, message, None);
    }

    pub fn info(&self, module: &str, message: &str) {
        self.log(LogLevel::Info, module, message, None);
    }

    pub fn warning(&self, module: &str, message: &str) {
        self.log(LogLevel::Warning, module, message, None);
    }

    pub fn error(&self, module: &str, message: &str) {
        self.log(LogLevel::Error, module, message, None);
    }

    pub fn info_with_metadata(&self, module: &str, message: &str, metadata: serde_json::Value) {
        self.log(LogLevel::Info, module, message, Some(metadata));
    }

    pub fn error_with_metadata(&self, module: &str, message: &str, metadata: serde_json::Value) {
        self.log(LogLevel::Error, module, message, Some(metadata));
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        use LogLevel::*;
        match (&self.min_level, level) {
            (Debug, _) => true,
            (Info, Debug) => false,
            (Info, _) => true,
            (Warning, Debug | Info) => false,
            (Warning, _) => true,
            (Error, Error) => true,
            (Error, _) => false,
        }
    }

    fn print_to_console(&self, entry: &LogEntry) {
        let local_time: DateTime<Local> = DateTime::from_timestamp(entry.timestamp, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
            .with_timezone(&Local::now().timezone());
        
        let level_str = match entry.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        };

        let color_code = match entry.level {
            LogLevel::Debug => "\x1b[36m",    // Cyan
            LogLevel::Info => "\x1b[32m",     // Green
            LogLevel::Warning => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m",    // Red
        };

        println!(
            "{}[{}] [{}] {}: {}\x1b[0m",
            color_code,
            local_time.format("%Y-%m-%d %H:%M:%S"),
            level_str,
            entry.module,
            entry.message
        );

        if let Some(ref metadata) = entry.metadata {
            println!("  Metadata: {}", serde_json::to_string_pretty(metadata).unwrap_or_default());
        }
    }

    fn write_to_file(&self, entry: &LogEntry) -> io::Result<()> {
        // Create log directory if it doesn't exist
        if let Some(parent) = Path::new(&self.log_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        let json_entry = serde_json::to_string(entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        writeln!(file, "{}", json_entry)?;
        file.flush()?;

        Ok(())
    }

    pub fn get_recent_logs(&self, count: usize) -> io::Result<Vec<LogEntry>> {
        if !Path::new(&self.log_file_path).exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.log_file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut entries = Vec::new();
        for line in lines.iter().rev().take(count) {
            if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                entries.push(entry);
            }
        }
        
        entries.reverse();
        Ok(entries)
    }

    pub fn clear_logs(&self) -> io::Result<()> {
        if Path::new(&self.log_file_path).exists() {
            std::fs::remove_file(&self.log_file_path)?;
        }
        Ok(())
    }
}

// Global logger instance
use std::sync::{Arc, Mutex, OnceLock};

static GLOBAL_LOGGER: OnceLock<Arc<Mutex<Logger>>> = OnceLock::new();

pub fn init_logger(log_file_path: &str, console_output: bool, min_level: LogLevel) {
    let logger = Logger::new(log_file_path, console_output, min_level);
    GLOBAL_LOGGER.set(Arc::new(Mutex::new(logger))).ok();
}

pub fn log_debug(module: &str, message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.debug(module, message);
        }
    }
}

pub fn log_info(module: &str, message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.info(module, message);
        }
    }
}

pub fn log_warning(module: &str, message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.warning(module, message);
        }
    }
}

pub fn log_error(module: &str, message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.error(module, message);
        }
    }
}

pub fn log_info_with_metadata(module: &str, message: &str, metadata: serde_json::Value) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.info_with_metadata(module, message, metadata);
        }
    }
}

pub fn log_error_with_metadata(module: &str, message: &str, metadata: serde_json::Value) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.error_with_metadata(module, message, metadata);
        }
    }
}

// Convenience macros
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::log_debug($module, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::log_info($module, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::log_warning($module, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::log_error($module, &format!($($arg)*))
    };
} 