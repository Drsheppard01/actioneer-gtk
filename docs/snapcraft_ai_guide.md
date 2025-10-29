# Snapcraft Build Guide for Actioneer AI Agents

This note distills the Snapcraft documentation fetched during the investigation into a checklist tailored for this repository. Follow these steps before touching the workflow or `snapcraft.yaml`.

## Environment setup
- Install LXD and ensure the managed instance can start (`sudo snap install lxd`, `lxd init`). Snapcraft 8 defaults to managed LXD builds when `--use-lxd` is passed.
- Install Snapcraft from the snap store (`sudo snap install snapcraft --classic`). Version 8.12.0 or newer matches the docs we captured.
- Run all commands from the repository root. A symlink (`snapcraft.yaml → snap/snapcraft.yaml`) exposes the manifest at the root so Snapcraft mounts the full workspace in LXD.
- Keep Rust available via `rustup` because the `rust` plugin honours the host toolchain when bootstrapping (Snapcraft refreshes it inside LXD).

## Project layout expectations
- `snapcraft.yaml` lives in `snap/` and is consumed through the root-level symlink. Do **not** duplicate the file.
- `snap/.snapcraftignore` was moved to the root (`.snapcraftignore`) so the entire workspace is filtered before upload.
- Assets referenced in `override-build` live at `data/me.spaceinbox.actioneer.desktop` and `icons/icons/hicolor/scalable/apps/actioneer.svg`. Paths are relative to repository root because `source: .` is used.

## Snapcraft.yaml checkpoints
1. **Metadata** — confirm `name`, `version`, `summary`, `description`, `grade`, `confinement`, `license`, `contact`, and URLs stay accurate (see docs on "Configure package information").
2. **Base** — `core24` matches the GNOME 46 extension. Changing it requires revisiting build dependencies.
3. **Architectures** — Snapcraft 8 replaces `architectures` with `platforms`. `platforms:` already declares `amd64` and `arm64`; keep both so the matrix build succeeds.
4. **Part configuration** — the `rust` plugin plus `source: .` assumes the workspace has `Cargo.toml` at the root. Do not move the manifest.
5. **Override build hook** — `craftctl default` runs the stock Rust build (`cargo install ...`). Extra installs copy desktop and icon assets into `meta/gui`.
6. **Stage packages** — libadwaita/libgtk/libssl ship runtime GTK stack. Use `stage-packages` for runtime libraries, `build-packages` for headers + pkgconfig.
7. **Slots/plugs** — the DBus session slot exposes `me.spaceinbox.actioneer`; keep it aligned with the desktop file `DBusActivatable` entry.

## Local build flow (`--use-lxd`)
1. `snapcraft clean actioneer --use-lxd` — reset the managed instance when you change dependencies or source layout.
2. `snapcraft pack --use-lxd --build-for=<arch>` — build and pack for a specific architecture. Omit `--build-for` to use the host architecture during debugging.
3. Outputs land in the repository root as `actioneer_<version>_<arch>.snap`. Remove them after tests to keep the tree clean.
4. Review lint diagnostics printed after `pack`. They are only warnings today (unused GUI libs, missing `donation` link) but note anything unstable.

## CI workflow expectations
- Workflow should run from the repo root so the symlinked manifest is detected automatically (`snapcraft pack --use-lxd --build-for=${{ matrix.arch }}`).
- Use separate output directories per architecture (`snap-output/<arch>/`) before uploading artifacts.
- Keep the cache step for Cargo but remember Snapcraft performs its own Rust build inside LXD; the cache mainly speeds up the initial `cargo` invocation during the hook.
- Ensure the Snapcraft store credentials are passed via `SNAPCRAFT_STORE_CREDENTIALS`; `snapcraft whoami` is the quick validation.
- After `pack`, call `snapcraft upload` (alias `snapcraft push`) with `--release edge` as currently configured.

## Troubleshooting checklist
- **Missing `prime/meta/snap.yaml`** — means the `pack` command did not consume the `prime` dir; verify `snapcraft pack prime` target and inspect `prime/meta/` contents.
- **`Cargo.toml` not found** — happens when Snapcraft cannot see the repo root. Always invoke Snapcraft from the top-level directory using the provided symlink.
- **Filename too long / recursive copy** — avoid symlinking the project back into `snap/`; rely on the symlinked manifest instead.
- **GTK runtime issues** — ensure `stage-packages` matches the GNOME platform (libadwaita-1-0, libgtk-4-1) and that the GNOME extension remains enabled.
- **LXD permissions** — if `lxc` complains about projects, run `lxd init --auto` before Snapcraft so managed instances can start.

Cross-reference:
- Ubuntu Snapcraft docs > *Configure package information*, *Select a base*, *Select platforms*, *Manage build dependencies*, *Add configuration options*, *Override default build process*.
- Ubuntu Snapcraft docs > *Customize the lifecycle* for notes on `craftctl` hook usage.

Keep this guide updated whenever the snap layout or workflow changes.
