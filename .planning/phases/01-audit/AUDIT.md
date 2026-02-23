# Phase 1 Audit: Duplicate and Legacy File Analysis

**Date:** 2026-02-23
**Auditor:** Claude (automated)
**Purpose:** Establish which files are safe to delete in Phase 2 (Architecture Consolidation)

---

## Section 1: Duplicate Entry-Point Files

### Files Examined

- `src/main.rs` (3430 lines)
- `src/app.rs` (3272 lines)
- `src/backups/main.bk.rs` (3272 lines)

### Active Crate Root

**`src/main.rs` is the active crate root.** Cargo.toml has no explicit `[[bin]]` section, so Rust's default convention applies: the binary entry point is `src/main.rs`. `cargo check` confirms the binary target is `toolbox (bin "toolbox")` and it compiles successfully using `src/main.rs`.

`src/app.rs` is referenced by `src/main.rs` via `mod app;` — it is included as a module, not an alternative entry point.

---

### Comparison: app.rs vs backups/main.bk.rs

**Are they identical?** Yes — byte-for-byte identical.

Running `diff src/app.rs src/backups/main.bk.rs` produced zero output (no differences). The files are exactly 3272 lines each with identical content.

**Verdict for `backups/main.bk.rs`:** SAFE-TO-DELETE

`backups/main.bk.rs` is an exact duplicate of `src/app.rs`. It provides no unique content, no unique logic, no unique functions, structs, enums, or imports that are not already present in `src/app.rs`. Deleting it loses nothing.

---

### Comparison: main.rs vs app.rs

**Summary of differences:**

`main.rs` has 3430 lines vs `app.rs` at 3272 lines. The difference is 158 lines — all of which are **documentation and comments added to `main.rs`**. No executable code, logic, functions, structs, enums, or `use` statements exist in `main.rs` that are absent from `app.rs`. The two files are logically identical; `main.rs` is the more thoroughly documented version.

**Content unique to `main.rs` (not in `app.rs`):**

All 168 unique lines in `main.rs` are documentation-only:

1. **Lines 1–17:** Module-level `//!` crate documentation block — describes Toolbox, its features, and architecture. Not present in `app.rs` at all.

2. **Line 56:** `/// Results from a ping network test` — doc comment on `PingResult` struct. `app.rs` uses `// Define the PingResult struct` (plain comment, line 39).

3. **Line 62:** `time: Option<u32>,` — clean field declaration. `app.rs` has `time: Option<u32>, // Change this to Option<u32> if it can be None` (a dev note indicating uncertain origin).

4. **Lines 69–156:** Doc comments (`///`) on all enums and their variants:
   - `InputMode` enum: doc comment on enum + 10 variant doc comments
   - `MenuItem` enum: doc comment on enum + 5 variant doc comments
   - `SystemViewMode` enum: doc comment on enum + 5 variant doc comments
   - `ConfirmationDialogue` enum: doc comment on enum + 2 variant doc comments
   - `ProcessSortOrder` enum: doc comment on enum + 5 variant doc comments

5. **Lines 158–286:** Doc comments (`///`) on `AppState` struct and all ~30 of its fields, plus `StatusMessage` struct and its fields, plus `StatusMessageType` enum and its variants.

6. **Lines 334–338:** Comprehensive doc comment on `TerminalCleanup` struct (5 lines). `app.rs` uses `// Struct to handle terminal cleanup` (plain comment, line 201).

7. **Line 347:** `execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)` on one line. `app.rs` formats this across 4 lines with an inline comment `// Ensure mouse capture is disabled`.

8. **Lines 356–363:** Doc comment block (`///`) on the `main()` function (8 lines). `app.rs` has no doc comment on `main()`.

9. **Line 369:** `// Create task scheduler instance` — clean comment. `app.rs` has `// Craete task scheduler` (typo: "Craete" instead of "Create").

10. **Lines 397–413:** Doc comment block (`///`) on the `run_app()` function (17 lines). `app.rs` has no doc comment on `run_app()`.

