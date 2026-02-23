# Coding Conventions

**Analysis Date:** 2026-02-22

## Naming Patterns

**Files:**
- Snake_case for module files: `network_tools.rs`, `password_manager.rs`, `task_scheduler.rs`, `system_utilities.rs`
- Organized into logical directories by feature: `src/ui/`, `src/modules/`, `src/core/`, `src/shared/`
- Structure follows feature-based organization with separate controller/view/model layers in `src/modules/`

**Functions:**
- Snake_case for all functions: `ping()`, `retrieve_password()`, `save_password()`, `parse_ping_output()`
- Getter/setter pattern: `get_encryption_key()`, `set_email_config()`
- Predicate functions use `is_` or `has_` prefix: `is_due()`, `has_pending_reminders()`
- Factory functions use descriptive names: `SpeedTestResult::status()`, `SpeedTestResult::error()`

**Variables:**
- Snake_case for all variable names: `min_level`, `packet_loss`, `round_trip_avg`, `file_path`
- Use descriptive names reflecting purpose: `packets_transmitted`, `memory_used`, `swap_used`
- Configuration fields use descriptive names: `check_interval_seconds`, `max_retry_attempts`, `email_timeout_seconds`

**Types:**
- PascalCase for struct names: `PingResult`, `PasswordEntry`, `TaskScheduler`, `SystemMonitor`, `LogEntry`
- PascalCase for enum variants: `TaskPriority::High`, `TaskStatus::Pending`, `LogLevel::Error`, `ReminderType::Email`
- Enum naming is descriptive: `TaskPriority`, `TaskStatus`, `ReminderType`, `LogLevel`

**Constants:**
- SCREAMING_SNAKE_CASE for module constants: `FILE_PATH = "passwords.json"`

## Code Style

**Formatting:**
- Rust standard formatting (2025 edition)
- Edition 2021 specified in `Cargo.toml`
- Blocks and formatting follow standard Rust conventions

**Linting:**
- No explicit clippy.toml or .rustfmt.toml detected - using defaults
- Implied adherence to standard Rust idioms based on code patterns
- Error handling uses `Result<T, E>` types consistently throughout

## Import Organization

**Order:**
1. Crate imports first: `use chrono::...`, `use serde::...`, `use std::...`
2. External crate imports: `use aes::Aes256`, `use lettre::...`, `use sysinfo::...`, `use tui::...`
3. Standard library imports: `use std::io`, `use std::fs`, `use std::sync::...`
4. Local module imports: `use crate::task_scheduler::...`, `use network_tools::...`

**Path Aliases:**
- Relative imports for external crates: `use chrono::Local`
- Absolute imports within crate: `use crate::task_scheduler::TaskScheduler`
- Convenience re-exports in mod.rs: `pub use models::*`, `pub use utils::*`

**Example from `src/network_tools.rs`:**
```rust
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
```

## Error Handling

**Patterns:**
- Standard Rust error type pattern: `Result<T, Box<dyn std::error::Error>>`
- Used across all fallible operations: `ping()`, `retrieve_password()`, `measure_speed()`
- Early return with `?` operator for error propagation

**Error Conversion:**
- Explicit error mapping using `.map_err()` with descriptive messages
- Context-specific error creation: `io::Error::new(io::ErrorKind::InvalidInput, "message")`
- File I/O errors explicitly handled with custom error types

**Example from `src/password_manager.rs`:**
```rust
fn decrypt(data: &[u8]) -> io::Result<Vec<u8>> {
    let key = get_encryption_key().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "Failed to get encryption key")
    })?;
    // Decode base64
    let decoded = general_purpose::STANDARD.decode(data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Base64 decoding failed: {:?}", e),
        )
    })?;
    // ...
}
```

## Logging

**Framework:** Custom logger implementation

**Module:** `src/logger.rs`

**Patterns:**
- Logger struct takes `log_file_path`, `console_output`, and `min_level` parameters
- Methods for each log level: `debug()`, `info()`, `warning()`, `error()`
- Metadata support: `info_with_metadata()`, `error_with_metadata()`
- Log entries are serialized as JSON with timestamp, level, module, message, and metadata
- Filtering based on log level prevents unnecessary logging

