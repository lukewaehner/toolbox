# Technology Stack

**Analysis Date:** 2026-02-22

## Languages

**Primary:**
- Rust 2021 edition - All application code, system utilities, and network tools

**Secondary:**
- JSON - Configuration files and data serialization

## Runtime

**Environment:**
- Rust binary (compiled executable)

**Package Manager:**
- Cargo 1.x
- Lockfile: `Cargo.lock` present (72.5 KB)

## Frameworks

**Core UI:**
- `tui` 0.19.0 - Terminal User Interface (TUI) framework for interactive console
- `crossterm` 0.28.1 - Terminal manipulation and event handling (mouse capture, raw mode, key events)

**Background Tasks:**
- `signal-hook` 0.3 - Unix signal handling for graceful shutdown (SIGINT)
- `chrono` 0.4 - Date/time handling and timestamp management

**Serialization:**
- `serde` 1.0 (with derive feature) - Data serialization framework
- `serde_json` 1.0 - JSON encoding/decoding

## Key Dependencies

**Critical:**
- `aes` 0.8 - AES-256 encryption for password storage
- `cbc` 0.1 - CBC mode cipher for encryption operations
- `cipher` 0.4 - Cryptographic primitives and block padding (PKCS7)
- `rand` 0.8 - Cryptographic random number generation for IV generation
- `base64` 0.22.1 - Base64 encoding/decoding for encrypted data serialization

**Networking:**
- `reqwest` 0.11 (with blocking and json features) - HTTP client for speed tests and network operations
- Version in lock: 0.11.27

**System Information:**
- `sysinfo` 0.29.10 (lock file: 0.29.11) - System resource monitoring (CPU, memory, disk, processes)

**Email & Notifications:**
- `lettre` 0.10 (with builder, smtp-transport, tokio1, tokio1-native-tls features) - SMTP email delivery (version in lock: 0.10.4)
- `notify-rust` 4.8 - Desktop system notifications

**Utilities:**
- `regex` 1.7 - Regular expression parsing (for network tool output)
- `dotenv` 0.15 - Environment variable loading from `.env` files
- `once_cell` 1.20.3 - Lazy static initialization
- `humansize` 2.1.3 - Human-readable file size formatting

## Configuration

**Environment:**
- Loaded via `dotenv` from `.env` file in project root
- Critical variable: `ENCRYPTION_KEY` (32-character string for AES-256, 256 bits)
- Configuration structure defined in `src/config.rs`

**Build:**
- Compilation: `cargo build` (debug) or `cargo build --release` (optimized)
- Output locations:
  - Debug: `./target/debug/toolbox`
  - Release: `./target/release/toolbox`

**Data Storage Configuration:**
- Email configuration: `email_config.json` (SMTP settings stored in root)
- Password database: `passwords.json` (AES-256 encrypted)
- Task storage: `tasks.json` (JSON format with priority, due dates, reminders)

## Platform Requirements

**Development:**
- Rust toolchain (1.70+)
- Cargo
- Unix-like system for terminal features and system calls (`ping` command)
- Native TLS support (tokio1-native-tls for SMTP over TLS)

**Production:**
- macOS/Linux (uses Unix-specific features: signal handling, ping command)
- Supported SMTP servers: Gmail (smtp.gmail.com:587), Outlook (smtp-mail.outlook.com:587), Yahoo (smtp.mail.yahoo.com:587)
- Terminal with support for raw mode and mouse capture

---

*Stack analysis: 2026-02-22*
