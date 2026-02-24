# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-23)

**Core value:** Task Scheduler and System Utilities must work reliably without crashing
**Current focus:** Phase 6 — Library Migration (ratatui upgrade)

## Current Position

Phase: 5 of 6 (System Utilities Stability) — COMPLETE
Plan: 2 of 2 in current phase — PLAN COMPLETE
Status: Plan 05-02 complete — all four System Utilities draw functions updated with per-panel N/A rendering; SYS-01 and SYS-02 requirements closed
Last activity: 2026-02-24 — Plan 05-02 complete (N/A gauges, error-row tables, disk error paragraph, per-row N/A fallbacks for zero fields)

Progress: [█████████░] 85%

## Performance Metrics

**Velocity:**
- Total plans completed: 11
- Average duration: 4.2 min
- Total execution time: 46 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-audit | 2 | 5 min | 2.5 min |
| 02-architecture-consolidation | 3 | 17 min | 5.7 min |
| 03-error-handling-foundation | 3 | 24 min | 8 min |
| 04-task-scheduler-stability | 2 | 10 min | 5 min |
| 05-system-utilities-stability | 2 | 6 min | 3 min |

**Recent Trend:**
- Last 5 plans: 3.2 min
- Trend: —

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Roadmap: Audit before delete — do not remove app.rs or backups/ until Phase 1 confirms they have no unique content
- Roadmap: Architecture consolidation precedes error handling — canonical structure must be established before unwrap() sweep
- Roadmap: Library migration last — ratatui upgrade deferred until codebase is clean to avoid compounding changes
- 01-01: src/main.rs confirmed as canonical crate root (Cargo default, no [[bin]] override)
- 01-01: src/app.rs is SAFE-TO-DELETE — identical logic to main.rs; only documentation differs
- 01-01: src/backups/main.bk.rs is SAFE-TO-DELETE — byte-for-byte identical to app.rs
- 01-01: Phase 2 must remove `mod app;` from main.rs before deleting app.rs
- 01-02: src/task_scheduler.rs is HAS-UNIQUE-CONTENT — SMS stack, full email dispatch, format_timestamp, and background thread dispatcher absent from MVC module
- 01-02: src/network_tools.rs is SAFE-TO-DELETE — MVC model is byte-for-byte copy of all code
- 01-02: src/password_manager.rs is SAFE-TO-DELETE — MVC model is full copy including crypto logic and test module
- 01-02: src/system_utilities.rs is SAFE-TO-DELETE — MVC model is byte-for-byte copy
- 01-02: All four legacy files are actively imported in main.rs — Phase 2 must update import sites before deleting
- 01-02: MVC controllers still import from legacy files at runtime — import migration is a Phase 2 prerequisite
- 02-01: model/mod.rs inline types are canonical — task.rs (Critical not Urgent) and scheduler.rs are incompatible stubs kept for Phase 3+ work
- 02-01: pub use task:: and pub use scheduler:: re-exports removed from model/mod.rs to prevent type conflicts
- 02-01: modules/mod.rs restricted to task_scheduler only — other MVC modules have unresolvable deps handled in plans 02-02 through 02-04
- 02-01: scheduler.rs rewritten without thiserror/crate::core — those crates/modules don't exist in current codebase
- 02-02: MVC view/controller sub-modules disabled in mod.rs for network_tools, password_manager, system_utilities — they reference crate::models, crate::ui, crate::shared which don't exist; only model exposed
- 02-02: modules/mod.rs now exposes all four modules; view/controller per-module files will need separate cleanup in later plans
- 02-02: bare network_tools::, system_utilities::, password_manager:: calls in main.rs replaced with full crate::modules::*::model:: paths
- 02-03: prepare_status_message made pub(crate) in main.rs — MVC submodules call it via crate::prepare_status_message without going through deleted app.rs
- 02-03: legacy handlers/ files also updated for consistency — zero crate::app:: references across all src/ files
- 02-03: mod app; was already absent from main.rs (audit was correct — app.rs was dead code, not declared as module)
- 03-01: panic hook registered before enable_raw_mode — restores terminal on panic path; TerminalCleanup drop guard handles normal exit; both paths covered
- 03-01: is_dark_mode() .expect() replaced with let-else returning false — cosmetic function, safe fallback
- 03-01: sort_by unwraps replaced with match (Option, Option) pattern — Ordering::Equal on missing IDs; IDs sourced from get_all_tasks() so gap unreachable
- 03-01: Mutex poison recovery via unwrap_or_else(|e| e.into_inner()) in parallel_speed_test threads
- 03-03: extract-result-before-notify pattern — clone error string from locked scheduler before calling app_state.push_notification to avoid borrow conflicts
- 03-03: add_task() changed from u32 to Result<u32, String> — only caller in main.rs, safe change to propagate save errors
- 03-03: SMTP gate in event-loop reminder check (not a separate handler) because background thread cannot push to app_state.notifications
- 03-03: Background thread save_tasks failures use eprintln! as fallback — cannot push to app_state from background thread
- 04-01: Validation flags reset on Esc but not on successful save — form fields cleared on save; flag persists but immediately clears when field length drops below threshold on next use
- 04-01: Color::Reset used as default border fallback — matches original Block::default().borders(Borders::ALL) visual without hardcoding a color
- 04-01: Char/Backspace arms restructured from match expressions to match blocks — required to run post-push validation without duplicating field dispatch logic
- 04-02: KeyCode::Char('t') arm placed before KeyCode::Char(c) catch-all — specific pattern takes priority without needing field guards
- 04-02: SMTP success uses NotificationSeverity::Warning (yellow) — no green/success severity variant exists; adding one not warranted for single use case
- 04-02: extract-result-before-notify pattern used for smtp_test_receiver polling — avoids borrow conflict between immutable try_recv and mutable push_notification
- 05-01: Arc::clone-before-mutate pattern — clone monitor Arc into local before if-let block so app_state borrow is freed for push_notification mutation
- 05-01: saturating_add used for u8 fail counters — prevents overflow on persistent sysinfo failures without panic
- 05-01: Single consolidated pre-draw refresh block (not two gated on active_menu) — monitoring stays current even when user navigates away from SystemUtilities
- 05-02: draw_process_list process_error: bool parameter passed from call site — keeps helper reusable and signature explicit vs accessing app_state directly
- 05-02: disk_panel_error branch ordered before disks.is_empty() — error flag takes priority over empty-disk state; both produce visible output, error is more specific
- 05-02: Per-row N/A fallback uses zero sentinel (mem_mb==0, run_time==0, disk_kb==0) — consistent with 05-01's sentinel-value-failure-detection pattern

### Pending Todos

None yet.

### Blockers/Concerns

- MVC view/controller files for network_tools, password_manager, system_utilities still reference crate::models, crate::ui, crate::shared — these need cleanup in later plans (currently compilation-disabled; crate::app references now fixed)

## Session Continuity

Last session: 2026-02-24
Stopped at: Completed 05-02-PLAN.md — N/A draw rendering for all four System Utilities panels; SYS-01 and SYS-02 closed; Phase 5 complete
Resume file: None
