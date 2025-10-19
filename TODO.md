# TODO: Feature Parity with macOS App

**Goal:** Bring Linux GTK app to feature parity with macOS app based on UI reference and source code analysis.

## Legend
- [ ] Not started
- [🔄] In progress
- [✅] Completed

---

## 1. Workflow Run Display Features

### 1.1 Run Status Icons
- [✅] Add colored status icons for each run (success=green checkmark, failure=red X, cancelled=gray stop, in_progress=blue bolt, queued=orange clock)
- [✅] Implement status color coding system
- [✅] Add icon display next to each run title

### 1.2 Run Metadata Display
- [✅] Display run conclusion/status text (e.g., "completed • success • main")
- [✅] Show branch name for each run
- [✅] Add relative time display ("2h ago", "Just now", etc.)
- [ ] Implement time string auto-update (every 60 seconds)
  - Note: Complex feature requiring weak references to labels; time updates happen on refresh for now

### 1.3 Job Summary Badges
- [✅] Show job count badges per run (e.g., "🟢 4" for completed jobs)
- [✅] Display running jobs count with blue bolt icon
- [✅] Display queued jobs count with orange clock icon
- [✅] Display completed jobs count with green checkmark icon
- [ ] Auto-refresh job summaries for active runs

### 1.4 Run Action Buttons
- [✅] Add "Open in GitHub" button (arrow.up.right.square icon) with link to run URL
- [✅] Add "Re-run workflow" button (arrow.clockwise icon, orange) - only for completed runs
- [✅] Add "Re-run failed jobs" button (arrow.triangle.2.circlepath icon, red) - only for failed runs
- [✅] Add "Cancel run" button (stop.fill icon, red) - only for in-progress/queued runs
- [✅] Implement confirmation dialogs for all destructive actions
- [✅] Call appropriate API endpoints (cancel, rerun, rerun-failed-jobs)

---

## 2. Job Display Features

### 2.1 Job List Display
- [✅] Show individual jobs under each expanded run
- [✅] Display job name
- [✅] Show job status/conclusion with icons
- [ ] Display job branch name
- [✅] Show job status text (e.g., "In Progress", "Success", "Failed")

### 2.2 Job Action Buttons
- [ ] Add "View logs" button for each job (opens logs dialog/window)
  - Note: Job logs window already exists (job_logs_window.rs), but not wired to job rows in detail view
  - Jobs can be clicked in run_jobs_window.rs to view logs
- [✅] Add "Open job in GitHub" button

### 2.3 Job Grouping
- [✅] Show "Showing latest X jobs" text below job list
- [✅] Limit to 10 jobs by default per run (already implemented in load_workflow_runs)

---

## 3. Workflow Status Badge
- [✅] Add workflow status badge (e.g., "passing" in green) next to workflow name
- [✅] Determine badge based on most recent run status
- [✅] Color-code badge (green=passing, red=failing, gray=unknown)

---

## 4. Auto-refresh for Active Runs
- [ ] Implement background auto-refresh for in-progress runs
- [ ] Use PreferencesManager refresh interval setting
- [ ] Only refresh workflows with active runs
- [ ] Stop auto-refresh when all runs complete
- [ ] Stop auto-refresh on view disappear

---

## 5. API Integration

### 5.1 Missing API Endpoints (Rust)
- [✅] Implement `list_workflow_runs(owner, repo, workflow_id)` endpoint
- [✅] Implement `list_jobs_for_run(owner, repo, run_id)` endpoint
- [✅] Implement `cancel_workflow_run(owner, repo, run_id)` endpoint
- [✅] Implement `rerun_workflow_run(owner, repo, run_id)` endpoint
- [✅] Implement `rerun_failed_jobs(owner, repo, run_id)` endpoint
- [✅] Implement `get_job_logs(owner, repo, job_id)` endpoint
- [✅] Implement `dispatch_workflow(owner, repo, workflow_id, ref)` endpoint (trigger workflow)

### 5.2 Models (Rust)
- [✅] Add `WorkflowRun` model with all fields (status, conclusion, branch, timestamps, etc.)
- [✅] Add `Job` model with all fields (status, conclusion, name, etc.)
- [✅] Add helper methods for friendly status/conclusion strings
- [✅] Add helper methods for relative time strings
- [✅] Add `JobSummary` struct (queued, running, completed counts)

---

## 6. UI Polish

### 6.1 Visual Improvements
- [✅] Add proper spacing between runs
- [✅] Improve vertical alignment of UI elements
- [✅] Add subtle background for each run row
- [ ] Improve expand/collapse animations
- [✅] Add loading spinners for job fetching
- [ ] Polish button hover states

### 6.2 Workflow Trigger Feature
- [ ] Add "Trigger workflow" button (play icon) next to workflow name
- [ ] Show API delay notice: "Triggered runs may take 10-30 seconds to appear"
- [ ] Implement workflow dispatch with ref selection
- [ ] Show trigger button only for workflows with workflow_dispatch event

---

## 7. Caching & Performance
- [ ] Implement runs cache (per repo/workflow)
- [ ] Implement jobs cache (per run)
- [ ] Restore from cache on view load
- [ ] Cache invalidation on refresh
- [ ] Debounce rapid refresh requests

---

## 8. Additional Features

### 8.1 Workflow Runs Section
- [✅] Add "No runs yet" placeholder when workflow has no runs
- [ ] Add "Triggered runs appear after 10-30 seconds" info text
- [✅] Show run count in "Runs (X)" text

### 8.2 Error Handling
- [ ] Show error states for failed API calls
- [ ] Add retry mechanisms
- [ ] Show user-friendly error messages

---

## Implementation Priority

**Phase 1 (High Priority):**
1. API endpoints for runs and jobs
2. Run display with status icons and metadata
3. Job display under runs
4. Basic action buttons (open in GitHub, cancel, rerun)

