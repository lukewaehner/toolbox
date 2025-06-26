use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub logging: LoggingConfig,
    pub reminder: ReminderConfig,
    pub security: SecurityConfig,
    pub ui: UiConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: String, // "debug", "info", "warning", "error"
    pub console_output: bool,
    pub file_path: String,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    pub check_interval_seconds: u64,
    pub max_retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub default_reminder_advance_minutes: i64,
    pub email_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub password_min_length: usize,
    pub encryption_key_rotation_days: u32,
    pub auto_lock_timeout_minutes: u32,
    pub require_confirmation_for_deletion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String, // "dark", "light", "auto"
    pub refresh_rate_ms: u64,
    pub show_status_bar: bool,
    pub compact_mode: bool,
    pub default_date_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub user_agent: String,
    pub speed_test_duration_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig {
                enabled: true,
                level: "info".to_string(),
                console_output: true,
                file_path: "logs/toolbox.log".to_string(),
                max_file_size_mb: 10,
                max_files: 5,
            },
            reminder: ReminderConfig {
                check_interval_seconds: 30,
                max_retry_attempts: 3,
                retry_delay_seconds: 300, // 5 minutes
                default_reminder_advance_minutes: 15,
                email_timeout_seconds: 30,
            },
            security: SecurityConfig {
                password_min_length: 8,
                encryption_key_rotation_days: 90,
                auto_lock_timeout_minutes: 30,
                require_confirmation_for_deletion: true,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                refresh_rate_ms: 1000,
                show_status_bar: true,
                compact_mode: false,
                default_date_format: "%Y-%m-%d %H:%M:%S".to_string(),
            },
            network: NetworkConfig {
                timeout_seconds: 30,
                max_retries: 3,
                user_agent: "Toolbox/1.0".to_string(),
                speed_test_duration_seconds: 10,
            },
        }
    }
}

pub struct ConfigManager {
    config_path: String,
    config: AppConfig,
}

impl ConfigManager {
    pub fn new(config_path: &str) -> Self {
        let mut manager = Self {
            config_path: config_path.to_string(),
            config: AppConfig::default(),
        };
        
        if let Err(e) = manager.load() {
            eprintln!("Failed to load config, using defaults: {}", e);
            // Save default config
            if let Err(e) = manager.save() {
                eprintln!("Failed to save default config: {}", e);
            }
        }
        
        manager
    }

    pub fn get(&self) -> &AppConfig {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.config_path).exists() {
            // Create config directory if it doesn't exist
            if let Some(parent) = Path::new(&self.config_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Create default config file
            self.config = AppConfig::default();
            self.save()?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)?;
        self.config = serde_json::from_str(&content)?;
        
        // Validate config
        self.validate()?;
        
        Ok(())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create config directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = AppConfig::default();
        self.save()?;
        Ok(())
    }

    pub fn update_logging(&mut self, enabled: bool, level: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.config.logging.enabled = enabled;
        self.config.logging.level = level.to_string();
        self.save()?;
        Ok(())
    }

    pub fn update_reminder_interval(&mut self, seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.config.reminder.check_interval_seconds = seconds;
        self.save()?;
        Ok(())
    }

    pub fn update_ui_theme(&mut self, theme: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.config.ui.theme = theme.to_string();
        self.save()?;
        Ok(())
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate logging level
        match self.config.logging.level.as_str() {
            "debug" | "info" | "warning" | "error" => {}
            _ => return Err("Invalid logging level".into()),
        }

        // Validate theme
        match self.config.ui.theme.as_str() {
            "dark" | "light" | "auto" => {}
            _ => return Err("Invalid UI theme".into()),
        }

        // Validate numeric ranges
        if self.config.security.password_min_length < 4 {
            return Err("Password minimum length too short".into());
        }

        if self.config.reminder.check_interval_seconds < 10 {
            return Err("Reminder check interval too short".into());
        }

        Ok(())
    }

    pub fn export_config(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn import_config(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let imported_config: AppConfig = serde_json::from_str(&content)?;
        
        // Temporarily set config to validate
        let old_config = self.config.clone();
        self.config = imported_config;
        
        match self.validate() {
            Ok(_) => {
                self.save()?;
                Ok(())
            }
            Err(e) => {
                // Restore old config on validation failure
                self.config = old_config;
                Err(e)
            }
        }
    }

    pub fn get_config_info(&self) -> String {
        format!(
            "Configuration Info:\n\
            - Config Path: {}\n\
            - Logging Level: {}\n\
            - Reminder Check Interval: {}s\n\
            - UI Theme: {}\n\
            - Network Timeout: {}s\n\
            - Auto-lock Timeout: {}min",
            self.config_path,
            self.config.logging.level,
            self.config.reminder.check_interval_seconds,
            self.config.ui.theme,
            self.config.network.timeout_seconds,
            self.config.security.auto_lock_timeout_minutes
        )
    }
}

// Global config manager
use std::sync::{Arc, Mutex, OnceLock};

static GLOBAL_CONFIG: OnceLock<Arc<Mutex<ConfigManager>>> = OnceLock::new();

pub fn init_config(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ConfigManager::new(config_path);
    GLOBAL_CONFIG.set(Arc::new(Mutex::new(config_manager)))
        .map_err(|_| "Failed to initialize global config")?;
    Ok(())
}

pub fn get_config<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&AppConfig) -> R,
{
    GLOBAL_CONFIG.get()?.lock().ok().map(|config| f(config.get()))
}

pub fn update_config<F>(f: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut ConfigManager) -> Result<(), Box<dyn std::error::Error>>,
{
    if let Some(config_manager) = GLOBAL_CONFIG.get() {
        if let Ok(mut config) = config_manager.lock() {
            f(&mut *config)?;
        }
    }
    Ok(())
} 