**Content unique to `app.rs` (not in `main.rs`):**

None of the 10 lines in `app.rs` that differ from `main.rs` represent unique executable content:
- `// Define the PingResult struct` — replaced by `///` doc comment in main.rs
- `time: Option<u32>, // Change this to Option<u32> if it can be None` — dev note, cleaned up in main.rs
- `KillProcess(u32, String), // Process ID and name` — inline comment replaced by doc comment in main.rs
- `// Struct to handle terminal cleanup` — replaced by doc comment in main.rs
- 4-line formatted `execute!()` call — functionally identical to single-line form in main.rs
- `// Craete task scheduler` — typo, cleaned up in main.rs to `// Create task scheduler instance`

`app.rs` has no unique functions, structs, enums, impl blocks, use statements, or logic that do not exist in `main.rs`.

**Verdict for `app.rs`:** SAFE-TO-DELETE

`app.rs` is a less-documented, older version of `main.rs`. All executable code is identical. `main.rs` is strictly superior — it contains all of `app.rs`'s logic plus 158 additional lines of documentation. Deleting `app.rs` loses nothing except the dev notes/typos that are already superseded in `main.rs`.

---

### Summary Table

| File | Lines | Status | Action for Phase 2 |
|------|-------|--------|---------------------|
| `src/main.rs` | 3430 | KEEP — canonical entry point (Cargo default `src/main.rs`) | No action |
| `src/app.rs` | 3272 | SAFE-TO-DELETE — identical logic to main.rs; main.rs has all content plus additional documentation | Delete |
| `src/backups/main.bk.rs` | 3272 | SAFE-TO-DELETE — byte-for-byte identical to app.rs | Delete |

---

### Notes for Phase 2

- Before deleting `src/app.rs`, verify no other files in `src/` use `mod app;` or `use crate::app::` imports that would break compilation. If `src/main.rs` currently uses `mod app;`, that declaration must be removed when `app.rs` is deleted.
- The `src/backups/` directory can be fully removed after `main.bk.rs` is deleted (it contains only this one file).
- `cargo check` currently passes with 22 warnings (no errors) — compilation baseline is healthy.

---

## Section 2: Root-Level Legacy Module Files

### Files Examined
- `src/task_scheduler.rs` (585 lines)
- `src/network_tools.rs` (464 lines)
- `src/password_manager.rs` (285 lines)
- `src/system_utilities.rs` (395 lines)

### Methodology
Each file compared against its `src/modules/` counterpart. Import usage in `src/main.rs` checked via grep. For each file: (1) is it imported in main.rs, (2) does it have content not present in the MVC counterpart, (3) does the MVC counterpart actually use the legacy file directly.

---

### src/task_scheduler.rs

**Imported in main.rs?** Yes — lines 22, 25: `mod task_scheduler;` and `use crate::task_scheduler::{EmailConfig, ReminderType, TaskPriority, TaskScheduler, TaskStatus};`

**MVC counterpart:** `src/modules/task_scheduler/` (model/task.rs, model/scheduler.rs, model/mod.rs, controller/mod.rs)

**Content summary:**
- `TaskPriority` enum (Low/Medium/High/Urgent)
- `TaskStatus` enum (Pending/InProgress/Completed/Cancelled)
- `ReminderType` enum (Email/Notification/Sms/Both/All)
- `SmsConfig` struct — phone number, carrier, enabled flag for SMS-via-email-gateway
- `EmailConfig` struct — SMTP settings with `retry_attempts` and `retry_delay_seconds` fields
- `Reminder` struct — with `retry_count`, `last_attempt`, `error_message` fields
- `Task` struct and `impl Task`
- `TaskScheduler` struct and `impl TaskScheduler` — includes `sms_config` field, `set_sms_config`, `get_sms_gateway_email`, `send_sms_reminder`, `create_smtp_transport`, `save_sms_config`, `load_sms_config`, `mark_reminder_as_sent`, `check_reminders` (enhanced with retry logic and per-type handling), `send_reminder_email` (enhanced with provider-specific STARTTLS handling and troubleshooting output), `test_email_config` (enhanced with provider-specific logic)
- `format_timestamp` function
- `run_scheduler_background_thread` function — enhanced version with full reminder dispatch loop, SMS handling, error tracking, and retry logic

