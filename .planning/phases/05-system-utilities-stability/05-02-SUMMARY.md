---
phase: 05-system-utilities-stability
plan: "02"
subsystem: system-utilities
tags: [rust, tui, sysinfo, error-rendering, n/a-fallback]

# Dependency graph
requires:
  - phase: 05-system-utilities-stability
    plan: "01"
    provides: Per-panel error tracking state (cpu_panel_error, memory_panel_error, disk_panel_error, process_panel_error) on AppState
provides:
  - All four System Utilities draw functions render stable N/A indicators when panel error flags are set
  - draw_resource_monitor: CPU and memory gauge sections branch on cpu_panel_error / memory_panel_error
  - draw_process_list helper: accepts process_error bool; renders header + single error row when true
  - draw_process_list_detailed: branches on process_panel_error; renders header + single error row when true
  - draw_disk_analyzer: branches on disk_panel_error; renders "Disk data unavailable" paragraph when true
affects:
  - 06-library-migration (draw functions use tui 0.19 API; migration to ratatui must preserve these error-state branches)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - panel-error-flag-branch: each draw function checks its error flag first and renders an N/A indicator (DarkGray gauge or Red paragraph) before touching snapshot data — stable output in any data state
    - per-row-na-fallback: zero numeric fields (memory_usage==0, run_time==0, disk_usage==0) render "N/A" rather than "0 MB" / "00:00:00" / "0 KB"

key-files:
  created: []
  modified:
    - src/main.rs

key-decisions:
  - "draw_process_list parameter process_error: bool passed from call site (app_state.process_panel_error) rather than making helper access app_state directly — keeps helper reusable and signature explicit"
  - "disk_panel_error branch placed before snapshot.disks.is_empty() check — error state takes priority over empty-disk state; both produce visible output, error is more specific"
  - "Per-row N/A fallback uses zero sentinel (mem_mb==0, run_time==0, disk_kb==0) consistent with the sentinel-value-failure-detection pattern established in 05-01"

patterns-established:
  - "panel-error-flag-branch: When a panel has an error flag set on AppState, check it first inside the snapshot branch and render an N/A placeholder — never crash or silently render stale/zero data"

requirements-completed: [SYS-01, SYS-02]

# Metrics
duration: 3min
completed: 2026-02-24
---

# Phase 5 Plan 02: System Utilities Stability — N/A Draw Rendering Summary

**All four System Utilities draw functions now branch on per-panel error flags to render visible N/A indicators (DarkGray gauges, Red error text, error-row tables) instead of crashing or silently rendering zero data**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-24T13:55:01Z
- **Completed:** 2026-02-24T13:57:30Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- CPU gauge in draw_resource_monitor branches on cpu_panel_error: shows DarkGray 0% gauge + "CPU monitoring unavailable" in Red when error
- Memory gauge in draw_resource_monitor branches on memory_panel_error: shows DarkGray 0% gauge + "Memory monitoring unavailable" in Red when error
- draw_process_list signature updated to accept process_error bool; renders table header + single "Process data unavailable" Red row when true
- draw_process_list_detailed branches on process_panel_error: same header + error row pattern; sort keybindings remain active (no-ops on empty data)
- draw_disk_analyzer branches on disk_panel_error before disks.is_empty() check: renders "Disk data unavailable" Red paragraph
- Per-row N/A fallbacks in both process tables: zero memory_usage, zero disk_usage, and zero run_time render "N/A" rather than "0 MB" / "0 KB" / "00:00:00"

## Task Commits

Each task was committed atomically:

1. **Task 1: Update draw_resource_monitor and draw_process_list for per-panel N/A rendering** - `b89cf3f` (feat)
2. **Task 2: Update draw_process_list_detailed and draw_disk_analyzer for N/A rendering** - `349fcb8` (feat)

**Plan metadata:** (docs commit — see final commit hash)

## Files Created/Modified
- `src/main.rs` - draw_resource_monitor (CPU/memory error branches), draw_process_list (process_error param + error row + per-row N/A), draw_process_list_detailed (process_panel_error branch + per-row N/A), draw_disk_analyzer (disk_panel_error branch)

## Decisions Made
- draw_process_list signature change (add process_error: bool) was clean — only one call site, passes app_state.process_panel_error directly
- disk_panel_error branch ordered before disks.is_empty() — error flag is a higher-priority condition than absence of disks
- Per-row N/A fallback threshold: zero sentinel (consistent with 05-01's sentinel-value-failure-detection pattern) — avoids introducing new threshold logic

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - cargo build and cargo check both produced zero errors on first attempt.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SYS-01 and SYS-02 fully closed: all System Utilities panels are panic-safe and user-visible error indicators are in place
- Phase 5 complete — all System Utilities draw paths produce stable, visible output regardless of whether sysinfo data is present
- Phase 6 (Library Migration: tui 0.19 → ratatui) can proceed; the error-state branches use the same tui widget API as normal rendering and will migrate cleanly

---
*Phase: 05-system-utilities-stability*
*Completed: 2026-02-24*
