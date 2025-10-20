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
- [✅] Improve expand/collapse animations (handled by GTK4 automatically)
- [✅] Add loading spinners for job fetching
- [✅] Polish button hover states (handled by libadwaita flat+circular classes)

### 6.2 Workflow Trigger Feature
- [✅] Add "Trigger workflow" button (play icon) next to workflow name
- [✅] Show API delay notice: "Triggered runs may take 10-30 seconds to appear"
- [✅] Implement workflow dispatch with ref selection
- [✅] Show trigger button for all workflows (users can trigger any workflow)

---

## 7. Caching & Performance
- [ ] Implement runs cache (per repo/workflow)
- [ ] Implement jobs cache (per run)
- [ ] Restore from cache on view load
- [ ] Cache invalidation on refresh
- [✅] Debounce rapid refresh requests (already implemented via loading guard)

---

## 8. Additional Features

### 8.1 Workflow Runs Section
- [✅] Add "No runs yet" placeholder when workflow has no runs
- [✅] Add "Triggered runs appear after 10-30 seconds" info text
- [✅] Show run count in "Runs (X)" text

### 8.2 Error Handling
- [✅] Show error states for failed API calls
- [✅] Add retry mechanisms
- [✅] Show user-friendly error messages

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

## Recent Updates (Current Session - Continued)

### Session 3: Layout Fix for Badges and Buttons + Major Feature Additions

**Completed Features:**

1. **Run Row Layout Fix** ✅ - Fixed badge and button positioning
   - Job summary badges now appear immediately after run title
   - Action buttons (open, rerun, cancel) appear right after badges
   - Previously badges and buttons were pushed to far right due to expander hexpand
   - Fixed by including badges and buttons in the expander's label widget
   - Much better visual layout matching the macOS app

2. **Vertical Alignment Refinement** ✅ - Improved badge/button alignment
   - Changed badges and buttons from center to start alignment
   - Added small top margin (1-2px) to align with title text baseline
   - Badges and buttons now align properly with the run title text
   - Prevents badges from appearing too low when subtitle is present

3. **Action Buttons Right-Aligned** ✅ - Moved buttons to right edge of card
   - Buttons (Open in GitHub, Re-run) now positioned at the right edge
   - Badges stay with the run title on the left
   - Buttons aligned to top (Start) to perfectly match title baseline
   - Removed extra margin to align naturally with expander label
   - Better visual separation between content and actions
   - Matches macOS app layout

4. **Auth Window Spinner Improvement** ✅ - Better spinner placement
   - Moved spinner next to "Waiting for authorization..." text
   - Previously spinner was awkwardly placed between buttons
   - Created horizontal status_box containing label and spinner
   - Cleaner, more professional appearance

5. **Job Row Layout Improvement** ✅ - Better alignment for job metadata
   - Grouped status, duration, and button into right_box container
   - All metadata elements now aligned to right edge of job row
   - Added `hexpand(false)` to right_box to prevent expansion
   - Added `hexpand(true)` to jobs_box and job_box to take full width
   - Added `margin_end(12)` to jobs_box for balanced spacing
   - Added ellipsize to job name to prevent overflow
   - Vertically centered within the row for consistent appearance
   - Clean separation between job name (left) and metadata (right)
   - **Verified with unit test** confirming proper alignment properties

6. **Copy to Clipboard for Auth Code** ✅ - Improved authentication UX
   - Added copy button next to the device auth code
   - Button shows visual feedback (checkmark) for 2 seconds after copying
   - Code remains selectable for manual copying if preferred
   - Cleaner, more user-friendly authentication experience

7. **Sign Out Confirmation & Better UX** ✅ - Improved sign out flow
   - Added confirmation dialog before signing out
   - Dialog explains user will need to sign in again
   - After sign out, automatically shows auth window
   - No need to restart application - seamless re-authentication
   - Much better user experience than previous "restart required" message

