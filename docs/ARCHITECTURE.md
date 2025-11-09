# Toolbox Architecture Documentation

This document describes the architecture and design patterns used in the Toolbox application.

## Table of Contents

- [Overview](#overview)
- [Architecture Pattern](#architecture-pattern)
- [Module Structure](#module-structure)
- [Data Flow](#data-flow)
- [Event-Driven Design](#event-driven-design)
- [Security Architecture](#security-architecture)
- [Concurrency Model](#concurrency-model)

## Overview

Toolbox is a terminal-based utility application built with Rust that provides multiple system administration and productivity tools. The application uses a modular architecture with clear separation of concerns.

### Technology Stack

- **Language**: Rust 2021 Edition
- **UI Framework**: TUI-rs + Crossterm
- **Encryption**: AES-256-CBC (aes, cbc crates)
- **System Monitoring**: sysinfo crate
- **Email**: lettre crate
- **Notifications**: notify-rust crate
- **Data Serialization**: serde + serde_json

## Architecture Pattern

The application follows a **Model-View-Controller (MVC)** pattern with event-driven architecture:

```
┌─────────────────────────────────────────────┐
│            Main Event Loop                   │
│  - Polls for user input                     │
│  - Updates UI based on state                │
│  - Dispatches events to handlers             │
└─────────────────────────────────────────────┘
              │
              ├──────────────┬──────────────┬──────────────┐
              ▼              ▼              ▼              ▼
      ┌──────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
      │   Password   │ │ Network  │ │  System  │ │   Task   │
      │   Manager    │ │  Tools   │ │ Utilities│ │Scheduler │
      └──────────────┘ └──────────┘ └──────────┘ └──────────┘
```

### MVC Components

Each feature module is organized into three layers:

1. **Model** (`model/`): Data structures and business logic
2. **View** (`view/`): UI rendering logic
3. **Controller** (`controller/`): Event handling and flow control

Example module structure:

```
modules/task_scheduler/
├── model/
│   ├── task.rs          # Task entity and methods
│   └── scheduler.rs     # Business logic
├── view/
│   └── mod.rs          # UI rendering
└── controller/
    └── mod.rs          # Input handling
```

## Module Structure

### Core Modules

#### `main.rs`

The application entry point that:
- Initializes the terminal UI
- Sets up the application state
- Starts the main event loop
- Handles graceful shutdown

#### `app_state.rs` (via main.rs)

Central state management:

```rust
struct AppState {
    active_menu: MenuItem,
    input_mode: InputMode,
    // Feature-specific state fields
    task_scheduler: Option<Arc<Mutex<TaskScheduler>>>,
    system_monitor: Option<Arc<Mutex<SystemMonitor>>>,
    // ... other state fields
}
```

### Feature Modules

#### Password Manager

**Purpose**: Secure storage and retrieval of login credentials

**Key Components**:
- Encryption/decryption using AES-256-CBC
- JSON serialization for data storage
- Environment variable based key management

**Data Flow**:
```
User Input → Validate → Encrypt → Write to File
                                      ↓
                                 passwords.json

File Read → Decrypt → Deserialize → Display
```

#### Network Tools

**Purpose**: Network diagnostics and speed testing

**Key Components**:
- Ping utility using system `ping` command
- HTTP-based download speed testing
- Multiple test servers for reliability

**Async Operations**:
- Speed tests run in background threads
- Results communicated via channels

#### System Utilities

**Purpose**: Real-time system monitoring and process management

**Key Components**:
- CPU, memory, and disk monitoring
- Process list with sorting and filtering
- Process management (view, kill)

**Update Strategy**:
- Periodic refresh based on configured interval
- Cached snapshots for UI rendering

#### Task Scheduler

**Purpose**: Task management with multi-channel reminders

**Key Components**:
- Task CRUD operations
- Reminder system (Email, SMS, Desktop notifications)
- Background thread for reminder checking
- Retry logic for failed reminders

**Background Processing**:
```
Main Thread                    Background Thread
    │                                 │
    │  Create Task                    │
    │──────────────────────────►      │
    │                                 │
    │                           Check Reminders
    │                           (every 30s)
    │                                 │
    │  ◄────────────────────────     │
    │  Send Notification             │
```

## Data Flow

### User Input Flow

```
User Keyboard Input
        ↓
Crossterm Event Polling
        ↓
Event Matching (KeyCode)
        ↓
State-Based Handler Dispatch
        ↓
Update AppState
        ↓
Re-render UI
```

### Example: Adding a Password

1. User navigates to Password Manager
2. Selects "Add Password"
3. `InputMode` changes to `Editing`
4. User fills in fields (service, username, password)
5. Presses Enter to save
6. Handler validates input
7. Calls `password_manager::save_password()`
8. Entry is encrypted and saved
9. Success message displayed
10. Returns to Normal mode

## Event-Driven Design

### Input Modes

The application uses different input modes to handle events:

```rust
enum InputMode {
    Normal,          // Navigation
    Editing,         // Text input
    Viewing,         // Read-only display
    EnterAddress,    // Network tool input
    SpeedTestRunning,// Async operation
    AddingTask,      // Task creation
    // ... other modes
}
```

Each mode has specific event handlers that determine how user input is processed.

### Event Handling Pattern

```rust
fn handle_input(key: KeyCode, app_state: &mut AppState) {
    match (app_state.active_menu, app_state.input_mode, key) {
        (MenuItem::PasswordManager, InputMode::Editing, KeyCode::Enter) => {
            save_current_password(app_state);
        }
        (MenuItem::NetworkTools, InputMode::Normal, KeyCode::Char('p')) => {
            start_ping_tool(app_state);
        }
        // ... other combinations
    }
}
```

## Security Architecture

### Password Manager Security

**Encryption Strategy**:

1. **Key Management**:
   - Key stored in environment variable (`ENCRYPTION_KEY`)
   - Key must be exactly 32 bytes (256 bits)
   - Loaded at runtime, never hardcoded

2. **Encryption Process**:
   ```
   Plaintext → AES-256-CBC → Base64 Encoding → File Storage
                    ↑
                Random IV (16 bytes)
   ```

3. **Storage Format**:
   ```
   [IV (16 bytes)][Encrypted Data (variable length)]
   ```

**Security Considerations**:
- Each encryption uses a unique random IV
- PKCS7 padding for block alignment
- No key derivation (user must provide strong key)
- Password file permissions should be restricted

### Network Security

- HTTPS used for speed test downloads
- TLS support for SMTP connections
- Email credentials stored in memory only (not logged)

## Concurrency Model

### Thread Safety

The application uses Rust's ownership system and standard library primitives for thread safety:

**Shared State**:
```rust
Arc<Mutex<TaskScheduler>>   // Shared across threads
Arc<Mutex<SystemMonitor>>   // Shared across threads
```

**Background Threads**:

1. **Task Scheduler Thread**:
   - Runs continuously in background
   - Checks for due reminders every 30 seconds
   - Sends notifications via email/desktop/SMS
   - Retries failed deliveries

2. **Speed Test Thread**:
   - Created on-demand when user starts speed test
   - Communicates results via channel
   - Terminates after completion

### Synchronization

**Pattern Used**:
```rust
// Acquire lock, perform operation, automatically release
{
    let mut scheduler = task_scheduler.lock().unwrap();
    scheduler.check_reminders();
    scheduler.process_retries();
} // Lock released here
```

**Message Passing**:
```rust
// Speed test uses channel for results
let (tx, rx) = mpsc::channel();
std::thread::spawn(move || {
    let result = perform_speed_test();
    tx.send(result).unwrap();
});

// Main thread receives result
if let Ok(result) = rx.try_recv() {
    display_result(result);
}
```

## Error Handling Strategy

### Error Types

Different modules use appropriate error types:

```rust
// IO errors for file operations
io::Result<T>

// Generic errors for external operations
Result<T, Box<dyn std::error::Error>>

// Custom errors for domain logic
Result<T, SchedulerError>
```

### Error Propagation

The application uses the `?` operator for clean error propagation:

```rust
pub fn save_password(entry: &PasswordEntry) -> io::Result<()> {
    let entries = load_passwords()?;  // Propagate error
    let json = serialize(&entries)?;   // Propagate error
    encrypt_and_save(&json)?;          // Propagate error
    Ok(())
}
```

### User-Facing Errors

Errors are converted to user-friendly messages:

```rust
match save_password(&entry) {
    Ok(()) => {
        app_state.status_message = Some("Password saved successfully");
    }
    Err(e) => {
        app_state.error_message = Some(format!("Failed to save: {}", e));
    }
}
```

## UI Rendering

### TUI Architecture

The UI uses the `tui-rs` crate with `crossterm` backend:

```
┌─────────────────────────────────────────┐
│         Terminal (Crossterm)            │
├─────────────────────────────────────────┤
│           TUI Framework                 │
│  - Layout management                    │
│  - Widget rendering                     │
│  - Style application                    │
├─────────────────────────────────────────┤
│        Application Components           │
│  - Main menu                            │
│  - Feature screens                      │
│  - Input modals                         │
│  - Status messages                      │
└─────────────────────────────────────────┘
```

### Rendering Flow

1. **Layout Calculation**: Divide screen into regions
2. **Widget Creation**: Build widgets with data
3. **Style Application**: Apply colors and formatting
4. **Render**: Draw to terminal buffer
5. **Flush**: Display on screen

### Update Strategy

- **Full Refresh**: On every event loop iteration
- **Efficient**: TUI framework only updates changed cells
- **60 FPS**: Event loop runs at ~16ms intervals

## Configuration Management

### Environment Variables

```
ENCRYPTION_KEY=<32-byte-key>  # Password encryption key
```

### Runtime Configuration

- Task storage: `tasks.json`
- Password storage: `passwords.json`
- Email config: Stored in task scheduler state
- SMS config: Stored in task scheduler state

## Extensibility

### Adding New Features

To add a new feature module:

1. Create module directory under `src/modules/`
2. Implement MVC structure:
   - `model/` - Data structures and logic
   - `view/` - UI rendering
   - `controller/` - Input handling
3. Add to `MenuItem` enum
4. Create UI drawing function
5. Add event handlers
6. Update main menu

### Adding New Reminder Types

To add a new reminder channel:

1. Add variant to `ReminderType` enum
2. Implement sending logic in task scheduler
3. Add UI option for selection
4. Update configuration if needed

## Performance Considerations

### Memory Usage

- Minimal allocations in event loop
- Reuse buffers where possible
- Drop large data structures when switching menus

### CPU Usage

- Sleep in event loop when no input
- Throttle system monitor updates
- Background threads sleep between checks

### File I/O

- Lazy loading of password file
- Debounced saves for task updates
- Atomic writes to prevent corruption

## Testing Strategy

### Unit Tests

- Pure functions tested in isolation
- Mock file system operations
- Test encryption/decryption round-trips

### Integration Tests

- Test complete workflows
- Verify state transitions
- Check error handling paths

### Manual Testing

- UI rendering on different terminal sizes
- Keyboard navigation
- Edge cases (empty data, long strings, etc.)

## Deployment

### Building

```bash
cargo build --release
```

### Running

```bash
./target/release/toolbox
```

### Requirements

- Linux/Unix system (for ping command)
- Terminal with color support
- Environment variables set (.env file)

## Future Improvements

Potential architectural enhancements:

1. **Plugin System**: Dynamic module loading
2. **Configuration Files**: TOML/JSON based config
3. **Database Backend**: SQLite for data storage
4. **REST API**: Optional HTTP server mode
5. **Web UI**: Browser-based interface option
6. **Logging Framework**: Structured logging to file
7. **Metrics**: Performance monitoring
8. **i18n**: Internationalization support

## Conclusion

The Toolbox architecture provides:

- **Modularity**: Clear separation between features
- **Maintainability**: Well-organized code structure
- **Security**: Strong encryption for sensitive data
- **Reliability**: Proper error handling throughout
- **Performance**: Efficient event-driven model
- **Extensibility**: Easy to add new features

This architecture serves as a solid foundation for a production-ready terminal utility application.
