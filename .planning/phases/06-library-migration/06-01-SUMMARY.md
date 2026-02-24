---
phase: 06-library-migration
plan: "01"
subsystem: infra
tags: [ratatui, tui, crossterm, cargo, dependencies]

# Dependency graph
requires:
  - phase: 05-system-utilities-stability
    provides: stable codebase baseline before library swap
provides:
  - ratatui = "0.29" declared in Cargo.toml, tui removed, standalone crossterm removed
affects:
  - 06-02-source-migration (will update all tui:: imports to ratatui::)

# Tech tracking
tech-stack:
  added: [ratatui 0.29.0]
  patterns: [crossterm types accessed via ratatui::crossterm re-exports (no standalone crossterm dep)]

key-files:
  created: []
  modified: [Cargo.toml, Cargo.lock]

key-decisions:
  - "ratatui = 0.29 pinned to minor (0.29.x range) — 0.30.0 available but 0.29 is latest stable targeted by plan"
  - "crossterm removed from Cargo.toml entirely — ratatui re-exports crossterm types; no standalone dep needed"
  - "No source changes in this plan — tui:: imports in main.rs will fail cargo check until Plan 02 updates them"

patterns-established:
  - "Dependency swap first, source migration second — clean split keeps git blame readable"

requirements-completed: [LIB-01]

# Metrics
duration: 1min
completed: 2026-02-24
---

# Phase 6 Plan 01: Dependency Swap Summary

**Cargo.toml updated: tui 0.19 removed, standalone crossterm removed, ratatui 0.29 added; cargo fetch resolves dependency tree cleanly**

## Performance

- **Duration:** ~1 min (37 seconds)
- **Started:** 2026-02-24T15:01:29Z
- **Completed:** 2026-02-24T15:02:06Z
- **Tasks:** 1 of 1
- **Files modified:** 2 (Cargo.toml, Cargo.lock)

## Accomplishments
- Replaced `tui = "0.19.0"` with `ratatui = "0.29"` in Cargo.toml
- Removed `crossterm = "0.28.1"` from Cargo.toml (crossterm now accessed via ratatui::crossterm)
- Cargo.lock resolved ratatui v0.29.0 and its transitive dependencies successfully
- cargo fetch exits 0 — dependency baseline is clean for Plan 02 source migration

## Task Commits

Each task was committed atomically:

1. **Task 1: Swap tui for ratatui and remove standalone crossterm** - `715bbbd` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `Cargo.toml` - tui removed, crossterm removed, ratatui = "0.29" added
- `Cargo.lock` - 20 new packages locked for ratatui 0.29.0 and dependencies

## Decisions Made
- Used `ratatui = "0.29"` specifier (0.29.x range) — 0.29.0 is current; 0.30.0 was shown as available but plan specifies 0.29 as the target version
- Removed crossterm line entirely per user decision in CONTEXT.md: "Access crossterm types via ratatui::crossterm re-exports"
- Did not run `cargo check` — source still uses `tui::` imports which would fail; cargo fetch confirms the dependency tree without touching source

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None — cargo fetch resolved the dependency tree without errors on the first attempt.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Cargo.toml dependency baseline is established; Plan 02 can begin replacing `tui::` imports with `ratatui::` and `crossterm::` imports with `ratatui::crossterm::` throughout main.rs
- No blockers

## Self-Check: PASSED

- FOUND: Cargo.toml (ratatui present, tui absent, crossterm absent)
- FOUND: 06-01-SUMMARY.md
- FOUND: commit 715bbbd (feat(06-01): swap tui for ratatui, remove standalone crossterm)

---
*Phase: 06-library-migration*
*Completed: 2026-02-24*
