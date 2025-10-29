## TODO: Project status (short)

Status: ✅ Feature parity with macOS achieved — main work complete.

Keep this short. Purpose: quick status, where to look, and how to validate.

Key achievements
- Feature parity with macOS (runs, jobs, actions, trigger, auto-refresh)
- Cache-first loading for workflows/runs/jobs (in-memory DataCache)
- Job logs viewer, desktop notifications, and toast feedback
- Robust error handling, retries, and headless-safe UI tests

Where to look
- UI: `src/ui/` (main_window, detail_view, run_jobs_window, job_logs_window)
- API: `src/api/` (client, models, http helpers)
- Auth & storage: `src/auth/`, `src/storage/token_storage.rs`
- Tests: `tests/` and unit tests in `src/`

Quick validation (local)
1. Unit & logic tests: `cargo test`
2. UI tests (requires display): `cargo test -- --ignored`
3. Lint: `cargo clippy -- -D warnings`
4. Format: `cargo fmt`

Optional / next steps (low priority)
- Enhanced streaming job logs (advanced viewer)
- Persist cache to disk (optional)
- Small UI micro-optimizations or accessibility checks

Notes
- Full history and detailed session notes are preserved in git commits.
- For larger changes, run the quick validation steps above and ensure `cargo clippy -- -D warnings` passes.

---

Recent Updates
- [✅] 2025-10-29 — Reworked Snapcraft builds (root-level symlink + `.snapcraftignore`), produced `actioneer_1.0.0_arm64.snap` via `snapcraft pack --use-lxd`, and documented the process in `docs/snapcraft_ai_guide.md`.
- [✅] 2025-10-28 — Hardened demo mode (hide release toggle, mock rate limits, skip token checks).
- [✅] 2025-10-28 — Re-enabled welcome screen demo mode toggle and ensured switching to real auth exits demo (`src/ui/main_window.rs`, `src/ui/welcome_screen.rs`).
- [✅] 2025-10-28 — Added a `gtk::Viewport` inside the sidebar `adw::ClampScrollable` (`src/ui/main_window.rs`) to eliminate GTK warnings about missing scrollable properties.
- [✅] 2025-10-28 — Added MIT license, refreshed README, and introduced Snapcraft packaging (`LICENSE`, `README.md`, `snap/snapcraft.yaml`).

Recent: This file was compacted to keep only the essentials. For deep dive history, search commits or the previous long-form TODO in the repo history.