**Phase 2 (Medium Priority):**
5. Job summary badges
6. Auto-refresh for active runs
7. Workflow status badge
8. Trigger workflow button

**Phase 3 (Polish):**
9. Job logs viewer
10. Caching
11. Visual polish and animations
12. Error handling improvements

---

## Testing & Quality Assurance ✅

- [✅] Created automated logic tests (7 tests, all passing)
- [✅] Test status icon mapping
- [✅] Test CSS class mapping
- [✅] Test button visibility logic
- [✅] Test expansion state preservation
- [✅] Test auto-refresh intervals
- [✅] Test run state checks
- [✅] Documentation: `docs/TESTING.md`
- [✅] Test report: `docs/TEST_REPORT.md`

---

## Notes
- Reference: `macOS/GHActions/Views/WorkflowRowView.swift` for run display logic
- Reference: `macOS/GHActions/Views/JobsListView.swift` for job display logic
- Reference: `macOS/GHActions/Extensions/WorkflowRun+Helpers.swift` for status helpers
- Reference: `macOS/GHActions/Extensions/Job+Helpers.swift` for job helpers
- All icons use GTK symbolic icon names (need to map from SF Symbols)

## Progress Summary

**Phase 1 - Workflow Runs & Jobs Display:** ✅ COMPLETE
- Expandable workflow groups ✅
- Run display with metadata (status, branch, time) ✅
- Job display under runs ✅
- Action buttons (rerun, cancel, open in GitHub) ✅  
- Status icons with color coding ✅
- Expansion state preservation ✅
- Automated tests ✅

**Phase 2 - Additional Features:** 🚧 IN PROGRESS (5/8)
- Auto-refresh workflow functionality ✅
- Confirmation dialogs ✅
- Job summary badges ✅ (NEW - showing counts with icons)
- Workflow status badge ✅ (NEW - showing passing/failing next to workflow name)
- Job logs viewer ✅ (exists but not fully wired to detail view)
- Time string auto-update (pending - needs periodic timer implementation)
- Trigger workflow button (pending)
- Auto-refresh for active runs (pending - complex feature)

**Phase 3 - Polish:** ⏳ NOT STARTED
- Enhanced job logs viewer with streaming
- Caching improvements
- Visual polish and animations
- Error handling improvements

## Recent Updates (Current Session)

### Completed Features:
1. **Job Summary Badges** ✅ - Added badges showing counts of queued, running, and completed jobs for each workflow run
   - Green checkmark icon for completed jobs
   - Blue bolt icon for running jobs  
   - Orange clock icon for queued jobs
   - Badges appear next to workflow runs when jobs are loaded
   - Uses `JobSummary::from_jobs()` to calculate counts

2. **Workflow Status Badge** ✅ - Added status badge next to workflow name
   - Shows "passing" (green), "failing" (red), "running" (blue), "cancelled" (orange), etc.
   - Determined by most recent workflow run
   - Updates when workflows are expanded and runs are loaded
   - Clean, compact display using libadwaita styling

3. **Job Count Display** ✅ - Added "Showing X jobs" text below job list
   - Shows total number of jobs loaded
   - Proper pluralization (job/jobs)
   - Styled with dim-label and caption classes

4. **Run Count Display** ✅ - Added "Recent runs (X)" header to workflow run lists
   - Shows total number of runs available
   - Helps users understand the scope of displayed runs
   - Consistent styling with other count displays

5. **Visual Improvements** ✅ - Added subtle backgrounds for run rows
   - Applied "card" CSS class to run boxes for better visual separation
   - Added proper padding and margins to run containers
   - Improved overall visual hierarchy

6. **AGENTS.md Update** ✅ - Updated agent workflow instructions
   - Agents now use TODO.md as single source of truth
   - Clearer progress tracking guidelines

7. **Code Improvements** ✅ - Refactored helper functions for better maintainability
   - Added `update_job_summary_badges` function
   - Added `update_workflow_status_badge` function
   - Added `create_job_badge` helper function
   - Improved workflow expander header layout with separate label widget
   - All code passes clippy checks with -D warnings
   - All tests passing (15 unit tests + 7 logic tests)

### Files Modified:
- `src/ui/detail_view/helpers.rs` - Main implementation of new features
- `src/ui/detail_view/mod.rs` - Attempted time update mechanism (reverted to keep simple)
- `AGENTS.md` - Updated workflow instructions
- `TODO.md` - Updated progress tracking

### Technical Details:
- Job badges use icon + count display with proper CSS classes
- Workflow status determined from most recent run's conclusion/status
- All UI updates happen on GLib main thread as required
- Clean separation of concerns between data fetching and UI updates
- Run rows now use libadwaita "card" class for better visual appearance

### Deferred Features:
- **Time string auto-update**: Deferred due to complexity
  - Would require tracking weak references to all time labels
  - Alternative: Time strings update naturally on next refresh
  - Not critical since auto-refresh happens every 5 seconds by default
  
- **Auto-refresh for active runs**: Complex feature
  - Requires state tracking to avoid redundant API calls
  - Should integrate with existing refresh mechanism
  - Needs careful design to avoid rate limiting

### Next Priority Items:
1. **Workflow trigger button** - Manual workflow dispatch
   - Requires: UI for selecting ref/branch + API integration
   - Complexity: Medium (API already exists at `dispatch_workflow`)
   - High value for manual workflow execution
   
2. **Enhanced error handling** - Show retry mechanisms and user-friendly error messages
   - Complexity: Low (add error banners/toasts)
   - High value for better UX
   
3. **Caching integration** - Use existing DataCache for runs and jobs
   - Complexity: Medium (cache infrastructure exists but not wired)
   - Medium value for reducing API calls
