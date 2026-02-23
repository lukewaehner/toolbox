# Codebase Structure

**Analysis Date:** 2026-02-22

## Directory Layout

```
toolbox/
├── src/
│   ├── main.rs                          # Alternative entry point (monolithic design)
│   ├── app.rs                           # Primary entry point + event loop
│   ├── modules/                         # Primary MVC module structure
│   │   ├── mod.rs                       # Module exports
│   │   ├── network_tools/
│   │   │   ├── mod.rs                   # Feature module re-exports
│   │   │   ├── model/                   # Network operations, ping parsing
│   │   │   ├── view/                    # TUI rendering for network tools
│   │   │   └── controller/              # Input handlers for network tools
│   │   ├── password_manager/
│   │   │   ├── mod.rs
│   │   │   ├── model/                   # AES-256 encryption, password storage
│   │   │   ├── view/                    # Password entry forms
│   │   │   └── controller/              # Input handlers for password entry
│   │   ├── system_utilities/
│   │   │   ├── mod.rs
│   │   │   ├── model/                   # sysinfo integration, system snapshots
│   │   │   ├── view/                    # System monitor UI rendering
│   │   │   └── controller/              # Input handlers for system views
│   │   └── task_scheduler/
│   │       ├── mod.rs
│   │       ├── model/
│   │       │   ├── mod.rs               # Scheduler core
│   │       │   ├── task.rs              # Task and Reminder types
│   │       │   └── scheduler.rs         # TaskScheduler implementation
│   │       ├── view/                    # Task list UI, reminder config
│   │       └── controller/              # Input handlers for task management
│   ├── handlers/                        # Legacy event handlers (may be deprecated)
│   │   ├── mod.rs
│   │   ├── normal_mode.rs
│   │   ├── network_tools.rs
│   │   ├── password_manager.rs
│   │   ├── system_utilities.rs
│   │   └── task_scheduler.rs
│   ├── models/                          # Legacy model definitions
│   │   ├── mod.rs
│   │   ├── app_state.rs                 # AppState struct definition
│   │   └── ping_result.rs               # Ping result model
│   ├── shared/                          # Cross-module utilities
│   │   ├── mod.rs
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── app_state.rs             # Shared AppState model
│   │   │   └── ping_result.rs           # Shared PingResult model
│   │   └── utils/
│   │       ├── mod.rs
│   │       └── common.rs                # Helper functions (color utilities, etc.)
│   ├── ui/                              # Legacy UI modules
│   │   ├── mod.rs
│   │   ├── main_menu.rs
│   │   ├── network_tools.rs
│   │   ├── password_manager.rs
│   │   ├── system_utilities.rs
│   │   ├── task_scheduler.rs
│   │   └── common.rs
│   ├── core/                            # Infrastructure/config
│   │   ├── mod.rs
│   │   ├── backup/                      # Backup functionality
│   │   ├── config/                      # Configuration management
│   │   └── logging/                     # Logging infrastructure
│   ├── logger.rs                        # Logging implementation
│   ├── config.rs                        # Configuration loading
│   ├── backup.rs                        # Backup utilities
│   ├── ping_result.rs                   # Ping result definition
│   └── backups/
│       └── main.bk.rs                   # Backup of main implementation
├── Cargo.toml                           # Rust dependencies
├── Cargo.lock                           # Dependency lock file
├── tasks.json                           # Task persistence file
├── passwords.json                       # Password storage (encrypted)
├── .env                                 # Environment configuration
├── .planning/
│   └── codebase/                        # Architecture documentation
├── docs/                                # Additional documentation
└── README.md                            # Project overview
```

## Directory Purposes

**`src/`:**
- Purpose: All source code
- Contains: Rust modules, entry points, feature implementations

**`src/modules/`:**
- Purpose: Primary application architecture using MVC pattern
- Contains: Four feature modules (PasswordManager, NetworkTools, SystemUtilities, TaskScheduler)
- Key files: Each module has `mod.rs`, `model/`, `view/`, `controller/` subdirectories

