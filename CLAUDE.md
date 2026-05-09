# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`tray-wrapper` is a Rust library that wraps a long-running async server (e.g. an Axum app) with a system tray icon for status display and quit control. Primary target is macOS, where running a server without a tray icon is awkward; Linux/Windows are best-effort.

The crate is pre-1.0 and intentionally so: the `ServerGenerator` trait shape is a workaround for the absence of stable generators/coroutines in Rust (rust-lang/rust#43122). Don't try to "fix" it with a cleaner abstraction — the current `Arc<dyn Fn() -> Pin<Box<dyn Future>>>` shape is the deliberate compromise.

## Commands

```bash
cargo build --locked          # build
cargo fmt --all -- --check    # format check (CI gate)
cargo test                    # runs both unit tests and the harnessed UI tests
cargo test --test ui_basic    # run a single UI test (also: ui_restart)
cargo run --example axum      # run the Axum integration example
```

The two UI tests (`ui_tests/basic.rs`, `ui_tests/restart.rs`) are declared in `Cargo.toml` with `harness = false` because each spins up the real winit event loop and tray icon on the main thread — they can't share a process. On Linux CI they run under `xvfb`.

## Architecture

The runtime model has three coupled pieces; understanding the boundary between them is the key to working in this codebase:

1. **Main thread runs the winit event loop** (`event_loop.rs`, `tray_wrapper.rs`). This is non-negotiable on macOS — the tray icon and menu must be created and pumped from the main thread. `setup_event_loop()` builds an `EventLoop<UserEvent>` and installs a `MenuEvent` handler that forwards menu clicks back into the same loop as `UserEvent::Menu`.

2. **A separate Tokio runtime drives the user's server** (`tray_wrapper.rs`). `TrayWrapper` owns its own `tokio::runtime::Runtime` and spawns the server generator there. The server task communicates back to the UI by sending `UserEvent::ServerStatus(...)` / `UserEvent::ServerExit` through an `EventLoopProxy`. Quitting (menu click or `ContinueRunning::Exit`) triggers `runtime.shutdown_timeout(10s)` then `event_loop.exit()`.

3. **The server generator is a restart loop, not a one-shot** (`server_generator.rs`). `ServerGenerator = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ContinueRunning> + Send>> + Send + Sync>` is called repeatedly — each call must produce a *fresh* future. The future's return value (`Continue` / `Exit` / `ExitWithError`) decides whether to call the generator again. This is why it's a `Fn` returning a boxed future and not just a single `Future`: the wrapper needs the ability to re-await.

### Status state machine

`ServerStatus` (`server_status.rs`) drives `MenuState::update_tray_icon` (`menu_state.rs`), which mutates both the tray icon's title overlay (`?` / `""` / `X` / `E`) and the first menu item's text. The state transitions are emitted from the spawned task in `TrayWrapper::new_events` — `Running` is sent right before each `await`, then `Stopped` / `Error` after, with `ServerExit` on `Continue::Exit`.

### Init ordering quirk

The tray icon is created in `new_events` on `StartCause::Init`, *not* in `resumed` or `new()`. This is a deliberate workaround for tauri-apps/tray-icon#90. After init, on macOS, `CFRunLoop::wake_up` is called to force a redraw — without this the icon doesn't appear. Don't refactor icon creation earlier in the lifecycle.

### Platform conditionals

- Linux: `gtk::init()` is called from `create_tray_wrapper` before `run_app`. Linux deps (`gtk`, `glib`) and the macOS dep (`objc2-core-foundation`) are gated via `[target."cfg(target_os = ...)"]` in `Cargo.toml`.
- The CI matrix builds on macos / ubuntu / windows; ubuntu needs `libgtk-3-dev libxdo-dev libxkbcommon-x11-0 libappindicator3-dev`.

## Release flow

CI is triggered on push: `build_test.yml` runs fmt+build+test on the matrix, then dispatches `release_publish.yml`. The release workflow reads `package.version` from `Cargo.toml`, checks whether a `v$VERSION` tag exists, and if not: `cargo publish` to crates.io, then creates a draft GitHub release and pushes the tag. Bumping the version in `Cargo.toml` is the release trigger.