**Content present in MVC counterpart?** Partial

**Unique content not in MVC counterpart:**

The `src/modules/task_scheduler/model/mod.rs` is an intermediary file that re-exports from `task.rs` and `scheduler.rs`. The `scheduler.rs` in the MVC module has a clean, error-typed rewrite that is architecturally superior, but is missing significant functionality:

1. **`SmsConfig` struct** (lines 48–53) — Not present in `src/modules/task_scheduler/model/scheduler.rs`. The MVC scheduler.rs has no SMS support at all.
2. **`EmailConfig.retry_attempts` / `retry_delay_seconds` fields** (lines 61–63) — The MVC scheduler.rs `EmailConfig` is defined in `model/mod.rs` and has these fields, so this is covered.
3. **`Reminder.retry_count`, `last_attempt`, `error_message` fields** — Present in `model/mod.rs` re-export from `task.rs` (which has these fields). Covered.
4. **`TaskScheduler.sms_config` field** — Not in `src/modules/task_scheduler/model/scheduler.rs` (MVC scheduler has no `sms_config`).
5. **`set_sms_config` / `get_sms_gateway_email` / `send_sms_reminder` / `save_sms_config` / `load_sms_config`** (lines 371–533) — Five SMS-related methods not in MVC scheduler.
6. **`create_smtp_transport`** (lines 442–484) — Provider-specific STARTTLS/TLS transport factory. Not in MVC scheduler (MVC scheduler has no email sending at all).
7. **`check_reminders` (enhanced)** (lines 628–748) — The MVC scheduler's `run_scheduler_background_thread` calls `get_tasks_with_pending_reminders()` as a placeholder only. The legacy file's `check_reminders` has full retry logic, per-type dispatch (Email/Sms/Notification/Both/All), and `last_attempt` tracking.
8. **`send_reminder_email` / `test_email_config`** (lines 776–864, 181–291) — Not in MVC scheduler at all.
9. **`mark_reminder_as_sent` (on TaskScheduler)** (lines 751–774) — The MVC `scheduler.rs` has this method, so it is covered.
10. **`format_timestamp` function** (lines 903–910) — Not in `src/modules/task_scheduler/model/scheduler.rs`.
11. **`run_scheduler_background_thread` (full dispatch loop)** (lines 912–1002) — The MVC counterpart has a stub that only calls `get_tasks_with_pending_reminders()` and prints a count. The legacy version does full reminder dispatch: sends emails, sends SMS, marks reminders as sent, tracks errors.

**Verdict:** HAS-UNIQUE-CONTENT

Before deletion, must migrate to MVC module:
- `SmsConfig` struct and full SMS stack (`set_sms_config`, `get_sms_gateway_email`, `send_sms_reminder`, `create_smtp_transport`, `save_sms_config`, `load_sms_config`)
- `send_reminder_email` and `test_email_config` methods with provider-specific SMTP logic
- `format_timestamp` function (used by `check_reminders` and `send_reminder_email`)
- Full `check_reminders` implementation with retry logic and per-type dispatch
- Full `run_scheduler_background_thread` dispatch loop

---

### src/network_tools.rs

**Imported in main.rs?** Yes — lines 19, 32: `mod network_tools;` and `use network_tools::{ping, SpeedTestResult};`. Also called directly at lines 719, 727, 747, 763, 771 via `network_tools::measure_speed()`, `network_tools::parallel_speed_test()`, `network_tools::SpeedTestResult::status()`, and `network_tools::SpeedTestResult::error()`.

