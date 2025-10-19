# Tokio + GTK integration audit and fix plan

Date: 2025-10-19

This document audits Rust source files for patterns that commonly cause runtime conflicts when mixing Tokio and the GLib/GTK main loop. It lists files that need changes, the concrete issue per file, and a small, low-risk plan to fix them. Follow the plan in small PRs (one or two files per PR) and run the build/tests after each change.

Summary of findings
- The repository already uses a background Tokio runtime and a global `Handle` (`src/main.rs`) — good. However many UI handlers await Tokio JoinHandles or use `.await` on JoinHandles inside GLib-local futures. This pattern can cause subtle execution ordering problems and risk moving non-Send GTK types across threads.
- Common problematic patterns found:
  - Awaiting a Tokio JoinHandle inside a GLib `spawn_local` future (e.g., `let _ = handle.await` inside `spawn_local`).
  - Calling `crate::runtime_handle().spawn(...).await.unwrap()` inside GLib contexts. This awaits the JoinHandle on the GLib executor rather than letting Tokio handle the async work and marshalling results back to GLib.
  - Using `.await.unwrap()` on JoinHandles returned by runtime spawns inside GLib contexts.

Files with occurrences (high-level)
- `src/main.rs`
- `src/ui/main_window.rs` (multiple locations)
- `src/ui/detail_view.rs`
- `src/ui/tasks/repo_status.rs`
- `src/ui/auth_window.rs`
- `src/ui/tasks/favorites_observer.rs`
- `src/ui/job_logs_window.rs`
- `src/ui/sidebar.rs`
- `src/ui/run_jobs_window.rs`
- `src/ui/workflow_runs_window.rs`
- `src/ui/preferences_window.rs`
- `src/preferences.rs`
- `src/favorites.rs`

Concrete issues & recommended changes (file-by-file)

- `src/main.rs`
  - Current pattern: spawns a thread which creates a `tokio::runtime::Runtime` and does `rt.block_on(async { pending::<()>().await })` to keep it alive, then stores its `Handle` in `OnceLock`.
  - Recommendation: Keep the single long-lived runtime approach, but construct it explicitly with the builder and `enable_all()` and `new_multi_thread()` so resource drivers are enabled. Example:

```rust
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .worker_threads(num_cpus::get())
    .build()?;
```

  - Reason: explicit builder ensures IO/time drivers enabled and control over worker threads. Keep running it on a background thread — do not call `block_on` on the GLib main thread.

- `src/ui/*.rs` (main_window, detail_view, job_logs_window, sidebar, run_jobs_window, workflow_runs_window, preferences_window, auth_window)
  - Patterns to fix (examples seen across these files):
    - `let repos_result = crate::runtime_handle().spawn(async move { client.list_repos().await }).await.unwrap();` inside a `spawn_local` closure.
    - `let _ = handle.await;` inside `glib::MainContext::default().spawn_local(async move { ... })` where `handle` is a `tokio::task::JoinHandle`.
  - Recommended pattern: Do the network/IO work entirely on the Tokio runtime, and then marshal results back to GLib with `spawn_local` (or `idle_add_local_once`). Avoid awaiting JoinHandles inside GLib executors. Example rewrite:

Bad:
```rust
glib::MainContext::default().spawn_local(async move {
    let repos_result = crate::runtime_handle().spawn(async move { client.list_repos().await }).await.unwrap();
    match repos_result { ... update UI ... }
});
```

Good:
```rust
// Spawn on tokio runtime — do network work there
let client_for_request = client.clone();
crate::runtime_handle().spawn(async move {
    let repos = client_for_request.list_repos().await;

    // Marshal result back to GLib main loop
    glib::MainContext::default().spawn_local(move || {
        match repos { Ok(r) => { /* update widgets */ }, Err(e) => { /* show error */ } }
    });
});
```

  - For patterns that need to wait for a group of tokio tasks to finish and then refresh UI, await inside a Tokio task and then call `spawn_local` once. Avoid awaiting the tokio JoinHandle inside `spawn_local`.

  - Also ensure Tokio-spawned tasks do not capture GTK types (`gtk::Widget`, `adw::ApplicationWindow`, `Rc<RefCell<...>>`, etc.). If state must be shared, convert to `Arc<Mutex<...>>` and only touch GTK widgets inside `spawn_local`.

