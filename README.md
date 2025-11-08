# Toolbox - A Rust-based System Utility Suite

Toolbox is a comprehensive terminal-based utility application built in Rust that provides a collection of tools for system administration, network diagnostics, password management, and task scheduling.

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## 📋 Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

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

The application uses a Terminal User Interface (TUI) built with the `tui` and `crossterm` crates, providing a responsive and interactive console experience. The application follows a **Model-View-Controller (MVC)** architecture with clear separation between:

- **Models**: Data structures and business logic
- **Views**: UI rendering and presentation
- **Controllers**: Event handling and user input processing

For detailed architecture information, see [Architecture Documentation](docs/ARCHITECTURE.md).

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

For complete API documentation, see [API Documentation](docs/API.md).

## Quick Start

Get up and running in 3 steps:

```bash
# 1. Clone the repository
git clone https://github.com/lukewaehner/toolbox.git
cd toolbox

# 2. Set up environment variables
echo "ENCRYPTION_KEY=your_32_character_secret_key_here" > .env

# 3. Build and run
cargo run --release
```

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

# The binary will be available at
./target/release/toolbox
```

For development builds (faster compilation, includes debug symbols):

```bash
cargo build
./target/debug/toolbox
```

## Configuration

## Configuration

### Environment Variables

1. Create a `.env` file in the project root:

```bash
ENCRYPTION_KEY=your_32_character_encryption_key
```

⚠️ **Important**: The encryption key must be exactly 32 characters (256 bits) long. This key is used to encrypt/decrypt password entries.

Example:
```bash
ENCRYPTION_KEY=my_super_secret_key_1234567890
```

### Email Configuration (Optional)

Email configuration for task reminders can be set up through the application UI:

1. Navigate to Task Scheduler
2. Press 'E' to configure email
3. Enter your SMTP details:
   - SMTP Server (e.g., `smtp.gmail.com`)
   - SMTP Port (usually `587` for TLS or `465` for SSL)
   - Your email address
   - Your email password or app-specific password

**Common SMTP Providers**:
- Gmail: `smtp.gmail.com:587` (use app password)
- Outlook: `smtp-mail.outlook.com:587`
- Yahoo: `smtp.mail.yahoo.com:587`

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

**Keyboard Shortcuts**:
- `a` - Add new task
- `r` - Add reminder to selected task
- `e` - Configure email settings
- `d` - Delete selected task
- `Enter` - View task details
- `Esc` - Go back

## Troubleshooting

### Common Issues

**"Invalid key length" error**
- Ensure your `ENCRYPTION_KEY` is exactly 32 characters long

**Ping not working**
- The ping command requires the system `ping` utility
- On some systems, you may need to run with elevated privileges

**Email reminders not sending**
- Verify SMTP configuration is correct
- For Gmail, use an app-specific password
- Check firewall settings for outbound SMTP connections

**Application crashes on startup**
- Ensure all dependencies are installed: `cargo check`
- Check that `.env` file exists and is properly formatted
- Verify terminal supports required features

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[API Documentation](docs/API.md)**: Detailed API reference for all modules
- **[Architecture](docs/ARCHITECTURE.md)**: System design and architecture overview
- **[Module Documentation](docs/modules.md)**: In-depth module explanations
- **[Contributing Guide](docs/CONTRIBUTING.md)**: How to contribute to the project
- **[Improvements](IMPROVEMENTS.md)**: List of features and enhancements
- **[Project Structure](PROJECT_RESTRUCTURE.md)**: Information about MVC restructuring

You can also generate Rust documentation:

```bash
cargo doc --no-deps --open
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](docs/CONTRIBUTING.md) for details on:

- Setting up your development environment
- Code style and conventions
- Testing requirements
- Submitting pull requests

## License

[MIT License](LICENSE)

## Acknowledgements

This project uses various excellent Rust crates:
- **TUI Framework**: `tui-rs` and `crossterm` for the terminal interface
- **Encryption**: `aes` and `cbc` for secure password storage
- **System Info**: `sysinfo` for system monitoring
- **Email**: `lettre` for email notifications
- **Notifications**: `notify-rust` for desktop notifications

See [Cargo.toml](Cargo.toml) for the complete list of dependencies.

## Project Status

Active development. See the [issues](https://github.com/lukewaehner/toolbox/issues) page for upcoming features and known bugs.