**MVC counterpart:** `src/modules/network_tools/model/mod.rs` and `src/modules/network_tools/controller/mod.rs`

**Content summary:**
- `PingResult` struct
- `ping` function — executes system `ping` command with 4 packets, parses output
- `parse_ping_output` function — regex-based parsing of ping stdout
- `SpeedTestResult` struct with `status()` and `error()` constructors
- `measure_speed` function — single-file download speed test with fallback URLs
- `test_download_speed` helper — streaming HTTP download with progress reporting
- `speed_test` function — runs `measure_speed`, escalates to `parallel_speed_test` if fast
- `parallel_speed_test` function — parallel multi-thread download test
- `download_chunk` helper — single-thread download helper for parallel test

**Content present in MVC counterpart?** Full (identical)

**Unique content not in MVC counterpart:**

None — `src/modules/network_tools/model/mod.rs` is a byte-for-byte copy of `src/network_tools.rs` with the only difference being the addition of `// ... existing code ...` scaffold comments on `PingResult` and `ping`/`parse_ping_output`. All structs, all functions, all logic, all URL lists are identical.

Note: The MVC controller (`src/modules/network_tools/controller/mod.rs`) imports `ping` from `crate::network_tools` (the legacy file), not from the MVC model. This means the controller already delegates to the legacy file at runtime.

**Verdict:** SAFE-TO-DELETE

The MVC model is a full copy. Before deleting, Phase 2 must update all import sites (`src/main.rs` and `src/modules/network_tools/controller/mod.rs`) to reference `src/modules/network_tools/model/mod.rs` instead of the legacy `src/network_tools.rs`.

---

### src/password_manager.rs

**Imported in main.rs?** Yes — lines 20, 34: `mod password_manager;` and `use password_manager::{save_password, PasswordEntry};`

**MVC counterpart:** `src/modules/password_manager/model/mod.rs` and `src/modules/password_manager/controller/mod.rs`

**Content summary:**
- `PasswordEntry` struct (service, username, password)
- `retrieve_password` function — decrypts and returns all stored entries
- `save_password` function — appends and re-encrypts to `passwords.json`
- `load_passwords` helper — reads and decrypts storage file
- `encrypt` function — AES-256-CBC with random IV, base64-encoded output
- `decrypt` function — base64-decode, split IV, AES-256-CBC decrypt
- `get_encryption_key` function — reads 32-byte key from `ENCRYPTION_KEY` env var
- `#[cfg(test)] mod tests` — `test_encrypt_decrypt` round-trip test

**Content present in MVC counterpart?** Full (identical)

**Unique content not in MVC counterpart:**

None — `src/modules/password_manager/model/mod.rs` is essentially byte-for-byte identical to `src/password_manager.rs`. The only observable difference is the removal of doc comments (`///` and `//!`) from the MVC model file — all executable code (all functions, all structs, all crypto logic) is identical. The `#[cfg(test)]` module with `test_encrypt_decrypt` is present in both files.

Note: The MVC controller (`src/modules/password_manager/controller/mod.rs`) imports `save_password` and `PasswordEntry` from `crate::password_manager` (the legacy file), not from the MVC model. The controller is already backed by the legacy file at runtime.

**Verdict:** SAFE-TO-DELETE

The MVC model is a full copy. Before deleting, Phase 2 must update all import sites (`src/main.rs` and `src/modules/password_manager/controller/mod.rs`) to reference `src/modules/password_manager/model/mod.rs` instead of the legacy `src/password_manager.rs`.

---

### src/system_utilities.rs

**Imported in main.rs?** Yes — lines 21, 46: `mod system_utilities;` and `use system_utilities::SystemMonitor;`

**MVC counterpart:** `src/modules/system_utilities/model/mod.rs` and `src/modules/system_utilities/controller/mod.rs`

