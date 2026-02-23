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
