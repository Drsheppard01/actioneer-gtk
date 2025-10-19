# Linux App Feature Parity with macOS App

This document tracks the implementation of missing features to achieve parity between the Linux (GTK) and macOS (SwiftUI) versions of Actioneer.

## Status Legend
- ✅ **Complete** - Feature fully implemented and tested
- 🔄 **In Progress** - Currently being worked on  
- ⏳ **Pending** - Not yet started
- ❌ **Not Needed** - Feature not applicable or already covered differently

---

## Core Behavior

### Repository Loading
- ✅ **Load repos once on startup** - No automatic periodic refresh of repos list
- ✅ **Manual refresh button** - User can manually reload repos list via button
- ✅ **No automatic repo list refresh** - Repos list only refreshes on manual button click

### Workflow/Run Updates  
- ✅ **Periodic refresh for selected repo only** - Background refresh uses preference interval
- ✅ **No refresh when no repo selected** - Background task only runs when repo is active
- ✅ **ETag support for API calls** - Reduces API usage by checking If-None-Match headers
- ✅ **User-configurable refresh interval** - ComboRow with 2s, 5s, 10s, 30s options
- ✅ **Workflows load only on repo selection** - Not fetched on app launch

---

## UI Components

### Main Window Header
- ✅ **Animated spinner during repo loading** - Shows activity while fetching repos
- ✅ **Rate limit display** - Shows remaining API calls and reset time
- ✅ **Refresh button** - Manual repo list reload
- ✅ **Auto-update rate limit after each API call** - Updates periodically during background refresh

### Sidebar
- ✅ **Favorites section** - Collapsible group showing favorite repos
- ✅ **Actions Enabled section** - Collapsible group for repos with workflows
- ✅ **Search/filter** - SearchEntry to filter repos by name
- ⏳ **Repository status indicators** - Show workflow success/failure counts per repo
- ⏳ **Last workflow run status icon** - Visual indicator next to repo name

### Detail View (Right Pane)
- ✅ **Expandable workflow groups** - Workflows as collapsible expanders
- ✅ **Expandable run groups** - Runs nested under workflows as expanders
- ✅ **Expandable job groups** - Jobs nested under runs as expanders
- ✅ **Preserve expansion state on refresh** - Keep expanded workflows open after reload
- ✅ **Refresh button with spinner** - Loading indicator during workflow refresh
- ✅ **Favorite toggle button** - Heart icon to add/remove repo from favorites
- ✅ **Run status icons** - Visual indicators (success/failure/in-progress) for each run
- ✅ **Relative timestamps** - "5 minutes ago", "2 hours ago" format for runs
- ✅ **Branch names in run subtitle** - Shows which branch triggered the run

### Run Actions
- ✅ **Open in GitHub button** - External link icon to open run in browser
- ✅ **Re-run workflow button** - Refresh icon for completed runs
- ✅ **Re-run failed jobs button** - Reboot icon for runs with failures
- ✅ **Cancel run button** - Stop icon for in-progress/queued runs
- ✅ **Confirmation dialogs** - Asks user before re-run/cancel actions

### Job Details
- ✅ **Job name display** - Show job names in list
- ✅ **Job status display** - Show status text (Success, Failure, etc.)
- ✅ **Job status icons** - Visual indicators per job
- ✅ **Open job in GitHub** - External link to job page (also allows viewing logs)
- ✅ **Job duration** - Shows how long each job took (e.g., "2m 34s")

---

## Preferences Window
- ✅ **Auto-refresh interval selector** - ComboRow with 2s, 5s, 10s, 30s
- ✅ **Desktop notifications toggle** - Enable/disable workflow completion notifications
- ✅ **Sound alerts toggle** - Enable/disable sound on workflow failure
- ⏳ **Persistent window size** - Remember and restore window dimensions
- ⏳ **Remember last selected repo** - Auto-select on next launch

---

## Data Management

### Caching
- ✅ **ETag-based HTTP caching** - Implemented in `src/api/http.rs`
- ⏳ **Workflow data cache** - Store workflows locally to reduce API calls
- ⏳ **Run data cache** - Store recent runs locally
- ⏳ **Cache invalidation** - Clear cache on sign-out or explicit refresh

### Favorites
- ✅ **Add/remove favorites** - Toggle button in detail view
- ✅ **Persistent favorites storage** - Saved to disk in JSON format
- ✅ **Favorites section in sidebar** - Separate collapsible group

### Authentication
- ✅ **OAuth Device Flow** - GitHub authentication via device code
- ✅ **Secure token storage** - Keyring integration for Linux
- ✅ **Sign out** - Clear token and return to auth screen
- ⏳ **Token refresh** - Auto-refresh expired tokens if supported

---

## Notifications & Feedback
- ⏳ **Desktop notifications** - System notifications for workflow completion
- ⏳ **Sound alerts** - Play sound on workflow failure
- ⏳ **Toast messages** - In-app temporary messages for actions (e.g., "Workflow re-run started")

---

## Polish & UX
- ⏳ **Keyboard shortcuts** - Cmd/Ctrl+R to refresh, etc.
- ⏳ **Empty states** - Better placeholders when no data
- ⏳ **Error handling** - User-friendly error messages
- ⏳ **Loading states** - Consistent spinners/placeholders
- ⏳ **Smooth animations** - Transitions for expand/collapse, list updates
- ⏳ **Accessibility** - Screen reader support, keyboard navigation

---

## Testing
- ⏳ **Automated UI tests** - Test key user flows
- ⏳ **ETag caching tests** - Verify API call reduction
- ⏳ **Background refresh tests** - Ensure proper task lifecycle
- ⏳ **Favorites persistence tests** - Verify save/load works

---

## Implementation Progress

### Current Session Work - COMPLETED ✅
1. ✅ Verified ETag implementation exists
2. ✅ Verified preferences ComboRow is correct
3. ✅ Verified background refresh only runs for selected repo
4. ✅ Confirmed no automatic repo list refresh
5. ✅ Auto-update rate limit display in background refresh loop
6. ✅ Confirmed confirmation dialogs exist for destructive actions
7. ✅ Confirmed status icons, relative timestamps, branch names implemented
8. ✅ Implemented job duration display
9. ✅ Verified job logs accessible via external GitHub link
10. ✅ Created comprehensive documentation (PARITY_TODO.md, SESSION_SUMMARY.md)

### Next Priority Items
1. ⏳ Desktop notifications for workflow completions
2. ⏳ Repository status indicators in sidebar (workflow success/failure counts)
3. ⏳ Persistent window size across sessions
4. ⏳ Remember last selected repo for auto-selection on launch
5. ⏳ Sound alerts for workflow failures

### Nice to Have
- ⏳ Toast messages for in-app feedback
- ⏳ Keyboard shortcuts (Ctrl+R to refresh, etc.)
- ⏳ Improved empty states
- ⏳ Animation polish
- ⏳ Accessibility enhancements