**Example usage from `src/logger.rs`:**
```rust
pub fn info(&self, module: &str, message: &str) {
    self.log(LogLevel::Info, module, message, None);
}

pub fn error_with_metadata(&self, module: &str, message: &str, metadata: serde_json::Value) {
    self.log(LogLevel::Error, module, message, Some(metadata));
}
```

## Comments

**When to Comment:**
- Module-level documentation using doc comments (`//!`) at top of file
- Function documentation with examples in doc comments
- Inline comments for complex logic or non-obvious code
- No verbose comments for self-explanatory code

**Doc Comments Pattern:**
- Three-slash doc comments (`///`) for functions and types
- Includes summary, Examples section with `# Examples` and `# Returns` sections
- Error documentation in `# Errors` sections
- Visible in generated documentation

**Example from `src/network_tools.rs`:**
```rust
/// Performs a ping test to a specified address
///
/// Sends 4 ICMP echo request packets to the target address and returns statistics.
///
/// # Arguments
///
/// * `address` - The IP address or hostname to ping
///
/// # Returns
///
/// Returns a `PingResult` containing packet transmission statistics and RTT measurements.
///
/// # Errors
///
/// Returns an error if:
/// - The ping command fails to execute
/// - The output cannot be parsed
/// - The target is unreachable
pub fn ping(address: &str) -> Result<PingResult, Box<dyn std::error::Error>> {
```

## Function Design

**Size:** Functions are generally focused and modular, ranging from 5-50 lines for core logic

**Parameters:**
- Functions take owned or borrowed references as appropriate
- String parameters often use `&str` rather than `String`
- Struct methods use `&self` or `&mut self` as needed
- Builder pattern not extensively used, direct construction preferred

**Return Values:**
- Fallible operations return `Result<T, E>`
- Single output values returned directly
- Collections returned as `Vec<T>` or owned types
- Status/enum types returned for state queries: `TaskStatus`, `LogLevel`

**Example from `src/task_scheduler.rs`:**
```rust
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

    pub fn is_due(&self) -> bool {
        let now = Utc::now().timestamp();
        self.due_date <= now
    }
}
```

## Module Design

**Exports:**
- Public items marked with `pub` keyword
- Structs, enums, and key functions explicitly public
- Implementation details kept private using private modules
- Re-exports used in mod.rs for convenient access

**Barrel Files:**
- Used in strategic locations: `src/shared/mod.rs`, `src/ui/mod.rs`
- Pattern: `pub use models::*; pub use utils::*;`
- Simplifies external imports and provides convenient namespace
- Clearly separates public API from internal implementation

**Module Organization in `src/modules/`:**
- Feature-specific directories with model/controller/view separation
- Each feature module has: `mod.rs`, `model/mod.rs`, `controller/mod.rs`, `view/mod.rs`
- Top-level re-exports in feature mod.rs make internal structure transparent

**Example from `src/shared/mod.rs`:**
```rust
pub mod models;
pub mod utils;

// Re-export for convenience
pub use models::*;
pub use utils::*;
```

## Struct and Enum Design

**Derive Macros:**
- Consistent use of `#[derive(Debug, Clone, Serialize, Deserialize)]` for data structs
- Configuration structs use `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Enum types include Debug and Clone when appropriate

**Serde Usage:**
- All serializable types derive `Serialize` and `Deserialize`
- Used for configuration files, password storage, JSON parsing
- Types suitable for JSON encoding

**Example from `src/config.rs`:**
```rust
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
    pub level: String,
    pub console_output: bool,
    pub file_path: String,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}
```

## Type Annotations

**Explicit Type Annotations:**
- Used when type cannot be inferred: `Vec::<PasswordEntry>::new()`
- Used for clarity in complex generics
- Function signatures include explicit return types

**Type Inference:**
- Allowed for simple local variables where type is obvious from context
- Collections and Result types often need explicit annotation when empty

---

*Convention analysis: 2026-02-22*
