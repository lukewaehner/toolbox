# Contributing to Toolbox

Thank you for your interest in contributing to Toolbox! This guide will help you get started with development, testing, and submitting contributions.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Architecture](#architecture)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Adding Features](#adding-features)

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (latest stable version): Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **A Unix-like system**: Linux or macOS (required for some features like ping)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/toolbox.git
cd toolbox
```

3. Add the upstream repository:

```bash
git remote add upstream https://github.com/lukewaehner/toolbox.git
```

## Development Setup

### Environment Configuration

1. Create a `.env` file in the project root:

```bash
cp .env.example .env  # If example exists
# OR create manually:
echo "ENCRYPTION_KEY=your_32_character_key_here_1234" > .env
```

**Important**: The `ENCRYPTION_KEY` must be exactly 32 characters long.

### Building the Project

```bash
# Debug build (faster compilation, includes debug symbols)
cargo build

# Release build (optimized, slower compilation)
cargo build --release
```

### Running the Application

```bash
# Run in development mode
cargo run

# Run with release optimizations
cargo run --release
```

### Checking Your Code

Before submitting changes, always run these checks:

```bash
# Check for compilation errors
cargo check

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run Clippy (linter)
cargo clippy -- -D warnings

# Generate documentation
cargo doc --no-deps --open
```

## Code Style

### Rust Formatting

We follow the standard Rust formatting guidelines:

```bash
# Format all code
cargo fmt
```

### Naming Conventions

- **Functions**: `snake_case`
- **Types/Structs/Enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Documentation Comments

All public items must have documentation comments:

```rust
/// Saves a password entry to encrypted storage
///
/// # Arguments
///
/// * `entry` - The password entry to save
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if saving fails.
///
/// # Examples
///
/// ```
/// let entry = PasswordEntry {
///     service: "example.com".to_string(),
///     username: "user".to_string(),
///     password: "pass".to_string(),
/// };
/// save_password(&entry)?;
/// ```
pub fn save_password(entry: &PasswordEntry) -> io::Result<()> {
    // Implementation
}
```

### Code Organization

#### Module Structure

Each feature module should follow the MVC pattern:

```
modules/
└── feature_name/
    ├── mod.rs              # Public API
    ├── model/
    │   ├── mod.rs
    │   └── data.rs         # Data structures and business logic
    ├── view/
    │   ├── mod.rs
    │   └── ui.rs           # UI rendering
    └── controller/
        ├── mod.rs
        └── handlers.rs     # Event handling
```

#### Import Organization

Organize imports in this order:

1. Standard library
2. External crates
3. Internal modules

```rust
// Standard library
use std::io;
use std::fs;

// External crates
use serde::{Deserialize, Serialize};
use crossterm::event::KeyCode;

// Internal modules
use crate::password_manager::PasswordEntry;
use crate::app_state::AppState;
```

## Architecture

### Understanding the Codebase

Read the [Architecture Documentation](ARCHITECTURE.md) for a comprehensive overview.

### Key Concepts

1. **AppState**: Central state structure containing all application data
2. **InputMode**: Determines how user input is handled
3. **MenuItem**: Represents the active feature/screen
4. **Event Loop**: Main loop that polls for input and updates UI

### Adding Documentation

#### Inline Comments

Use inline comments sparingly, only for complex logic:

```rust
// Calculate packet loss percentage (received/transmitted)
let loss = (transmitted - received) as f32 / transmitted as f32 * 100.0;
```

#### Module Documentation

Every module should have a module-level doc comment:

```rust
//! Password Manager Module
//!
//! Provides secure password storage using AES-256 encryption.
//! Passwords are encrypted before being written to disk.
```

#### Function Documentation

Document all public functions with:
- Brief description
- Arguments explanation
- Return value description
- Examples (when helpful)
- Error conditions

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test password_manager
```

### Writing Tests

#### Unit Tests

Place unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_encryption() {
        let data = b"test password";
        let encrypted = encrypt(data).expect("Encryption failed");
        let decrypted = decrypt(&encrypted).expect("Decryption failed");
        assert_eq!(data, decrypted.as_slice());
    }
}
```

#### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/password_manager_tests.rs
use toolbox::password_manager::{save_password, retrieve_password, PasswordEntry};

#[test]
fn test_save_and_retrieve() {
    let entry = PasswordEntry {
        service: "test".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    
    save_password(&entry).expect("Save failed");
    let entries = retrieve_password().expect("Retrieve failed");
    assert!(entries.iter().any(|e| e.service == "test"));
}
```

### Test Coverage

Aim for high test coverage, especially for:
- Business logic
- Data serialization/deserialization
- Error handling paths
- Edge cases

