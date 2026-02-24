# Requirements: Toolbox

**Defined:** 2026-02-23
**Core Value:** Task Scheduler and System Utilities must work reliably without crashing

## v1 Requirements

Requirements for v1.0 Stabilize milestone. Each maps to roadmap phases.

### Architecture & Migration

- [x] **ARCH-01**: Application has a single canonical entry point (`main.rs` only — `app.rs` removed)
- [ ] **ARCH-02**: Duplicate files (`app.rs`, `backups/main.bk.rs`) audited and removed
- [x] **ARCH-03**: Legacy root-level module files (`task_scheduler.rs`, `network_tools.rs`, `password_manager.rs`, `system_utilities.rs`) removed after audit
- [x] **ARCH-04**: `main.rs` references modules exclusively through `src/modules/` controllers, no root-level imports

### Error Handling

- [x] **ERR-01**: All `.unwrap()` / `.expect()` calls replaced with `Result`-based error handling
- [x] **ERR-02**: Email configuration save/load errors propagated to UI layer (not swallowed with `eprintln!`)
- [x] **ERR-03**: File I/O errors (tasks, passwords) propagated to caller

### Task Scheduler Stability

- [x] **TASK-01**: Task Scheduler handles malformed input (invalid dates, malformed email addresses) without panicking
- [ ] **TASK-02**: SMTP test returns user-visible error on failure instead of panicking

### System Utilities Stability

- [ ] **SYS-01**: System Utilities handles resource query failures (CPU, memory, disk) without panicking
- [ ] **SYS-02**: System Utilities shows user-visible error when monitoring data unavailable

### Terminal Safety

- [x] **TERM-01**: Terminal restored to normal state on panic (Drop guard on raw mode)

### Library Migration

- [ ] **LIB-01**: `tui` crate (0.19) replaced with `ratatui` (drop-in upgrade, functionally equivalent)

## v2 Requirements

Deferred to future milestone. Tracked but not in current roadmap.

### Security Hardening

- **SEC-01**: Encryption key derived from user password (Argon2/PBKDF2) rather than stored raw in `.env`
- **SEC-02**: Email credentials encrypted at rest (not stored in plaintext `email_config.json`)
- **SEC-03**: Input validation for email addresses and SMTP server fields

### Performance

- **PERF-01**: Email sending moved to background thread (non-blocking UI)
- **PERF-02**: System monitoring data cached with configurable refresh interval

### Test Coverage

- **TEST-01**: Unit tests for Task Scheduler CRUD operations
- **TEST-02**: Unit tests for error handling paths
- **TEST-03**: Integration tests for full task workflow

### Data Persistence

- **DATA-01**: Atomic file writes (write-then-rename) to prevent corruption
- **DATA-02**: Schema version field in all JSON files for forward compatibility

## Out of Scope

| Feature | Reason |
|---------|--------|
| New features (any module) | Stabilization first — foundation must be solid |
| Password Manager improvements | Not actively used; deprioritized |
| Network Tools improvements | Not actively used; deprioritized |
| GUI / non-terminal interface | Not part of this project |
| Multi-user support | Far future scope |
| Database backend | Not needed at current scale |
| Configuration UI | v2+ after foundation is stable |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| ARCH-01 | Phase 2 | Complete |
| ARCH-02 | Phase 1+2 | Audited (Phase 1 plan 01-01 complete) |
| ARCH-03 | Phase 1 | Audited (Phase 1 plan 01-02 complete) |
| ARCH-04 | Phase 2 | Complete |
| ERR-01 | Phase 3 | Complete |
| ERR-02 | Phase 3 | Complete |
| ERR-03 | Phase 3 | Complete |
| TASK-01 | Phase 4 | Complete |
| TASK-02 | Phase 4 | Pending |
| SYS-01 | Phase 5 | Pending |
| SYS-02 | Phase 5 | Pending |
| TERM-01 | Phase 3 | Complete |
| LIB-01 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0

---
*Requirements defined: 2026-02-23*
*Last updated: 2026-02-23 after plan 03-01 completion (ERR-01 and TERM-01 complete)*