**`src/modules/{feature}/model/`:**
- Purpose: Core business logic and data models for the feature
- Contains: Data structures, encryption/decryption, system calls, calculations
- Examples:
  - `src/modules/password_manager/model/mod.rs`: encryption with AES-256, password persistence
  - `src/modules/network_tools/model/mod.rs`: ping command execution, output parsing
  - `src/modules/system_utilities/model/mod.rs`: system information collection via sysinfo
  - `src/modules/task_scheduler/model/task.rs`: Task and Reminder types

**`src/modules/{feature}/view/`:**
- Purpose: Terminal User Interface rendering for the feature
- Contains: TUI drawing functions, layout definitions, widget configuration
- Pattern: Functions named `draw_*`; receive `AppState` and Frame; render to terminal

**`src/modules/{feature}/controller/`:**
- Purpose: Input event handling and state transitions
- Contains: Keyboard event handlers, mode transitions, validation
- Pattern: Handle user input → update AppState → trigger model operations

**`src/handlers/` (Legacy):**
- Purpose: Original event handler organization (may be deprecated in favor of modular controllers)
- Contains: Handler functions for different input modes
- Status: Partially overlaps with `src/modules/*/controller/`

**`src/models/`:**
- Purpose: Central data structure definitions (legacy location)
- Contains: AppState, PingResult
- Note: Some duplication with `src/shared/models/`

**`src/shared/`:**
- Purpose: Cross-module utilities and models
- Contains: Common data types, helper functions, color utilities
- Key files:
  - `src/shared/models/app_state.rs`: AppState definition
  - `src/shared/utils/common.rs`: Helper functions like `get_text_color()`

**`src/ui/` (Legacy):**
- Purpose: Original UI module organization (may be deprecated)
- Contains: Drawing functions for main menu and features

**`src/core/`:**
- Purpose: Infrastructure and configuration
- Contains: Backup, config management, logging setup
- Status: Partially used; logging in `src/logger.rs` may be primary

**Root Level Files:**
- `Cargo.toml`: Rust package manifest with dependencies
- `Cargo.lock`: Locked dependency versions
- `tasks.json`: Persisted task data (created at runtime)
- `passwords.json`: Encrypted password storage (created at runtime)
- `.env`: Environment configuration (contains ENCRYPTION_KEY, etc.)
- `README.md`: Project documentation

## Key File Locations

**Entry Points:**
- `src/app.rs`: Primary entry point with `main()` and `run_app()` event loop
- `src/main.rs`: Alternative monolithic entry point
- Both contain duplicate implementations of the application structure

**Configuration:**
- `Cargo.toml`: Dependencies and package metadata (lines 1-25)
- `.env`: Environment variables (encryption key, etc.)
- `src/config.rs`: Configuration initialization logic
- `src/core/config/mod.rs`: Additional config management

**Core Logic:**

Password Manager:
- `src/modules/password_manager/model/mod.rs`: AES-256 encryption/decryption, file I/O

Network Tools:
- `src/modules/network_tools/model/mod.rs`: Ping execution, output parsing with regex

System Utilities:
- `src/modules/system_utilities/model/mod.rs`: System resource collection (SystemSnapshot)

Task Scheduler:
- `src/modules/task_scheduler/model/scheduler.rs`: Scheduler logic
- `src/modules/task_scheduler/model/task.rs`: Task and Reminder types
- `src/task_scheduler.rs`: Background thread implementation

**Testing:**
- `src/password_manager.rs`: Contains test module (`mod tests`)
- No dedicated test files; tests inline with source

## Naming Conventions