## Submitting Changes

### Before You Submit

1. **Update tests**: Add tests for new features
2. **Run all checks**: `cargo check`, `cargo test`, `cargo clippy`
3. **Format code**: `cargo fmt`
4. **Update documentation**: Add/update doc comments and README if needed
5. **Test manually**: Run the application and test your changes

### Commit Messages

Write clear, descriptive commit messages:

```
Add email retry logic for failed reminders

- Implement exponential backoff for email retries
- Add retry count tracking in Reminder struct
- Update UI to show retry status
- Add tests for retry logic
```

**Format**:
- First line: Brief summary (50 chars or less)
- Blank line
- Detailed description with bullet points

### Pull Request Process

1. **Create a branch**:

```bash
git checkout -b feature/my-new-feature
```

2. **Make your changes** and commit them:

```bash
git add .
git commit -m "Add my new feature"
```

3. **Push to your fork**:

```bash
git push origin feature/my-new-feature
```

4. **Create a Pull Request** on GitHub:
   - Go to your fork on GitHub
   - Click "New Pull Request"
   - Select your branch
   - Fill in the PR template with:
     - Description of changes
     - Related issue numbers
     - Testing performed
     - Screenshots (if UI changes)

5. **Address review feedback**:
   - Make requested changes
   - Push updates to the same branch
   - Respond to comments

### Code Review

Pull requests will be reviewed for:
- Code quality and style
- Test coverage
- Documentation completeness
- Security considerations
- Performance impact
- Compatibility

## Adding Features

### Adding a New Tool/Feature

1. **Plan the feature**:
   - Define requirements
   - Design data structures
   - Plan UI layout
   - Consider error cases

2. **Create module structure**:

```bash
mkdir -p src/modules/new_feature/{model,view,controller}
touch src/modules/new_feature/{mod.rs,model/mod.rs,view/mod.rs,controller/mod.rs}
```

3. **Implement the model** (data structures and logic):

```rust
// src/modules/new_feature/model/mod.rs
pub struct FeatureData {
    pub field1: String,
    pub field2: i32,
}

impl FeatureData {
    pub fn new() -> Self {
        Self {
            field1: String::new(),
            field2: 0,
        }
    }
}
```

4. **Implement the view** (UI rendering):

```rust
// src/modules/new_feature/view/mod.rs
use tui::{Frame, backend::Backend, widgets::Paragraph};

pub fn draw_feature<B: Backend>(f: &mut Frame<B>, data: &FeatureData) {
    let paragraph = Paragraph::new(format!("Value: {}", data.field1));
    f.render_widget(paragraph, f.size());
}
```

5. **Implement the controller** (input handling):

```rust
// src/modules/new_feature/controller/mod.rs
use crossterm::event::KeyCode;

pub fn handle_input(key: KeyCode, data: &mut FeatureData) {
    match key {
        KeyCode::Char(c) => data.field1.push(c),
        KeyCode::Backspace => { data.field1.pop(); },
        _ => {}
    }
}
```

6. **Add to main menu**:
   - Add variant to `MenuItem` enum
   - Add menu option in main menu drawing
   - Add case in event loop's draw function
   - Add input handler

7. **Add documentation**:
   - Module-level docs
   - Function docs
   - Update README if needed

8. **Add tests**:
   - Unit tests for logic
   - Integration tests for workflows

### Adding Dependencies

When adding a new dependency:

1. Add to `Cargo.toml`:

```toml
[dependencies]
new_crate = "1.0"
```

2. Document why it's needed in the PR
3. Check license compatibility
4. Keep dependencies minimal

### UI Guidelines

When adding UI components:

1. **Consistent styling**: Use existing color scheme
2. **Responsive**: Handle different terminal sizes
3. **Keyboard navigation**: Ensure all features are keyboard-accessible
4. **Clear feedback**: Show status messages for operations
5. **Error display**: Show user-friendly error messages

### Performance Guidelines

1. **Avoid blocking**: Use background threads for long operations
2. **Efficient updates**: Only refresh what's needed
3. **Memory conscious**: Drop large data when not needed
4. **Profile if needed**: Use `cargo flamegraph` for profiling

## Common Tasks

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Use rust-gdb or lldb for debugging
rust-gdb target/debug/toolbox
```

### Benchmarking

```bash
# Add benchmark to benches/ directory
cargo bench
```

## Getting Help

- **Issues**: Check existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Read the docs in the `docs/` directory

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the Rust Code of Conduct

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License).

## Recognition

Contributors will be acknowledged in the project README.

Thank you for contributing to Toolbox!
