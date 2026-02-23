# Roadmap: Toolbox

## Overview

The v1.0 Stabilize milestone resolves a stalled MVC migration in a Rust TUI application. Three nearly-identical files of unclear ownership coexist with a partially-wired module structure, 105+ panic-prone unwrap() calls, and no terminal cleanup on crash. Work proceeds in dependency order: audit what exists before deleting anything, establish a single canonical entry point, eliminate unwraps broadly, harden the two actively-used modules (Task Scheduler and System Utilities), then upgrade the deprecated TUI library once the codebase is clean.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Audit** - Identify canonical code and map all duplicate/legacy files before touching anything
- [ ] **Phase 2: Architecture Consolidation** - Establish single entry point and route all imports through src/modules/
- [ ] **Phase 3: Error Handling Foundation** - Replace all unwrap()/expect() calls and add terminal drop guard
- [ ] **Phase 4: Task Scheduler Stability** - Harden Task Scheduler against bad input and SMTP failures
- [ ] **Phase 5: System Utilities Stability** - Harden System Utilities against resource query failures
- [ ] **Phase 6: Library Migration** - Upgrade deprecated tui crate to ratatui

## Phase Details

### Phase 1: Audit
**Goal**: Every duplicate and legacy file is understood — what it contains, what differs, and whether it is safe to remove
**Depends on**: Nothing (first phase)
**Requirements**: ARCH-02, ARCH-03
**Success Criteria** (what must be TRUE):
  1. A written diff summary exists comparing main.rs, app.rs, and backups/main.bk.rs — identifying what (if anything) each file contains that the others do not
  2. Every root-level legacy module file (task_scheduler.rs, network_tools.rs, password_manager.rs, system_utilities.rs) is catalogued with a verdict: safe-to-delete or has-unique-content
  3. The binary still compiles and runs after the audit — no accidental deletions
**Plans**: 2 plans

Plans:
- [x] 01-01-PLAN.md — Diff the three duplicate entry-point files (main.rs, app.rs, backups/main.bk.rs) and write audit verdicts
- [x] 01-02-PLAN.md — Catalogue the four root-level legacy module files against their src/modules/ counterparts and write final audit conclusions

### Phase 2: Architecture Consolidation
**Goal**: The application has one canonical entry point (main.rs) and all feature logic is wired through src/modules/ controllers — no root-level imports, no dead duplicates
**Depends on**: Phase 1
**Requirements**: ARCH-01, ARCH-04
**Success Criteria** (what must be TRUE):
  1. app.rs and backups/main.bk.rs are deleted; main.rs is the sole entry point
  2. All root-level legacy module files removed (confirmed safe by Phase 1 audit)
  3. main.rs imports feature logic exclusively through src/modules/*/controller paths — no use crate::task_scheduler or equivalent root-level imports
  4. The binary compiles cleanly with no dead_code or unused_imports warnings for the removed files
**Plans**: TBD

### Phase 3: Error Handling Foundation
**Goal**: The application no longer contains any call site that can panic on bad data — all I/O and parse failures propagate as Results, and a terminal panic crash leaves the shell usable
**Depends on**: Phase 2
**Requirements**: ERR-01, ERR-02, ERR-03, TERM-01
**Success Criteria** (what must be TRUE):
  1. Zero .unwrap() / .expect() calls remain in the codebase (verified by grep)
  2. Email config save and load return Result — a failure surface a message in the UI rather than printing to stderr and continuing
  3. Task file and password file I/O return Result to their callers — callers display errors to the user
  4. A panic (simulated or real) leaves the terminal in normal mode — raw mode is not left active after the process exits
**Plans**: TBD

### Phase 4: Task Scheduler Stability
**Goal**: The Task Scheduler cannot be crashed by any user input and surfaces all failure states as visible UI messages
**Depends on**: Phase 3
**Requirements**: TASK-01, TASK-02
**Success Criteria** (what must be TRUE):
  1. Entering an invalid date format in the task creation form shows an error message — the application does not panic
  2. Entering a malformed email address in any Task Scheduler field shows an error message — the application does not panic
  3. The SMTP test button returns a user-visible error message when the connection fails — the application does not panic or silently swallow the failure
**Plans**: TBD

### Phase 5: System Utilities Stability
**Goal**: The System Utilities module handles all resource query failures gracefully and always presents either a reading or an explicit error — never a crash
**Depends on**: Phase 3
**Requirements**: SYS-01, SYS-02
**Success Criteria** (what must be TRUE):
  1. If a CPU, memory, or disk query returns an error (e.g., sysinfo fails), the System Utilities view shows a user-visible error indicator rather than panicking
  2. The System Utilities module can be navigated freely even when one or more monitoring data sources are unavailable — no panic, no blank/frozen screen
**Plans**: TBD

### Phase 6: Library Migration
**Goal**: The tui 0.19 crate is replaced by ratatui and the application compiles and behaves identically from the user's perspective
**Depends on**: Phase 5
**Requirements**: LIB-01
**Success Criteria** (what must be TRUE):
  1. Cargo.toml references ratatui, not tui; tui is no longer a dependency
  2. The binary compiles without errors or warnings related to the library swap
  3. All four modules (Task Scheduler, System Utilities, Password Manager, Network Tools) render their views correctly after the upgrade — no visual regressions
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Audit | 2/2 | Complete | 2026-02-23 |
| 2. Architecture Consolidation | 0/TBD | Not started | - |
| 3. Error Handling Foundation | 0/TBD | Not started | - |
| 4. Task Scheduler Stability | 0/TBD | Not started | - |
| 5. System Utilities Stability | 0/TBD | Not started | - |
| 6. Library Migration | 0/TBD | Not started | - |