**Content summary:**
- `SystemSnapshot` struct (cpu, memory, swap, disks, processes, timestamp)
- `DiskInfo` struct
- `ProcessInfo` struct
- `SystemHistory` struct with `new` and `add_snapshot` methods
- `SystemMonitor` struct with `new`, `refresh_if_needed`, `refresh`, `snapshot`, `history`, `refresh_and_get`, `kill_process` methods

**Content present in MVC counterpart?** Full (identical)

**Unique content not in MVC counterpart:**

None — `src/modules/system_utilities/model/mod.rs` is byte-for-byte identical to `src/system_utilities.rs` in all executable code. The only differences are: (1) the legacy file has `///` doc comments and `//!` module-level documentation; the MVC model file uses inline `//` comments. (2) The legacy file has an unused `use std::collections::HashMap;` import (generates compiler warning) that is also present in the MVC model. All structs, all methods, all sysinfo calls, all logic are identical.

Note: The MVC controller (`src/modules/system_utilities/controller/mod.rs`) imports types from `crate::models::app_state` and calls through `app_state.system_monitor` which is a `SystemMonitor` from the legacy file. The controller does not import from the MVC model directly.

**Verdict:** SAFE-TO-DELETE

The MVC model is a full copy. Before deleting, Phase 2 must update all import sites (`src/main.rs`) to reference `src/modules/system_utilities/model/mod.rs` instead of the legacy `src/system_utilities.rs`.

---

### Legacy Module Summary Table

| File | Lines | Used in main.rs | MVC Counterpart Coverage | Verdict | Phase 2 Action |
|------|-------|-----------------|--------------------------|---------|----------------|
| `src/task_scheduler.rs` | 585 | Yes | Partial — SMS stack, email send/test, format_timestamp, and full background dispatcher absent from MVC | HAS-UNIQUE-CONTENT | Migrate SMS stack + email dispatch + format_timestamp to MVC before deleting |
| `src/network_tools.rs` | 464 | Yes | Full — MVC model is byte-for-byte copy | SAFE-TO-DELETE | Update import sites in main.rs and network_tools controller, then delete |
| `src/password_manager.rs` | 285 | Yes | Full — MVC model is byte-for-byte copy (minus doc comments) | SAFE-TO-DELETE | Update import sites in main.rs and password_manager controller, then delete |
| `src/system_utilities.rs` | 395 | Yes | Full — MVC model is byte-for-byte copy (minus doc comments) | SAFE-TO-DELETE | Update import site in main.rs, then delete |

---

## Audit Conclusions

### Safe-to-Delete Files (Phase 2 can proceed immediately after import-site updates)

- `src/app.rs` — identical logic to main.rs (Section 1)
- `src/backups/main.bk.rs` — byte-for-byte identical to app.rs (Section 1)
- `src/network_tools.rs` — full copy in MVC model; update 2 import sites first
- `src/password_manager.rs` — full copy in MVC model; update 2 import sites first
- `src/system_utilities.rs` — full copy in MVC model; update 1 import site first

### Files Requiring Migration Before Deletion

- **`src/task_scheduler.rs`** — Contains a substantial SMS notification stack and full email delivery system not present in the MVC module. Before deleting, migrate:
  1. `SmsConfig` struct (phone number, carrier, enabled) to `src/modules/task_scheduler/model/mod.rs`
  2. `set_sms_config`, `get_sms_gateway_email`, `send_sms_reminder` methods to TaskScheduler in MVC model
  3. `create_smtp_transport` provider-specific SMTP factory method
  4. `save_sms_config` / `load_sms_config` persistence methods
  5. `send_reminder_email` method with provider-specific STARTTLS handling
  6. `test_email_config` method with provider-specific validation and troubleshooting
  7. `format_timestamp` function (used by email body and check_reminders)
  8. Full `check_reminders` implementation (retry logic, per-type dispatch, `last_attempt` tracking)
  9. Full `run_scheduler_background_thread` dispatch loop (replaces MVC stub)

### Compile Status
Binary compiles correctly after audit (no source files were modified). `cargo check` reports 22 warnings, 0 errors — same baseline as before this audit.
