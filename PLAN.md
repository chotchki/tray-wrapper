# PLAN.md

Working through the improvements identified in the codebase audit. Phases are ordered by ratio of value to risk: quick wins first, then test correctness, then the testability refactor that unlocks real coverage.

## Phase 1 — CI hygiene and shutdown-race bug

Small, mostly mechanical. The `.expect` fix is the only real bug in the bunch but it ships with the rest because they all touch the same areas.

- [x] 1.1 Add `cargo clippy --all-targets -- -D warnings` step to `.github/workflows/build_test.yml` (after fmt, before build).
- [x] 1.2 Add `cargo build --examples --locked` step to the same workflow so the Axum example can't silently rot.
- [x] 1.3 Fix shutdown-race panics in `src/tray_wrapper.rs:93-112` — the five `elp.send_event(...).expect("Event Loop Closed!")` calls can fire during normal shutdown when the user quits mid-`await`. Replace with `let _ = elp.send_event(...)` and `break` out of the loop on send failure (the event loop being gone means we're done).
- [x] 1.4 Simplify `MenuState::quit_matches` (`src/menu_state.rs:60-68`) — the `if X { true } else { false }` is just `X`. Likely flagged by 1.1 anyway.
- [x] 1.5 Remove stale comment "The error type doesn't matter in this case" at `src/tray_wrapper.rs:76` — `update_tray_icon` returns `()`, the comment is a leftover.
- [x] 1.6 Rename `TrayWrapperError::RunTime` (wraps `std::io::Error` from `Runtime::new()`) to something honest like `TokioRuntimeInit`. This is a public API change — note in changelog.

**Phase exit:** all boxes ticked, `cargo test` green on all three CI platforms, no clippy warnings.

## Phase 2 — Test correctness

The existing UI tests are passing but lying. Fix that before adding more.

- [ ] 2.1 Fix `ui_tests/restart.rs` — the generator currently returns `Exit` on the first call, so the `Continue` branch never runs and the assertion `*runs == 1` confirms it. Invert the logic: return `Continue` on the first call, `Exit` on the second, assert `*runs == 2`. The test name finally matches what it does.
- [ ] 2.2 Add a `ui_tests/exit_with_error.rs` (declared `harness = false` in `Cargo.toml`) covering the `ContinueRunning::ExitWithError` path — currently exercised by zero tests.

**Phase exit:** restart test actually restarts, error-exit path is covered by an e2e test, all UI tests pass under xvfb.

## Phase 3 — Testability refactor (the big one)

Goal: pull pure logic out from under the winit event loop so it's reachable from ordinary `#[test]` functions. This is what actually moves coverage.

- [ ] 3.1 Extract a pure `fn status_display(status: &ServerStatus) -> (Option<&str>, &str)` (or similar — name the tuple if it improves clarity) returning the tray title overlay and the menu-item text. `MenuState::update_tray_icon` becomes a thin wrapper that calls this and applies the values.
- [ ] 3.2 Unit tests for `status_display` covering all four `ServerStatus` variants including the `Stopped(s)` and `Error(e)` payload pass-through.
- [ ] 3.3 Extract the spawned task's restart loop (`src/tray_wrapper.rs:89-117`) into a free function generic over a sender trait, e.g. `async fn run_server_loop<S: StatusSender>(gen: ServerGenerator, sender: S)`. `EventLoopProxy<UserEvent>` gets a thin `impl StatusSender` wrapper.
- [ ] 3.4 Unit tests for `run_server_loop` with a `Vec`-backed mock sender, covering: `Continue` then `Exit`, `Continue` then `ExitWithError`, immediate `Exit`, immediate `ExitWithError`. Assert the exact event sequence.
- [ ] 3.5 Update module docs / `lib.rs` doc comment if the public surface shifted (the `StatusSender` trait may need to stay private).

**Phase exit:** all boxes ticked, line coverage on `src/menu_state.rs` and `src/tray_wrapper.rs` measurably up (codecov delta visible on the PR), e2e UI tests still green.

## Phase 4 — Miri (optional, recommend skipping)

Only worth doing if Phase 3 lands and you want belt-and-suspenders on the pure extracted code. Caveats up top of the plan apply: miri can't touch the FFI-heavy parts, and there's no `unsafe` in our own code for it to scrutinize.

- [ ] 4.1 Decide go/no-go after Phase 3. Default: no-go.
- [ ] 4.2 If go: add a `miri` job to CI running `cargo +nightly miri test` filtered to the pure modules only (the FFI-touching tests will need `#[cfg_attr(miri, ignore)]`).

---

When a phase finishes, summarize what shipped and sweep its section to `PLAN_ARCHIVE.md`.
