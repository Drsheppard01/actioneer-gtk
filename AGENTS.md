AGENTS.md

This repository includes guidance for automated coding agents (Copilot-style) in `.github/copilot-instructions.md`.

This file documents the responsibilities and workflow for agents working on this repo.

Agent responsibilities
- Read `.github/copilot-instructions.md` before making changes. It contains the project's runtime, UI, and concurrency rules.
- Prefer in-repo docs under `docs/` for widget usage instead of external web pages.
- When changing UI code, consult `docs/libadwaita/` for widget behavior and constraints.
- Run `cargo fmt` and `cargo check` after edits.
- Add or update tests for functional changes. Unit tests live in `src/` files using `#[cfg(test)]`.

Development workflow for agents
1. Read `.github/copilot-instructions.md` and this `AGENTS.md`.
2. **Use TODO.md as the single source of truth for progress tracking.** Update TODO.md with:
   - Mark items as [🔄] when starting work
   - Mark items as [✅] when completed
   - Add detailed notes in the "Recent Updates" section
   - Keep the file current throughout the session
3. Make minimal, well-scoped edits. Prefer edits that are small and testable.
4. Run format and build checks locally in the workspace. Use the project's cargo toolchain.
5. When adding HTTP caching or background work, ensure:
   - Network calls run on Tokio via `crate::runtime_handle().spawn(...)`.
   - GTK widget updates happen on GLib main thread via `glib::MainContext::default().spawn_local(...)` or `glib::idle_add_local_once(...)`.
6. Push changes and open a PR. Ensure `cargo clippy -- -D warnings` passes for new code.

Hand-off to Copilot Coding Agent
- If you want an asynchronous agent to continue implementing a large task, add the hashtag `#github-pull-request_copilot-coding-agent` to the PR description and include the task body. The agent will create a branch and follow the instructions.

Notes
- The project uses `parking_lot::Mutex` for shared state in many places. Use `Arc<Mutex<T>>` for state passed to tokio tasks.
- There is an existing ETag caching implementation in `src/api/http.rs`. If you update API endpoints, consider reusing that mechanism.

Contact
- If something is unclear, open an issue or ping the repo owner in GitHub.