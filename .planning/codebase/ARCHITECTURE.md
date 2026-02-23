# Architecture

**Analysis Date:** 2026-02-22

## Pattern Overview

**Overall:** MVC (Model-View-Controller) with Modular Feature Architecture

**Key Characteristics:**
- Terminal User Interface (TUI) built with `tui` and `crossterm` crates
- Event-driven architecture with mode-based input handling
- Four feature modules (PasswordManager, NetworkTools, SystemUtilities, TaskScheduler) with independent MVC layers
- Centralized application state (`AppState`) that flows through the event loop
- Background thread model for long-running tasks (speed tests, task scheduler reminders)

## Layers

**Presentation (View):**
- Purpose: Render TUI components and layouts to terminal
- Location: `src/modules/*/view/` and `src/ui/`
- Contains: Drawing functions, layout definitions, widget rendering
- Depends on: `AppState`, tui framework, shared utilities
- Used by: Main event loop in `src/app.rs` or `src/main.rs`

**Application (Controller):**
- Purpose: Handle user input events and state transitions
- Location: `src/modules/*/controller/` and `src/handlers/`
- Contains: Keyboard event handlers, navigation logic, mode transitions
- Depends on: `AppState`, Model layer
- Used by: Main event loop for keyboard/input routing

**Business Logic (Model):**
- Purpose: Core functionality and data management for each feature
- Location: `src/modules/*/model/`
- Contains: Data structures (Task, PasswordEntry, PingResult, SystemSnapshot), encryption/decryption, system calls, calculations
- Depends on: External crates (sysinfo, lettre, aes, regex, serde)
- Used by: Controllers

**Shared/Cross-Module:**
- Purpose: Common data types and utilities
- Location: `src/shared/models/`, `src/shared/utils/`
- Contains: Ping result models, common helper functions, color utilities
- Depends on: Standard library, serde
- Used by: All modules

## Data Flow

**Main Application Loop:**

1. User presses key → `crossterm` event handler captures KeyEvent
2. Event routed to appropriate handler based on current `InputMode` and active menu
3. Handler (in `src/handlers/` or `src/modules/*/controller/`) updates `AppState`
4. State changes trigger model operations (encryption, network calls, system queries)
5. View layer renders UI based on updated `AppState`
6. Terminal is refreshed with new UI

**Password Manager Flow:**

1. User selects "PasswordManager" menu item
2. Input mode switches to `Editing`
3. Key presses populate `service`, `username`, `password` fields in `AppState`
4. On Enter: `src/modules/password_manager/model/mod.rs::save_password()` encrypts and writes to `passwords.json`
5. View renders confirmation message

**Network Tools Flow:**

1. User selects tool (ping) → mode becomes `EnterAddress`
2. Address input captured into `app_state.address`
3. On Enter: `src/modules/network_tools/model/mod.rs::ping()` spawns ping command, parses output
4. Result stored in `app_state.result` as JSON string
5. View renders `draw_view_results()` with parsed ping statistics

**System Utilities Flow:**

1. `SystemMonitor` continuously runs in background (spawned at startup)
2. Periodically captures system snapshots (`SystemSnapshot`)
3. User navigates views (Overview → CpuDetails, ProcessList, etc.)
4. View renders current `app_state.system_snapshot` based on active `system_view_mode`
5. If user selects process to kill: confirmation dialogue → signal sent via Command

**Task Scheduler Flow:**

1. `TaskScheduler` instance created at startup and stored in `AppState`
2. Background thread `run_scheduler_background_thread()` periodically checks due dates and reminders
3. User creates/edits tasks through input modes
4. Reminders sent via email (lettre) or system notifications (notify-rust)
5. Task data persisted to `tasks.json`

**State Management:**

- Single centralized `AppState` struct holds all application state
- State updated through event handlers
- State passed as mutable reference through render pipeline
- No global mutable state (uses Arc<Mutex<>> for background tasks)

## Key Abstractions

**AppState:**
- Purpose: Central state container for entire application
- Examples: `src/app.rs` lines 158-221, `src/models/app_state.rs`
- Pattern: Single source of truth; passed through event loop; updated by handlers; consumed by views

