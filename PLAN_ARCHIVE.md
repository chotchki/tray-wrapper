# PLAN_ARCHIVE.md

Completed phases swept from `PLAN.md`.

## Phase 1 — CI hygiene and shutdown-race bug — _shipped in `e63ee73`_

Six small changes; the shutdown-race fix was the only real bug.

- [x] 1.1 Add `cargo clippy --all-targets -- -D warnings` step to `.github/workflows/build_test.yml` (after fmt, before build).
- [x] 1.2 Add `cargo build --examples --locked` step to the same workflow so the Axum example can't silently rot.
- [x] 1.3 Fix shutdown-race panics in `src/tray_wrapper.rs:93-112` — the five `elp.send_event(...).expect("Event Loop Closed!")` calls can fire during normal shutdown when the user quits mid-`await`. Replaced with `let _ = elp.send_event(...)` and `break` out of the loop on send failure.
- [x] 1.4 Simplified `MenuState::quit_matches` to a `matches!` expression.
- [x] 1.5 Removed stale comment "The error type doesn't matter in this case" at `src/tray_wrapper.rs:76`.
- [x] 1.6 Renamed `TrayWrapperError::RunTime` → `TokioRuntimeInit`. Breaking pre-1.0 API change; bumped `Cargo.toml` version `0.3.2 → 0.4.0` in the same commit.

## Phase 2 — Test correctness — _shipped in `29a6777`_

Existing UI tests were passing but lying.

- [x] 2.1 Fixed `ui_tests/restart.rs` — generator was returning `Exit` on the first call, so the `Continue` branch never ran. Inverted the logic; assertion now `*runs == 2`.
- [x] 2.2 Documented the `ContinueRunning::ExitWithError` asymmetry on the variant's doc comment — unlike `Exit` it does NOT terminate the event loop, by design, so the user can read the error before manually quitting.

**Note on coverage:** the originally-planned e2e test for `ExitWithError` was dropped — it can't be e2e because the event loop intentionally stays up after the error. The path is instead covered by Phase 3.4's mock-sender unit tests, which assert the exact `Running → Error("...")` event sequence (a stricter check than e2e anyway).

## Phase 3 — Testability refactor — _shipped in `bd3a9e1`_

Pulled pure logic out from under the winit event loop. Test count went from 0 unit tests + 2 UI tests → 8 unit tests + 2 UI tests.

- [x] 3.1 Extracted pure `fn status_display(status: &ServerStatus) -> (&'static str, &str)` in `menu_state.rs`. `MenuState::update_tray_icon` is now a thin wrapper.
- [x] 3.2 Four unit tests for `status_display` covering all `ServerStatus` variants including string pass-through.
- [x] 3.3 Extracted the restart loop into `pub(crate) async fn run_server_loop<S: StatusSender>` in a new `server_loop` module. `EventLoopProxy<UserEvent>` gets a thin `impl StatusSender` in `event_loop.rs`. `tray_wrapper.rs` now just calls `rt.spawn(run_server_loop(sg, elp))`.
- [x] 3.4 Four unit tests for `run_server_loop` with a `Vec`-backed mock sender pinning the exact event sequence on every termination path: immediate `Exit`, immediate `ExitWithError`, `Continue → Exit`, `Continue → ExitWithError`.
- [x] 3.5 No public-surface shift — trait + helper are `pub(crate)`, `status_display` is private, `ServerStatus` lives in a private module. No doc updates needed.

## Phase 5 — Coverage report visibility — _shipped in `08f0886`_

Coverage workflow + badge already existed; this phase made the existing data more useful as a code-review signal.

- [x] 5.1 Added a `comment:` block to `.codecov.yml` (layout: `reach, diff, flags, files`, `require_changes: false`) so each PR gets an automatic Codecov comment with per-file coverage diff.

**Verification (observational, post-merge):** the next codecov run on main should reflect the Phase 3 unit tests (badge percentage moves up); the next PR opened against main should show an auto-generated Codecov comment.

## Phase 4 — Miri — _no-go_

Decided not to add miri. Miri can't execute the FFI calls that dominate this crate's runtime (winit / tray-icon / gtk / objc2-core-foundation), and the crate has zero `unsafe` of its own — so miri's scope would be limited to the pure extracted modules from Phase 3, which already have full unit-test coverage. The dependencies' `unsafe` is not something miri can verify from a downstream crate.

- [x] 4.1 Decided no-go after Phase 3.
- [ ] 4.2 ~~Add miri CI job~~ — skipped per 4.1.
