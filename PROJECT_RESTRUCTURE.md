# Project Structure Refactoring - MVC Architecture

## Current Problems

- All UI code is mixed into a single `ui/` folder
- Controllers (handlers) are separate from their related modules
- Models are scattered across root-level files
- Hard to navigate and maintain as the project grows
- No clear separation of concerns

## Proposed New Structure

```
src/
├── main.rs                    # Application entry point
├── app.rs                     # Main application orchestration
│
├── core/                      # Core system functionality
│   ├── mod.rs
│   ├── config/               # Configuration management
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   └── types.rs
│   ├── logging/              # Logging system
│   │   ├── mod.rs
│   │   ├── logger.rs
│   │   └── formatters.rs
│   └── backup/               # Backup and recovery
│       ├── mod.rs
│       ├── manager.rs
│       └── metadata.rs
│
├── modules/                   # Feature modules (MVC pattern)
│   ├── mod.rs
│   │
│   ├── task_scheduler/        # Task scheduling module
│   │   ├── mod.rs
│   │   ├── model/            # Data structures & business logic
│   │   │   ├── mod.rs
│   │   │   ├── task.rs       # Task entity
│   │   │   ├── reminder.rs   # Reminder logic
│   │   │   ├── scheduler.rs  # Main scheduler
│   │   │   └── email.rs      # Email functionality
│   │   ├── view/            # UI components
│   │   │   ├── mod.rs
│   │   │   ├── menu.rs      # Main menu
│   │   │   ├── task_list.rs # Task listing view
│   │   │   ├── add_task.rs  # Add task form
│   │   │   └── config.rs    # Configuration views
│   │   └── controller/      # Event handling & flow control
│   │       ├── mod.rs
│   │       ├── task_handler.rs
│   │       ├── config_handler.rs
│   │       └── reminder_handler.rs
│   │
│   ├── password_manager/      # Password management module
│   │   ├── mod.rs
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── entry.rs      # Password entry
│   │   │   ├── storage.rs    # Storage logic
│   │   │   └── encryption.rs # Encryption handling
│   │   ├── view/
│   │   │   ├── mod.rs
│   │   │   ├── menu.rs
│   │   │   ├── list.rs
│   │   │   └── edit_form.rs
│   │   └── controller/
│   │       ├── mod.rs
│   │       ├── entry_handler.rs
│   │       └── auth_handler.rs
│   │
│   ├── network_tools/         # Network utilities module
│   │   ├── mod.rs
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── ping.rs
│   │   │   ├── speed_test.rs
│   │   │   └── results.rs
│   │   ├── view/
│   │   │   ├── mod.rs
│   │   │   ├── menu.rs
│   │   │   ├── input_form.rs
│   │   │   └── results_display.rs
│   │   └── controller/
│   │       ├── mod.rs
│   │       ├── ping_handler.rs
│   │       └── speed_test_handler.rs
│   │
│   └── system_utilities/      # System monitoring module
│       ├── mod.rs
│       ├── model/
│       │   ├── mod.rs
│       │   ├── monitor.rs
│       │   ├── process.rs
│       │   └── snapshot.rs
│       ├── view/
│       │   ├── mod.rs
│       │   ├── overview.rs
│       │   ├── process_list.rs
│       │   └── resource_monitor.rs
│       └── controller/
│           ├── mod.rs
│           ├── system_handler.rs
│           └── process_handler.rs
│
└── shared/                    # Shared components
    ├── mod.rs
    ├── models/               # Common data structures
    │   ├── mod.rs
    │   ├── app_state.rs     # Global app state
    │   ├── events.rs        # Event types
    │   └── common.rs        # Common types
    ├── utils/               # Utility functions
    │   ├── mod.rs
    │   ├── ui_common.rs     # Common UI helpers
    │   ├── validation.rs    # Input validation
    │   └── formatters.rs    # Display formatters
    └── traits/              # Shared traits
        ├── mod.rs
        ├── controller.rs    # Controller trait
        ├── view.rs         # View trait
        └── model.rs        # Model trait
```

## Benefits of This Structure

### 1. **Clear Separation of Concerns**

