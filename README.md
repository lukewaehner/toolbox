# Toolbox - A Rust-based System Utility Suite

Toolbox is a comprehensive terminal-based utility application built in Rust that provides a collection of tools for system administration, network diagnostics, password management, and task scheduling.

## Features

### Password Manager

- Securely store and manage passwords with AES-256 encryption
- Add, view, and manage login credentials for various services
- Automatic encryption/decryption of sensitive data

### Network Tools

- Ping utility for network diagnostics
- Speed test tool to measure download speeds
- Multiple speed test providers for reliability

### System Utilities

- Real-time system resource monitoring (CPU, memory, disk)
- Process management with detailed information
- Disk space analyzer

### Task Scheduler

- Create and manage tasks with priorities, due dates, and status tracking
- Set reminders via desktop notifications and email
- Tag and filter tasks for easy organization

## Technical Details

### Architecture

The application uses a Terminal User Interface (TUI) built with the `tui` and `crossterm` crates, providing a responsive and interactive console experience. The application is structured around an event-driven model with different operational modes.

### Security

- Passwords are encrypted with AES-256 in CBC mode
- Encryption keys are stored in environment variables
- Password data is serialized to JSON before encryption

### Dependencies

- `aes`, `cbc`, `cipher`: For encryption/decryption operations
- `tui`, `crossterm`: Terminal UI framework
- `reqwest`: For network operations
- `serde`, `serde_json`: Data serialization
- `chrono`: Date and time handling
- `lettre`: Email functionality
- `notify-rust`: Desktop notifications
- `sysinfo`: System information

## Installation

### Prerequisites

- Rust and Cargo installed

### Building from source

```bash
# Clone the repository
git clone https://github.com/lukewaehner/toolbox.git
cd toolbox

# Build the application
cargo build --release

# Run the application
cargo run --release
```

### Configuration

1. Create a `.env` file in the project root with:
   ```
   ENCRYPTION_KEY={secret_key_here}
   ```
- Note: This must be 32 characters long
2. Email configuration can be set up within the application

## Usage

### Navigation

- Use arrow keys to navigate menus
- Press Enter to select an option
- Press Esc to go back or exit a screen
- Follow on-screen instructions for specific tools

### Password Management

1. Select "Password Manager" from the main menu
2. Add new credentials with service name, username, and password
3. View and manage existing credentials

### Network Tools

1. Select "Network Tools" from the main menu
2. Choose ping or speed test utilities
3. Follow prompts to enter domains or IP addresses

### System Monitoring

1. Select "System Utilities" from the main menu
2. Navigate between CPU, memory, disk, and process monitoring views

### Task Scheduling

1. Select "Task Scheduler" from the main menu
2. Add new tasks with descriptions, due dates, and priorities
3. Set up email reminders for important tasks

## License

[MIT License](LICENSE)

## Acknowledgements

- This project uses various Rust crates (see Cargo.toml for details)