8. **Workflow Trigger Button** ✅ - Manual workflow dispatch
   - Added play button next to each workflow name
   - Opens dialog to select branch/ref (defaults to "main")
   - **Fetches and displays all repository branches in dropdown**
   - Shows loading spinner while fetching branches
   - Graceful fallback to "main" if branch fetch fails
   - Shows helpful notice: "Triggered runs may take 10-30 seconds to appear"
   - Integrated with existing `dispatch_workflow` API
   - Error handling with user-friendly error dialogs
   - High-value feature for CI/CD management
   - **Much better UX than typing branch names!**

9. **Welcome Screen** 🔄 - Sign-in state UI (IN PROGRESS)
   - Created welcome_screen.rs with beautiful sign-in UI
   - Matches macOS app design with icon, title, features list
   - "Sign in with GitHub" button (blue, prominent)
   - "Try Demo Mode" button (disabled as requested)
   - Ready to integrate into main window with stack switcher
   - TODO: Wire up authentication flow, switch between welcome/main views

### Known Issues to Fix:
- [ ] Background auto-refresh with ETag (triggered workflows don't appear without manual refresh)
- [ ] Preserve workflow expansion state on repository refresh
- [ ] Integrate welcome screen into main window (show on startup if not authenticated)

### Files Modified (Session 3):
- `src/ui/detail_view/helpers.rs` - Fixed run row layout, button positioning, job row alignment and width, added workflow trigger with branch selector
- `src/ui/auth_window.rs` - Improved spinner placement and added copy-to-clipboard button
- `src/ui/main_window.rs` - Added sign out confirmation and seamless re-authentication
- `src/api/models.rs` - Added Branch and BranchCommit models
- `src/api/repos.rs` - Added list_branches function
- `src/api/client.rs` - Added list_branches method
- `TODO.md` - Updated progress tracking

### Technical Details:
- Badges stay in label_box (left side with title)
- Buttons moved to row_container with `set_halign(gtk::Align::End)` and `set_valign(gtk::Align::Start)`
- Buttons have no extra margin - natural alignment with expander label
- Both expander and buttons_box are children of row_container with Start alignment
- Job row: job_box and jobs_box have `hexpand(true)` to fill available width
- Job row: job_name has `hexpand(true)`, right_box has `hexpand(false)` and `halign(End)`
- Job row metadata (status, duration, button) grouped in right_box with `halign(End)` and `valign(Center)`
- Added unit test `test_job_row_layout_properties` to verify alignment
- Branch selector uses glib::MainContext channel for async data fetching
- ComboBoxText provides native dropdown UI with loading spinner
- Branch fetching gracefully falls back to "main" if API fails
- Spinner now in horizontal box with status label for better alignment
- Changed `set_valign(gtk::Align::Center)` to `set_valign(gtk::Align::Start)` for badges
- Added `margin_top(1-2px)` for fine-tuned baseline alignment on badges

### User-Reported Issues Fixed:
✅ Badge and button positioning (far right → immediately after run title)
✅ Vertical alignment of badges and buttons (centered → aligned with title)
✅ Action buttons moved to right edge (as requested in image 6.png)
✅ Auth dialog spinner placement (between buttons → next to status text)
✅ Buttons vertical alignment perfected (removed extra margin for natural alignment)
✅ Job row metadata alignment (scattered → grouped and right-aligned per image 8.png)
✅ Job rows not taking full width (added margin_end and verified hexpand)
✅ Auth code needs copy button (added with visual feedback per image 3.png)
✅ Sign out needs confirmation (added dialog with Yes/No)
✅ Sign out requires restart (fixed - now seamlessly shows auth window)

---

## Previous Session Summary

### Session 2: UI Fixes and Error Handling Improvements

**Completed Features:**

1. **Job Display Alignment Fix** ✅ - Fixed icon alignment in job rows
   - Icons now align to top instead of center for better visual appearance
   - Applied to run_jobs_window.rs job rows
   - Improved readability when job names wrap to multiple lines

2. **Job Sublabel Improvements** ✅ - Made job details more meaningful
   - Changed from "Status: completed • Conclusion: success" to "Success • 2m 15s"
   - Shows friendly status (Success, Failed, In Progress, etc.)
   - Displays duration when available
   - More concise and user-friendly information

3. **Enhanced Error Handling** ✅ - Added retry mechanisms with user-friendly messages
   - Error messages now show "Unable to load..." instead of generic "Failed"
   - Displays actual error details in smaller caption text
   - Added "Retry" button with suggested-action styling
   - Retry buttons reload the data when clicked
   - Implemented for both workflow runs and jobs loading

4. **Info Text for Empty Runs** ✅ - Added helpful message about workflow dispatch
   - Shows "Triggered runs may take 10-30 seconds to appear" when no runs exist
   - Helps users understand why manually triggered workflows might not appear immediately
   - Better user experience for workflow_dispatch scenarios

5. **Visual Polish Completed** ✅ - Marked remaining visual improvements
   - Expand/collapse animations: Handled automatically by GTK4 expanders
   - Button hover states: Handled by libadwaita flat+circular classes
   - All Phase 1 visual improvements now complete

### Files Modified (Session 2):
- `src/ui/run_jobs_window.rs` - Fixed job icon alignment and sublabel text
- `src/ui/detail_view/helpers.rs` - Enhanced error handling with retry buttons, added info text
- `TODO.md` - Updated progress tracking

### Technical Details:
- Job icons use `set_valign(gtk::Align::Start)` for top alignment
- Error boxes use vertical layout with properly styled labels and buttons
- Retry buttons clone necessary Arc/String values to avoid move errors
- All error states now provide actionable recovery options
- Friendly status uses existing helper methods from Job impl

### Code Quality:
- All 23 tests passing (16 unit tests + 7 logic tests)
- Added new unit test for job row layout verification
- Zero clippy warnings with `-D warnings`
- Code properly formatted with `cargo fmt`
- Test verifies: job name expands, right_box doesn't expand, proper alignment

### Progress Update:
**Phase 1 - Workflow Runs & Jobs Display:** ✅ COMPLETE
**Phase 2 - Additional Features:** 🚧 IN PROGRESS (6/8 items)
- Job summary badges ✅
- Workflow status badge ✅  
- Enhanced error handling ✅ (NEW)
- Visual improvements ✅ (NEW)
- Info text for users ✅ (NEW)
- Time string auto-update (deferred)
- Auto-refresh for active runs (pending)
- Trigger workflow button (pending)

**Phase 3 - Polish:** 🚧 IN PROGRESS (2/4 items)
- Error handling improvements ✅ (NEW)
- Visual polish ✅ (NEW)
- Caching improvements (pending)
- Enhanced job logs viewer (pending)

### User-Reported Issues Fixed:
✅ Job icon alignment (center → top)
✅ Job sublabel values (raw status → friendly status + duration)

### Next Priority Items:
1. **Workflow trigger button** - Manual workflow dispatch UI
   - Medium complexity, high value
   - API already exists (`dispatch_workflow`)
   
2. **Caching integration** - Wire up existing DataCache
   - Medium complexity, medium value
   - Reduce API calls and improve performance
   
3. **Auto-refresh for active runs** - Smart background updates
   - High complexity, high value
   - Only refresh runs that are in-progress or queued

---

## Previous Session Summary

### Session 1: Core Features Implementation

**Completed Features:**

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

### Deferred Features (Session 1):
- **Time string auto-update**: Deferred due to complexity
  - Would require tracking weak references to all time labels
  - Alternative: Time strings update naturally on next refresh
  - Not critical since auto-refresh happens every 5 seconds by default
  
- **Auto-refresh for active runs**: Complex feature
  - Requires state tracking to avoid redundant API calls
  - Should integrate with existing refresh mechanism
  - Needs careful design to avoid rate limiting