- **Model**: Data structures, business logic, and data persistence
- **View**: UI components and rendering logic
- **Controller**: Event handling, user input processing, and flow control

### 2. **Better Organization**

- Related functionality is grouped together
- Easy to find and modify specific features
- Clear dependencies between components

### 3. **Scalability**

- Easy to add new modules following the same pattern
- Individual modules can be developed independently
- Minimal coupling between different features

### 4. **Maintainability**

- Changes to one component don't affect others
- Easier to test individual parts
- Clear responsibility boundaries

### 5. **Code Reusability**

- Shared utilities and components
- Common traits for consistent interfaces
- Reusable UI components

## Example Module Structure - Task Scheduler

### Model Layer (`modules/task_scheduler/model/`)

```rust
// task.rs - Core task entity
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub reminders: Vec<Reminder>,
}

// scheduler.rs - Main business logic
pub struct TaskScheduler {
    tasks: HashMap<u32, Task>,
    config: SchedulerConfig,
}

impl TaskScheduler {
    pub fn add_task(&mut self, task: Task) -> Result<u32, SchedulerError>;
    pub fn get_due_tasks(&self) -> Vec<&Task>;
    pub fn send_reminders(&self) -> Result<(), SchedulerError>;
}
```

### View Layer (`modules/task_scheduler/view/`)

```rust
// menu.rs - Main task scheduler menu
pub fn draw_task_menu<B: Backend>(f: &mut Frame<B>, state: &TaskState);

// task_list.rs - Task listing view
pub fn draw_task_list<B: Backend>(f: &mut Frame<B>, tasks: &[Task]);

// add_task.rs - Task creation form
pub fn draw_add_task_form<B: Backend>(f: &mut Frame<B>, form_state: &TaskFormState);
```

### Controller Layer (`modules/task_scheduler/controller/`)

```rust
// task_handler.rs - Task-related event handling
pub fn handle_task_input(
    event: KeyCode,
    state: &mut AppState
) -> Result<(), ControllerError>;

pub fn handle_add_task(
    form_data: TaskFormData,
    scheduler: &mut TaskScheduler
) -> Result<u32, ControllerError>;
```

## Migration Strategy

### Phase 1: Setup New Structure

1. Create the new directory structure
2. Move core functionality (config, logging, backup)
3. Update main.rs to use new modules

### Phase 2: Migrate One Module

1. Start with task_scheduler as a proof of concept
2. Split existing code into MVC components
3. Update imports and fix compilation

### Phase 3: Migrate Remaining Modules

1. Password manager
2. Network tools
3. System utilities

### Phase 4: Cleanup

1. Remove old structure
2. Update documentation
3. Add integration tests

## Code Quality Improvements

### 1. **Shared Traits**

```rust
// shared/traits/controller.rs
pub trait Controller {
    type Event;
    type State;
    type Error;

    fn handle_event(&self, event: Self::Event, state: &mut Self::State)
        -> Result<(), Self::Error>;
}

// shared/traits/view.rs
pub trait View<B: Backend> {
    type State;

    fn render(&self, frame: &mut Frame<B>, state: &Self::State);
}
```

### 2. **Error Handling**

```rust
// Each module has its own error types
#[derive(Debug, thiserror::Error)]
pub enum TaskSchedulerError {
    #[error("Task not found: {id}")]
    TaskNotFound { id: u32 },

    #[error("Email configuration error: {0}")]
    EmailConfig(String),

    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),
}
```

### 3. **Configuration Management**

```rust
// Each module can have its own config section
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub task_scheduler: TaskSchedulerConfig,
    pub password_manager: PasswordManagerConfig,
    pub network_tools: NetworkToolsConfig,
}
```

## Development Workflow Benefits

1. **Feature Development**: Work on one module without affecting others
2. **Testing**: Unit test models, integration test controllers
3. **Code Review**: Smaller, focused pull requests
4. **Team Collaboration**: Multiple developers can work on different modules
5. **Documentation**: Each module can have its own README and examples

This restructuring transforms the codebase from a monolithic structure into a well-organized, maintainable, and scalable application following modern software engineering practices.
