# Toolbox

A terminal-based system utility suite written in Rust. Bundles a password manager, network diagnostics, system monitoring, and a task scheduler into a single TUI.

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Features

- **Password Manager** - AES-256-CBC encrypted credential storage
- **Network Tools** - ping and download speed test (multiple providers)
- **System Utilities** - live CPU/memory/disk monitoring, process management, disk usage
- **Task Scheduler** - tasks with priorities, due dates, tags, and reminders via desktop notifications or email

## Quick Start

```bash
git clone https://github.com/lukewaehner/toolbox.git
cd toolbox
echo "ENCRYPTION_KEY=your_32_character_secret_key_here" > .env
cargo run --release
```

`ENCRYPTION_KEY` must be exactly 32 characters (256 bits). Email/SMTP for task reminders is configured from inside the app (Task Scheduler → `e`).

## Architecture

MVC layout (models / views / controllers) on top of [`ratatui`](https://github.com/ratatui-org/ratatui) (with its bundled `crossterm` backend). See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

Core dependencies: `ratatui`, `aes`/`cbc`/`cipher`, `reqwest`, `serde`, `chrono`, `lettre`, `notify-rust`, `sysinfo`. See [Cargo.toml](Cargo.toml) for the full list.

## Keyboard Shortcuts

Arrow keys navigate, `Enter` selects, `Esc` goes back. Inside Task Scheduler:

| Key     | Action                       |
| ------- | ---------------------------- |
| `a`     | Add new task                 |
| `r`     | Add reminder to selected task|
| `e`     | Configure email settings     |
| `d`     | Delete selected task         |
| `Enter` | View task details            |

## Troubleshooting

- **"Invalid key length"** - `ENCRYPTION_KEY` must be exactly 32 characters.
- **Ping fails** - relies on the system `ping` binary; may need elevated privileges on some platforms.
- **Email reminders not sending** - verify SMTP settings; for Gmail use an app-specific password.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API reference](docs/API.md)
- [Modules](docs/modules.md)
- [Contributing](docs/CONTRIBUTING.md)

Generate Rust docs locally with `cargo doc --no-deps --open`.

## License

[MIT](LICENSE)