**InputMode Enum:**
- Purpose: Determines how keyboard input is interpreted
- Examples: `Normal`, `Editing`, `Viewing`, `EnterAddress`, `AddingTask`, `ConfiguringEmail`
- Pattern: Mode-based input handling; different modes route to different handlers; mode transitions defined in handlers

**MenuItem Enum:**
- Purpose: Represents top-level features/menu selections
- Examples: `Main`, `PasswordManager`, `NetworkTools`, `SystemUtilities`, `TaskScheduler`
- Pattern: Active menu controls which feature module is displayed; drives conditional rendering

**Module Structure (Repeated for each feature):**
- Model: Core data structures and operations (encryption, system calls)
- View: TUI rendering functions for that feature
- Controller: Event handlers specific to that feature
- Pattern: Isolates feature logic; enables independent testing and modification

**Task/Reminder Model:**
- Purpose: Manages reminders with retry logic and error tracking
- Examples: `src/modules/task_scheduler/model/task.rs` (Task, Reminder structs)
- Pattern: Serializable structs; background thread monitors and executes; supports multiple reminder types (Email, Notification, SMS, Both, All)

**SystemMonitor:**
- Purpose: Periodic system information collection
- Examples: `src/modules/system_utilities/model/mod.rs` (SystemSnapshot, SystemHistory)
- Pattern: Captures immutable snapshots; UI reads latest snapshot; supports historical data for trends

## Entry Points

**Application Startup:**
- Location: `src/app.rs::main()` (primary) or `src/main.rs::main()` (alternative)
- Triggers: Binary execution
- Responsibilities:
  - Initialize terminal (raw mode, alternate screen)
  - Create TaskScheduler and spawn background thread
  - Setup signal handling (Ctrl+C)
  - Initialize AppState with defaults
  - Enter main event loop via `run_app()`

**Main Event Loop:**
- Location: `src/app.rs::run_app()` (lines 250+)
- Triggers: Initialization complete
- Responsibilities:
  - Poll crossterm events in tight loop
  - Route events to mode-specific handlers
  - Update AppState
  - Call render pipeline
  - Check running flag for exit condition

**Feature Handlers:**
- Location: `src/handlers/` (legacy) or `src/modules/*/controller/` (MVC)
- Triggers: Specific InputMode or MenuItem active + keyboard event
- Responsibilities:
  - Parse keyboard input
  - Validate user input
  - Call model methods
  - Update AppState fields
  - Transition to new InputMode/MenuItem

**Render Pipeline:**
- Location: Various `draw_*` functions in view modules
- Triggers: Every event loop iteration
- Responsibilities:
  - Read current AppState
  - Calculate layout using tui constraints
  - Render widgets (tables, paragraphs, gauges)
  - Display error/status messages
  - Clear and refresh terminal

## Error Handling

**Strategy:** Explicit error propagation with fallback UI display

**Patterns:**

- Result types used throughout: `Result<T, Box<dyn std::error::Error>>`
- Errors caught in handlers and stored in `AppState::error_message`
- Error message displayed in UI and auto-cleared after timeout or user action
- Examples:
  - Encryption failure → error message + mode reset to Normal
  - Network call failure (ping) → error stored and displayed
  - File I/O (passwords.json, tasks.json) → error handling with re-initialization

## Cross-Cutting Concerns

**Logging:**
- Logger module in `src/logger.rs` with file and console output
- Custom LogLevel enum (Debug, Info, Warning, Error)
- Used for debugging; not critical for feature functionality

**Validation:**
- Input validation in handlers (empty string checks, date parsing)
- Data validation in models (encryption key checks, command execution)
- No centralized validation framework; ad-hoc per feature

**Authentication:**
- No user authentication
- Encryption key derived from environment variable (`ENCRYPTION_KEY` in `.env`)
- Task scheduler supports email/SMTP authentication (username/password stored in AppState and email config)
- Email credentials configured through UI (`ConfiguringEmail` mode)

**Concurrency:**
- Background thread for TaskScheduler reminders
- Arc<Mutex<>> used for shared state (SystemMonitor, TaskScheduler)
- Channels (mpsc) used for speed test results
- No async/await; blocking threads with message passing
