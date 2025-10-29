# Snapcraft Build Guide for Actioneer AI Agents

This note distills the Snapcraft documentation fetched during the investigation into a checklist tailored for this repository. Follow these steps before touching the workflow or `snapcraft.yaml`.

## Environment setup
- Ensure Docker is available (`sudo apt-get install -y docker.io` on Ubuntu; GitHub-hosted runners already include it). The workflow uses the Snapcraft OCI image published at `ghcr.io/canonical/snapcraft`.
- Pull the image you plan to use locally, e.g. `sudo docker pull ghcr.io/canonical/snapcraft:8_core24` for core24 snaps.
- Run all commands from the repository root. A symlink (`snapcraft.yaml → snap/snapcraft.yaml`) exposes the manifest at the root so the container sees the full workspace mounted at `/project`.
- Keep Rust available via `rustup` if you want to run `cargo` locally outside the container (Snapcraft will bootstrap Rust inside the container during the build).

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

## Local build flow (Snapcraft Docker image)
1. `sudo docker run --rm -v "$PWD:/project" -w /project ghcr.io/canonical/snapcraft:8_core24 clean actioneer --destructive-mode` to wipe `parts/`, `prime/`, and related directories.
2. `sudo docker run --rm -v "$PWD:/project" -w /project ghcr.io/canonical/snapcraft:8_core24 pack --destructive-mode --build-for=<arch> --output snap-output/<arch>/actioneer_<arch>.snap` to build for a specific architecture.
3. Outputs land in `snap-output/<arch>/`. Remove them after tests to keep the tree clean, and run `sudo chown -R $USER:$USER snap-output parts prime stage` if you need to adjust permissions.
4. Review lint diagnostics printed after `pack`. They are currently warnings (unused GUI libs, missing `donation` link) but track regressions.

## CI workflow expectations
- Workflow should run from the repo root so the symlinked manifest is detected automatically. The current job shells into Docker with `ghcr.io/canonical/snapcraft:8_core24` and runs `snapcraft clean ... --destructive-mode` followed by `snapcraft pack ... --destructive-mode --build-for=${{ matrix.arch }}`.
- Use separate output directories per architecture (`snap-output/<arch>/`) before uploading artifacts.
- Keep the cache step for Cargo but remember Snapcraft performs its own Rust build inside the container; the cache mainly speeds up the initial `cargo` invocation during the hook.
- Snapcraft is still installed on the host via `snapd` so that `snapcraft whoami` and `snapcraft upload` can run against the store; keep those commands after the container build completes.
- Ensure the Snapcraft store credentials are passed via `SNAPCRAFT_STORE_CREDENTIALS`; `snapcraft whoami` is the quick validation.
- After `pack`, call `snapcraft upload` (alias `snapcraft push`) with `--release edge` as currently configured.

## Troubleshooting checklist
- **Missing `prime/meta/snap.yaml`** — means the `pack` command did not consume the `prime` dir; verify `snapcraft pack` completed and inspect `prime/meta/` contents.
- **`Cargo.toml` not found** — happens when Snapcraft cannot see the repo root. Always invoke Docker from the top-level directory so `/project` maps to the workspace and keep the root symlink intact.
- **Filename too long / recursive copy** — avoid symlinking the project back into `snap/`; rely on the symlinked manifest instead.
- **GTK runtime issues** — ensure `stage-packages` matches the GNOME platform (libadwaita-1-0, libgtk-4-1) and that the GNOME extension remains enabled.
- **Permission denied removing build dirs** — container builds run as root. Use `sudo chown -R $USER:$USER parts prime stage snap-output` after a build if cleanup fails.

Cross-reference:
- Ubuntu Snapcraft docs > *Configure package information*, *Select a base*, *Select platforms*, *Manage build dependencies*, *Add configuration options*, *Override default build process*.
- Ubuntu Snapcraft docs > *Customize the lifecycle* for notes on `craftctl` hook usage.

Keep this guide updated whenever the snap layout or workflow changes.
