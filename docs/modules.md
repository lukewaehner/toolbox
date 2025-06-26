# Toolbox Module Documentation

This document provides detailed information about each module in the Toolbox application, explaining their purpose and functionality.

## Core Modules

### main.rs

The entry point of the application that initializes the terminal UI, sets up the application state, and handles the main event loop. It also manages terminal cleanup to ensure proper exit.

### app.rs

Contains the core application logic, including:

- `run_app`: The main application loop that handles user input and updates the UI
- Event handling for different application modes
- Status message management
- Function to perform cleanup when exiting the application

### models/app_state.rs

Defines the application state and data structures including:

- `AppState`: Central state structure that tracks the current menu, input mode, and all data fields
- `MenuItem`: Enumeration of the main menu items (Main, PasswordManager, NetworkTools, etc.)
- `InputMode`: Different input modes for the application (Normal, Editing, Viewing, etc.)
- `StatusMessage`: Structure for displaying temporary status messages to the user
- Default implementation for the initial application state

## Feature Modules

### password_manager.rs

Handles secure password storage and management:

- AES-256 encryption in CBC mode for password data
- Functions to save, load, and retrieve password entries
- Secure encryption and decryption using environment variables for keys
- JSON serialization for password storage

### network_tools.rs

Provides networking utilities:

- Ping functionality to test network connectivity
- Speed testing to measure download speeds
- Multiple test servers for reliable speed measurements
- Parallel download testing for more accurate results

### system_utilities.rs

System monitoring and management functionality:

- Real-time CPU, memory, and disk usage monitoring
- Process list with detailed information
- Process management (viewing, sorting, terminating)
- Disk space analysis

### task_scheduler.rs

Task and reminder management system:

- Task creation with priorities, due dates, and tags
- Reminder system with email and desktop notifications
- Background thread to check for due tasks and reminders
- Email configuration for sending notifications

## UI Modules

### ui/

Contains the Terminal User Interface (TUI) components:

- `main_menu.rs`: Renders the main application menu
- `password_manager.rs`: UI for the password management screens
- `network_tools.rs`: UI for network diagnostic tools
- `system_utilities.rs`: UI for system monitoring and management
- `task_scheduler.rs`: UI for the task scheduling functionality

## Handler Modules

### handlers/

Contains input handler functions for different application modes:

- `normal_mode.rs`: Handles the normal navigation mode
- `password_manager.rs`: Handles input for password management screens
- `network_tools.rs`: Handles input for network tool screens
- `system_utilities.rs`: Handles input for system monitoring screens
- `task_scheduler.rs`: Handles input for task scheduling screens

## Utility Modules

### models/

Contains data structures used throughout the application:

- `app_state.rs`: Application state structure
- `ping_result.rs`: Structure for ping command results

## Design Patterns

The application follows several design patterns:

1. **State Pattern**: Different application states control the behavior and UI
2. **Event-driven Architecture**: The app responds to user input events
3. **Model-View-Controller**: Separation of data (models), presentation (ui), and logic (handlers)
4. **Thread Pool**: Background workers for tasks that shouldn't block the main UI thread
