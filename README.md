# Actioneer for Linux

Native GNOME application for managing GitHub Actions, built with Rust, GTK4, and libadwaita.

## Prerequisites

- Rust (1.70+)
- libadwaita (1.4+)
- pkg-config

### Ubuntu/Debian

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config
```

### Fedora

```bash
sudo dnf install gtk4-devel libadwaita-devel pkg-config
```

## Building

```bash
cargo build
```

## Configuration

The app uses GitHub OAuth for authentication. Default credentials are provided, but developers can use their own.

### Quick Start (Uses Default Credentials)

```bash
cargo run
```

### Using Your Own GitHub OAuth App

1. Copy the example configuration:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your GitHub OAuth app credentials:
   ```bash
   GITHUB_CLIENT_ID=your_client_id_here
   GITHUB_CLIENT_SECRET=your_client_secret_here
   ```

3. Run the app:
   ```bash
   cargo run
   ```

See [CONFIGURATION_GUIDE.md](CONFIGURATION_GUIDE.md) for detailed setup instructions.

## Running

```bash
cargo run
```

## Desktop Integration

To integrate Actioneer with GNOME Shell (application overview, search, and notifications), install the desktop entry and icons into your local data directories:

```bash
# Desktop entry
mkdir -p ~/.local/share/applications
cp data/me.spaceinbox.actioneer.desktop ~/.local/share/applications/

# App icons (hicolor theme)
mkdir -p ~/.local/share/icons
cp -r icons/icons/hicolor ~/.local/share/icons/
gtk-update-icon-cache ~/.local/share/icons/hicolor
```

Log out and back in (or restart GNOME Shell) after installation so the new icon and launcher appear in the overview. For system-wide installs, copy the same files to `/usr/share/applications` and `/usr/share/icons/hicolor` instead.

## Development

### Format code

```bash
cargo fmt
```

### Lint

```bash
cargo clippy -- -D warnings
```

### Run tests

```bash
cargo test
```

### Ensure zero warnings

```bash
cargo build 2>&1 | grep "warning:" | wc -l  # Should output: 0
cargo clippy -- -D warnings                  # Should pass with no errors
```

See [ZERO_WARNINGS_GUIDE.md](ZERO_WARNINGS_GUIDE.md) for maintaining code quality.

## Architecture

- `src/main.rs` - Application entry point with Tokio runtime integration
- `src/api/` - GitHub API client and models
- `src/auth/` - OAuth device flow implementation
- `src/storage/` - Secure token storage
- `src/cache.rs` - In-memory data caching system
- `src/preferences.rs` - Settings and preferences management
- `src/favorites.rs` - Favorites/bookmarks management
- `src/notifications.rs` - Desktop notification system
- `src/ui/` - GTK4/libadwaita UI components
   - `main_window.rs` - Main application window with repository list
   - `sidebar.rs` - Repository list grouping helpers
   - `detail_view.rs` - Embedded repository workflows pane
   - `detail_placeholder.rs` - Placeholder messaging utilities
   - `job_logs_window.rs` - Job logs viewer
   - `auth_window.rs` - OAuth device flow UI

## Features

### Completed
- [x] Project skeleton
- [x] API models (translated from Swift)
- [x] OAuth device flow auth module
- [x] Auth UI with device flow
- [x] Secure token storage (keyring)
- [x] GitHub API client implementation (all endpoints)
- [x] Main window UI with repository list
- [x] Repository list with search
- [x] Repository detail view (workflows)
- [x] Workflow runs view
- [x] Job logs viewer with monospace text
- [x] Workflow run cancellation
- [x] Refresh buttons on all views
- [x] Sign out functionality
- [x] Status icons for runs and jobs
- [x] Async/await integration with GTK
- [x] **Data caching system** - Reduces API calls
- [x] **Preferences manager** - Persistent settings
- [x] **Favorites system** - Bookmark repositories
- [x] **Notification foundation** - Desktop notifications structure

### Not Yet Implemented
- [ ] Workflow dispatch UI (trigger workflows manually)
- [ ] Desktop notifications for workflow completion
- [ ] Settings/preferences window
- [ ] Workflow favorites
- [ ] Multiple GitHub accounts
- [ ] Flatpak packaging
- [ ] AppStream metadata
- [ ] CI/CD automation

## Usage

1. **Launch the app**: Run `cargo run` or execute the built binary
2. **Sign in**: On first launch, you'll see the device flow auth screen
   - Copy the user code shown
   - Click "Open GitHub in Browser"
   - Paste the code and authorize the app
3. **Browse repositories**: The main window shows all your repositories
   - Use the search bar to filter repositories
   - Click refresh to update the list
4. **View workflows**: Click on any repository to see its workflows
5. **View runs**: Click on a workflow to see its recent runs
6. **View jobs**: Click on a run to see its jobs
7. **View logs**: Click on a job to see its logs
8. **Cancel runs**: In the jobs view, click the stop button to cancel a running workflow

## Technical Details

### Async Runtime Integration
The app uses a hybrid approach to handle async operations:
- **Tokio runtime**: Created at startup for HTTP/network operations (reqwest)
- **glib main loop**: Used for all UI updates via `glib::MainContext::default().spawn_local()`
- This prevents thread safety issues with GTK widgets while allowing async HTTP calls

### Authentication
- Uses GitHub OAuth Device Flow
- Tokens stored securely in system keyring
- Scopes: `repo`, `workflow`
- Client ID: `Iv1.b507a08c87ecfe98`

## License

MIT