- `src/ui/tasks/repo_status.rs` and `src/ui/tasks/favorites_observer.rs`
  - These modules spawn parallel tasks on the runtime. Ensure spawned closures only capture Send + 'static data (Strings, client clones, Arc clones) and not GTK objects.
  - If the code currently schedules UI updates after the work, confirm those use `glib::MainContext::default().spawn_local` (I found such scheduling already present in many places) — leave them.

- `src/preferences.rs`, `src/favorites.rs`
  - There are `await.unwrap()` calls on async manager methods. That's OK when invoked from an async context running on Tokio; if those are executed inside GLib-local futures, follow the same marshal pattern above.

Fix plan (practical step-by-step)
1. Create a small branch/PR for `main.rs` only: update runtime construction to use `Builder::new_multi_thread().enable_all()` and set worker threads. Verify that the `OnceLock<Handle>` pattern continues to work and run `cargo build` and `cargo test`.

2. Fix UI patterns grouped by area (multiple small PRs):
   - PR A: `src/ui/main_window.rs` — replace occurrences where `crate::runtime_handle().spawn(...).await.unwrap()` is awaited inside `spawn_local` with the recommended marshal pattern. Replace `let _ = handle.await` inside `spawn_local` by awaiting inside a Tokio task and then scheduling UI update via `spawn_local`.
   - PR B: `src/ui/detail_view.rs` and `src/ui/job_logs_window.rs` — same pattern.
   - PR C: other UI modules (`sidebar.rs`, `run_jobs_window.rs`, `workflow_runs_window.rs`, `preferences_window.rs`, `auth_window.rs`) — fix all similar patterns.

3. Fix any failing tests and update code where GTK objects are accidentally captured by Tokio spawns (convert to Arc/Mutex or move data only needed by background tasks).

4. Add targeted unit tests where possible to assert that manager APIs (favorites/preferences) can be called from Tokio tasks. Consider adding a small integration test that spawns a runtime and uses `glib::MainContext::default().spawn_local` to ensure no deadlocks (small, optional).

5. Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, and a manual run `cargo run` to exercise the UI.

Examples (concrete replacements)
- Replace pattern A (awaiting JoinHandle in GLib)

Before:
```rust
glib::MainContext::default().spawn_local(async move {
    let handle = crate::runtime_handle().spawn(async move { /* work */ });
    let _ = handle.await; // awaiting on GLib executor
    refresh_ui();
});
```

After:
```rust
// Do the waiting on Tokio runtime
crate::runtime_handle().spawn(async move {
    let _ = /* work */.await;
    // Now marshal back to GLib to refresh UI
    glib::MainContext::default().spawn_local(move || {
        refresh_ui();
    });
});
```

- Replace pattern B (spawn then await result via `.await.unwrap()` inside GLib)

Before:
```rust
glib::MainContext::default().spawn_local(async move {
    let result = crate::runtime_handle().spawn(async move { client.call().await }).await.unwrap();
    // use result
});
```

After:
```rust
let client_for_request = client.clone();
crate::runtime_handle().spawn(async move {
    let result = client_for_request.call().await;
    glib::MainContext::default().spawn_local(move || {
        // handle result on UI thread
    });
});
```

Testing & verification
- After each PR, run:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build
```

- Manual smoke test: `cargo run` and exercise the UI paths that trigger repository loading, auth, refreshing, and favorites changes.

Risk & mitigation
- Risk: Changing where `.await` happens could expose bugs where code expected results synchronously on the GLib side. Mitigation: Keep UI update logic local to `spawn_local` closures and use clear error handling when background tasks fail.
- Risk: Accidentally capturing GTK types in tokio tasks. Mitigation: Add a short linter rule or code review checklist: "Do not capture `gtk`/`adw`/`glib` types in `crate::runtime_handle().spawn` tasks".

Follow-up
- If you want, I can implement PR A (fix `src/ui/main_window.rs` occurrences) and run the build/tests here. Tell me to proceed and I'll update the file(s), run `cargo test`, and iterate until green (or report blockers).
