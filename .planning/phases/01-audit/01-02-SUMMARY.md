---
phase: 01-audit
plan: 02
subsystem: audit
tags: [rust, legacy-files, mvc, diff, task-scheduler, network-tools, password-manager, system-utilities]

# Dependency graph
requires:
  - phase: 01-audit-01
    provides: "AUDIT.md Section 1 with safe-to-delete verdicts for entry-point duplicates"
provides:
  - "AUDIT.md Section 2 with explicit verdicts for all four root-level legacy module files"
  - "HAS-UNIQUE-CONTENT verdict for task_scheduler.rs with detailed migration checklist"
  - "SAFE-TO-DELETE verdicts for network_tools.rs, password_manager.rs, system_utilities.rs"
  - "Audit Conclusions section summarizing all Phase 2 prerequisites"
affects: [02-consolidate, 03-errors]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Audit-before-delete: comparison of legacy root files against MVC module counterparts before any deletion"

key-files:
  created:
    - ".planning/phases/01-audit/01-02-SUMMARY.md"
  modified:
    - ".planning/phases/01-audit/AUDIT.md"

key-decisions:
  - "src/task_scheduler.rs is HAS-UNIQUE-CONTENT — SMS notification stack (SmsConfig, send_sms_reminder, get_sms_gateway_email), full email dispatch (send_reminder_email, test_email_config, create_smtp_transport), format_timestamp, and complete background thread dispatcher are absent from MVC module"
  - "src/network_tools.rs is SAFE-TO-DELETE — src/modules/network_tools/model/mod.rs is a byte-for-byte copy of all functions and structs"
  - "src/password_manager.rs is SAFE-TO-DELETE — src/modules/password_manager/model/mod.rs is a full copy including AES-256-CBC crypto logic and test module"
  - "src/system_utilities.rs is SAFE-TO-DELETE — src/modules/system_utilities/model/mod.rs is a byte-for-byte copy including SystemMonitor, SystemHistory, and all sysinfo logic"
  - "All four legacy files are actively imported in main.rs — Phase 2 must update import sites before deleting"
  - "MVC controllers still import from legacy files at runtime, not from MVC model modules — import migration is a Phase 2 prerequisite"

patterns-established:
  - "Verdict pattern: compare function-by-function including helper functions, not just public API surface"
  - "MVC migration completeness check: verify controllers import from MVC model, not from legacy file"

requirements-completed: [ARCH-03]

# Metrics
duration: 2min
completed: 2026-02-23
---

# Phase 1 Plan 02: Legacy Module File Audit Summary

**Audited all four root-level legacy module files against MVC counterparts — network_tools.rs, password_manager.rs, and system_utilities.rs are safe-to-delete full copies, while task_scheduler.rs has unique SMS+email dispatch content requiring migration before deletion**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T17:41:45Z
- **Completed:** 2026-02-23T17:43:58Z
- **Tasks:** 1 of 1
- **Files modified:** 1

## Accomplishments

- Read all four legacy files (1729 lines total) and their MVC counterparts (10 files) in parallel
- Confirmed all four legacy files are actively imported in `src/main.rs` (lines 19–22, 25, 32, 34, 46)
- Established `src/network_tools.rs` as SAFE-TO-DELETE: `src/modules/network_tools/model/mod.rs` is a byte-for-byte copy of all code including `ping`, `parse_ping_output`, `SpeedTestResult`, `measure_speed`, `speed_test`, `parallel_speed_test`, and `download_chunk`
- Established `src/password_manager.rs` as SAFE-TO-DELETE: `src/modules/password_manager/model/mod.rs` is a full copy of all AES-256-CBC crypto logic, the `#[cfg(test)]` module, and all public functions
- Established `src/system_utilities.rs` as SAFE-TO-DELETE: `src/modules/system_utilities/model/mod.rs` is a byte-for-byte copy of `SystemMonitor`, `SystemHistory`, `SystemSnapshot`, `DiskInfo`, `ProcessInfo`, and all methods
- Established `src/task_scheduler.rs` as HAS-UNIQUE-CONTENT: the MVC `scheduler.rs` is a clean architectural rewrite missing the entire SMS notification stack, full email delivery system, `format_timestamp`, and the complete background thread dispatcher
- Appended Section 2 and Audit Conclusions to AUDIT.md with actionable Phase 2 migration checklist
- Confirmed `cargo check` still passes: 22 warnings, 0 errors (no src files modified)

## Task Commits

Each task was committed atomically:

1. **Task 1: Audit each root-level legacy module file against its src/modules/ counterpart** - `2e3d657` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `.planning/phases/01-audit/AUDIT.md` - Section 2 (legacy module audit) and Audit Conclusions appended

## Decisions Made

- Compared function-by-function rather than just public API surface — caught that `format_timestamp` and internal helpers like `create_smtp_transport` are unique to the legacy file
- Identified that MVC controllers still import from legacy files at runtime (e.g., `network_tools/controller/mod.rs` imports `ping` from `crate::network_tools`, not the MVC model) — documented as Phase 2 prerequisite
- Noted `src/modules/task_scheduler/model/mod.rs` is an intermediary file that partially re-exports MVC types but retains a full legacy `TaskScheduler` struct alongside the new `scheduler.rs` rewrite — the MVC module has two competing TaskScheduler definitions

## Deviations from Plan

None — plan executed exactly as written. No src files were modified; `cargo check` confirms 22 warnings (same as before), zero errors.

## Issues Encountered

None — all files read cleanly. The `src/modules/task_scheduler/model/mod.rs` contains a confusing hybrid of legacy code re-exported alongside the new MVC types, but this is a pre-existing condition documented in the audit, not an execution issue.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- AUDIT.md is complete: both Section 1 and Section 2 are written, all verdicts are explicit
- Phase 2 (Architecture Consolidation) has a clear ordered deletion checklist:
  1. Delete `src/backups/main.bk.rs` (immediate, no prerequisites)
  2. Remove `mod app;` from `main.rs`, then delete `src/app.rs`
  3. Migrate SMS+email stack from `src/task_scheduler.rs` into `src/modules/task_scheduler/`, then update import sites in `main.rs` and the MVC controller, then delete legacy file
  4. Update import sites for `network_tools`, `password_manager`, `system_utilities` in both `main.rs` and relevant MVC controllers, then delete those three legacy files
- The task_scheduler migration is the only complex work in Phase 2; all other deletions are straightforward import-site rewrites

## Self-Check

- FOUND: `.planning/phases/01-audit/AUDIT.md` (Section 2 appended, 12 SAFE-TO-DELETE/HAS-UNIQUE-CONTENT matches)
- FOUND: `## Section 2` header in AUDIT.md
- FOUND: `## Audit Conclusions` section in AUDIT.md
- FOUND: 4 `**Verdict:**` entries in AUDIT.md Section 2 (one per legacy file)
- FOUND: commit `2e3d657` (Task 1 — feat(01-audit-02))
- `cargo check` passes: 22 warnings, 0 errors

## Self-Check: PASSED

---
*Phase: 01-audit*
*Completed: 2026-02-23*
