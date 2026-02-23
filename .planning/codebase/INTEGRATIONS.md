# External Integrations

**Analysis Date:** 2026-02-22

## APIs & External Services

**Network Diagnostics:**
- Speed test servers (multiple CDN-based URLs for redundancy):
  - `http://speedtest-ny.turnkeyinternet.net/100mb.bin` (100 MB file)
  - `http://speedtest.tele2.net/50MB.zip` (50 MB file)
  - `http://speedtest.belwue.net/random-50M` (50 MB file)
  - `http://ipv4.download.thinkbroadband.com/50MB.zip` (50 MB file)
  - Fallback: 10 MB variants of above URLs
  - SDK/Client: `reqwest` 0.11 (blocking HTTP client)
  - Purpose: Download speed measurement for network diagnostics

**System Ping:**
- Native system `ping` command invocation via subprocess
- Parses output using regex patterns
- Supports ICMP echo request/reply testing

## Data Storage

**Databases:**
- Not applicable - No traditional database

**File Storage:**
- Local filesystem (project root and subdirectories):
  - `passwords.json` - Encrypted password entries (AES-256-CBC)
  - `tasks.json` - Task scheduler data (JSON)
  - `email_config.json` - SMTP configuration (JSON)
  - Logs directory: `logs/toolbox.log` (if logging enabled)
- Client: None (direct file I/O via `std::fs`)
- Storage format: JSON with encryption applied to sensitive data

**Caching:**
- In-memory only via `once_cell` for lazy static initialization
- System snapshots maintained in `SystemHistory` structure (configurable point limit)

## Authentication & Identity

**Auth Provider:**
- Custom SMTP authentication
  - Implementation: Username/password credentials sent to SMTP server
  - Storage: `EmailConfig` struct in `email_config.json`
  - Fields required:
    - `email`: Sender email address
    - `smtp_server`: SMTP hostname
    - `smtp_port`: Port number (587 for TLS, 465 for SSL)
    - `username`: SMTP authentication username
    - `password`: SMTP authentication password

**Encryption:**
- AES-256 in CBC mode for password storage
- Random IV (initialization vector) generated per encryption operation
- Key source: `ENCRYPTION_KEY` environment variable (32 characters / 256 bits)
- Implementation: `aes` + `cbc` + `cipher` crates with PKCS7 padding

## Monitoring & Observability

**Error Tracking:**
- None detected - Error handling via Result types and error messages

**Logs:**
- Console output via `println!` macros
- Optional file logging configuration in `src/config.rs`:
  - Log level: configurable (debug, info, warning, error)
  - File path: `logs/toolbox.log` (default)
  - Max file size: 10 MB (configurable)
  - Max files: 5 rotating files (configurable)

**System Monitoring:**
- `sysinfo` crate for CPU, memory, disk, and process metrics
- No external monitoring service integration

## CI/CD & Deployment

**Hosting:**
- Self-contained binary deployment
- No cloud platform integration detected
- Can be compiled to single executable via `cargo build --release`

**CI Pipeline:**
- None detected - No GitHub Actions, GitLab CI, or other CI configuration found

## Environment Configuration

**Required env vars:**
- `ENCRYPTION_KEY` - Must be exactly 32 characters (256 bits) for AES-256

**Optional env vars:**
- None detected - All other configuration via JSON files

**Secrets location:**
- Environment variables: `.env` file (loaded via `dotenv`)
- Note: `.env` file exists but contents are not included in version control (typical practice)
- SMTP credentials stored in application via UI configuration in `email_config.json`

## Webhooks & Callbacks

**Incoming:**
- None detected - No webhook endpoints

**Outgoing:**
- Email reminders via SMTP
  - Transport: SMTP over TLS/SSL via `lettre` crate
  - Trigger: When task reminder time is reached (checked every 30 seconds by default)
  - Format: HTML email with task details
  - Retry: Up to 3 attempts with 5-minute delay between retries (configurable)
  - Timeout: 30 seconds per send operation (configurable)

- Desktop notifications
  - Transport: Native system notifications via `notify-rust`
  - Trigger: When task reminder time is reached
  - Platforms: Linux/macOS (uses platform-native notification daemon)

## Network Configuration

**Timeout Settings:**
- Network operations: 30 seconds timeout (configurable in `AppConfig`)
- SMTP email operations: 30 seconds timeout (configurable in `ReminderConfig`)
- Speed test duration: 10 seconds (configurable)

**Retry Logic:**
- Network operations: Up to 3 retries (configurable)
- Email reminders: Up to 3 retry attempts with 5-minute delay between retries

**User Agent:**
- Default: "Toolbox/1.0" (configurable in `NetworkConfig`)

---

*Integration audit: 2026-02-22*
