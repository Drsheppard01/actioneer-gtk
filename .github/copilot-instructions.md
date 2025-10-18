<!-- Actioneer-gtk: Copilot / AI agent instructions -->
# Quick guide for automated coding agents

This repository is a native GTK4/libadwaita desktop client for GitHub Actions written in Rust. The notes below focus on the patterns and files an AI coding agent should know to make safe, useful changes quickly.

- Repo entry & runtime
  - App entry: `src/main.rs`. A Tokio runtime is spawned on a background thread and a global handle is stored via `OnceLock`. Use `crate::runtime_handle()` to run async HTTP work on the background runtime.
  - UI run-loop: GTK / libadwaita run on the GLib main loop. Never touch GTK widgets from Tokio threads.

- Key modules (big picture)
  - `src/ui/` — all UI components and glue. `src/ui/main_window.rs` shows most interaction patterns (loading repos, refreshing UI, connecting buttons).
  - `src/api/` — GitHub client, endpoints and models. See `src/api/client.rs` and `src/api/models.rs` for API surface.
  - `src/auth/` — OAuth Device Flow implementation (`device` module). Changes to auth must ensure TokenStorage interaction remains compatible.
  - `src/storage/token_storage.rs` — secure token handling via system keyring. This file contains a live test of the keyring; edits can affect developer machines.
  - `src/cache.rs`, `src/preferences.rs`, `src/favorites.rs` — app-level caching and persistence helpers used throughout the UI.

- Concurrency & integration patterns (important)
  - Background HTTP and long-running work: run on Tokio via `crate::runtime_handle().spawn(async move { ... })` (see `load_repositories`, `spawn_repo_status_tasks`).
  - UI updates: schedule UI changes on GLib using `glib::MainContext::default().spawn_local(...)` or `glib::idle_add_local_once(...)`. Follow the code in `src/ui/main_window.rs` as the canonical pattern.
  - Shared state: use `Arc<Mutex<T>>` (parking_lot::Mutex) for data shared between UI and background tasks. UI-specific ownership often uses `Rc<RefCell<...>>` for widgets/panes.

- Authentication & token handling
  - Token lifecycle lives in `TokenStorage`. `TokenStorage::new()` performs a keyring test and may return `KeyringUnavailable`. Handle that explicitly — the UI currently falls back to showing the auth window.
  - The OAuth device flow UI is in `src/ui/auth_window.rs` and the flow implementation in `src/auth/device.rs`.

- How to add API calls safely
  - Add or change endpoints under `src/api/` and update `client.rs` for higher-level helpers.
  - In UI code, clone the client and run HTTP calls on the Tokio runtime. After awaiting the result, marshal results back to the GLib main thread before touching GTK widgets. Example pattern used in repo:

```rust
// run HTTP on tokio
let repos_result = crate::runtime_handle().spawn(async move { client.list_repos().await }).await.unwrap();
// then update UI on glib/main thread
glib::MainContext::default().spawn_local(async move { /* refresh widgets */ });
```

- Build, test, and quality gates (must-do in PRs)
  - System deps (Ubuntu/Debian): `sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config` (see `README.md`).
  - Build: `cargo build`; Run: `cargo run` (reads `.env` when provided).
  - Tests: `cargo test` (there are unit tests such as token storage lifecycle).
  - Formatting & linting: `cargo fmt` and `cargo clippy -- -D warnings`. The project aims for zero warnings; a PR should not introduce warnings.

- Project-specific conventions
  - Prefer `parking_lot::Mutex` for shared state; code frequently clones `Arc<Mutex<T>>` before spawning tasks.
  - UI changes must use `glib::idle_add_local_once` or `spawn_local` to ensure GTK safety.
  - Token/keyring interactions are tested at runtime in `token_storage.rs` — avoid destructive cleanup in tests that run on developer machines.

- Files to reference when making changes
  - `src/main.rs` (runtime + app bootstrap)
  - `src/ui/main_window.rs` (primary UI patterns)
  - `src/storage/token_storage.rs` (keyring usage)
  - `src/api/client.rs` and `src/api/models.rs` (API surface)
  - `README.md` (dev setup and system deps)

If anything here is unclear or you need examples for a particular change (adding endpoints, changing auth, updating a UI pane), tell me which area to expand and I will update this file accordingly.