**Files:**
- Module aggregators: `mod.rs`
- Feature implementations: `{feature_name}.rs` (e.g., `network_tools.rs`, `task_scheduler.rs`)
- Structural modules: lowercase with underscores (e.g., `system_utilities`, `password_manager`)
- Backup/legacy: `*.bk.rs` suffix
- Configuration: lowercase (e.g., `config.rs`, `logger.rs`)

**Directories:**
- Feature modules: lowercase, underscores (e.g., `network_tools`, `password_manager`)
- Layer directories: lowercase (e.g., `model`, `view`, `controller`)
- Shared code: `shared`, `core`, `handlers`, `ui`
- Infrastructure: `core`, `docs`, `.planning`

**Code Symbols:**

Enums:
- `InputMode`: Application input state
- `MenuItem`: Top-level menu options
- `SystemViewMode`: System monitor view options
- `TaskPriority`: Task priority levels
- `TaskStatus`: Task states
- `ReminderType`: Reminder delivery methods

Structs:
- `AppState`: Central application state
- `PasswordEntry`: Password with metadata
- `PingResult`: Ping command results
- `SystemSnapshot`: System resource snapshot
- `Task`: Task definition with metadata
- `Reminder`: Reminder configuration

Functions:
- Handlers: `handle_{feature}_{action}`
- Rendering: `draw_{feature}_{view}`
- Model operations: `{operation}` (e.g., `ping()`, `encrypt()`, `save_password()`)

## Where to Add New Code

**New Feature:**
- Primary code: Create `src/modules/{feature_name}/` with `model/`, `view/`, `controller/` subdirectories
- Models: `src/modules/{feature_name}/model/mod.rs` for data structures and operations
- Views: `src/modules/{feature_name}/view/mod.rs` for TUI rendering
- Controllers: `src/modules/{feature_name}/controller/mod.rs` for input handling
- Tests: Add test module inline or in `mod.rs` file
- Add feature export to `src/modules/mod.rs`

**New Component/Module within existing feature:**
- Implementation: `src/modules/{feature}/model/` for new data types
- Rendering: `src/modules/{feature}/view/` for UI functions
- Input handling: `src/modules/{feature}/controller/` for event handlers
- State: Add fields to relevant enums/structs in `src/modules/{feature}/model/`

**Utilities:**
- Shared helpers: `src/shared/utils/common.rs` (ensure cross-feature usefulness)
- Feature-specific helpers: Keep in feature's `model/` directory
- Configuration: `src/core/config/mod.rs`
- Logging: `src/logger.rs`

**Cross-Feature Types:**
- Shared data models: `src/shared/models/`
- Common UI patterns: `src/shared/utils/`

## Special Directories

**`src/backups/`:**
- Purpose: Version control for implementation backups
- Generated: Manual backups, not auto-generated
- Committed: Yes (version control)
- Contents: `main.bk.rs` - backup of previous main implementation

**`target/`:**
- Purpose: Build artifacts and compiled code
- Generated: Yes (cargo build)
- Committed: No (.gitignore)
- Contents: Debug/release builds, dependency builds, intermediate objects

**`.planning/codebase/`:**
- Purpose: Architecture documentation (created by GSD tools)
- Generated: Yes (documentation generation)
- Committed: Yes (reference material)
- Contents: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, etc.

**`docs/`:**
- Purpose: Additional project documentation
- Generated: Manual documentation
- Committed: Yes
- Contents: Design documents, feature specifications

## Module Organization Summary

The codebase has two coexisting architectural approaches:

**MVC Module-Based (Primary - `src/modules/`):**
- Clean separation of concerns per feature
- Recommended for new code
- Each module self-contained with model, view, controller

**Legacy Handler-Based (`src/handlers/`, `src/ui/`, `src/main.rs`):**
- Centralized event handlers
- Centralized UI rendering
- Monolithic design
- Gradually being replaced

**Shared Infrastructure:**
- `src/shared/`: Cross-feature utilities and types
- `src/core/`: Configuration, logging, backup
- `AppState`: Central state container used by all features
