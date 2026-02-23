# Toolbox

## What This Is

A Rust TUI (terminal user interface) productivity toolbox with four integrated modules: Task Scheduler, Password Manager, Network Tools, and System Utilities. Built with `tui` and `crossterm`, it runs entirely in the terminal as a compiled binary. The codebase is currently mid-refactor — a half-finished migration from a flat monolithic structure to a clean MVC architecture.

## Core Value

The Task Scheduler and System Utilities must work reliably without crashing — those are the modules in active use.

## Requirements

### Validated

- ✓ Terminal UI with tui + crossterm — existing
- ✓ Task Scheduler: tasks with priority, due dates, reminders — existing
- ✓ Task Scheduler: email notifications via SMTP (Gmail, Outlook, Yahoo) — existing
- ✓ Password Manager: AES-256 encrypted local storage — existing
- ✓ Network Tools: ping, speed tests, network info — existing
- ✓ System Utilities: CPU, memory, disk, process monitoring — existing
- ✓ Module structure in src/modules/ (partially implemented MVC) — existing

### Active

- [ ] Finish MVC migration — complete module separation, remove flat/root-level duplicates
- [ ] Eliminate 105+ panic-prone unwrap() calls with proper Rust error handling
- [ ] Resolve code duplication across main.rs, app.rs, and backups/main.bk.rs
- [ ] Stabilize Task Scheduler: robust error handling, no panics on bad input
- [ ] Stabilize System Utilities: robust error handling, no panics
- [ ] Clean up inconsistent module imports (root-level vs modules/)
- [ ] Establish single source of truth for application entry point

### Out of Scope

- New features — deferred until the foundation is solid
- Password Manager improvements — not actively used, deprioritized
- Network Tools improvements — not actively used, deprioritized
- GUI / non-terminal interface — not part of this project

## Context

This is a brownfield refactor. The codebase was being migrated from a single large main.rs (~3,430 lines) toward an MVC structure in src/modules/, but the migration stalled. Both the old flat structure and the new module structure coexist, creating confusion about which is canonical.

Key facts from codebase analysis:
- `src/main.rs` (~3,430 lines), `src/app.rs` (~3,272 lines), `src/backups/main.bk.rs` (~3,272 lines) are nearly identical — source of truth is unclear
- `src/modules/` contains MVC implementations (model/view/controller) for all 4 features — this is the target architecture
- Root-level files like `src/task_scheduler.rs`, `src/network_tools.rs`, etc. are legacy — should be removed after migration
- 105+ `.unwrap()` / `.expect()` calls — main source of crashes
- Email config and encryption errors silently swallowed instead of propagated

## Constraints

- **Tech stack**: Rust 2021 edition — no language change
- **Platform**: macOS/Linux only (uses Unix signal handling, ping command)
- **Scope**: Stabilization refactor only — no new features until this milestone is complete
- **Approach**: Investigate duplicate files before deleting — do not blindly delete without auditing

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Finish MVC migration (not flatten back) | Modules/ structure is cleaner and more maintainable long-term | — Pending |
| Audit app.rs and backups/ before deleting | User wants to understand what's there before removing | — Pending |
| Prioritize Task Scheduler + System Utilities | These are the actively used modules | — Pending |
| Replace unwrap() with proper error handling | Main source of panics/crashes | — Pending |

---
*Last updated: 2026-02-22 after initialization*
