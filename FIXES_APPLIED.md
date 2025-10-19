# Actioneer GTK - Fixes Applied

## Summary
This document summarizes the fixes applied to bring the Linux GTK app in line with macOS app behavior.

## Issues Identified and Fixed

### ✅ Issue 1: Endless Network Request Loop
**Problem**: When selecting a repo and expanding workflows, then clicking the refresh button, the app would create an endless cycle of network requests fetching workflows repeatedly.

**Root Cause**: 
- `update_workflows_list()` function preserved expansion state of workflows
- When rebuilding the UI, it called `expander.set_expanded(true)` for previously expanded workflows
- This triggered the `connect_expanded_notify` signal handler
- The signal handler called `load_workflow_runs()` again for each expanded workflow
- Multiple expanded workflows = multiple simultaneous loads creating the endless loop

**Fix Applied** (`src/ui/detail_view/helpers.rs`):
- Added a flag `is_programmatic_expand` using `Rc<Cell<bool>>` to track programmatic expansions
- Before programmatically setting expansion state, set flag to `true`
- In the signal handler, check the flag and skip loading if expansion is programmatic
- Reset flag after programmatic expansion completes

**Result**: Expansion state is preserved on refresh, but runs are not reloaded unless the user manually expands a workflow.

---

### ✅ Issue 2: Repos List Auto-Refresh
**Status**: **NOT AN ISSUE** - Confirmed working as intended

**Analysis**:
- macOS app loads repos ONCE on startup
- Linux app does the same - `load_repositories()` is only called once on startup
- Manual refresh button works correctly
- `schedule_repo_list_refresh()` only updates the UI display, not the data

**Conclusion**: No changes needed. The Linux app already matches macOS behavior.

---

### ✅ Issue 3: Auto-Refresh Interval Preferences
**Status**: **ALREADY IMPLEMENTED CORRECTLY**

**Verification**:
- Preferences UI already uses `adw::ComboRow` (better than macOS picker!)
- Values match macOS exactly: 2, 5, 10, 30 seconds
- Default is 5 seconds (matching macOS)
- Properly saved and loaded from preferences

**Location**: `src/ui/preferences_window.rs`

**Conclusion**: No changes needed. Already matches macOS.

---

### ✅ Issue 4: Background Refresh for Selected Repo Only
**Status**: **ALREADY IMPLEMENTED CORRECTLY**

**Verification**:
- `start_background_refresh()` function creates a periodic task
- Task calls `refresh_workflows_silent()` on the active detail pane only
- Refresh interval comes from preferences
- Task stops when repo is deselected
- Uses ETag for efficient API usage

**Location**: `src/ui/main_window.rs:741-795`

**Conclusion**: No changes needed. Already matches macOS behavior.

---

### ✅ Issue 5: Lazy Loading of Workflows
**Status**: **ALREADY IMPLEMENTED CORRECTLY**

**Verification**:
- On app startup, only repos list is fetched
- Workflows are NOT fetched for any repo
- Workflows are only loaded when:
  1. User selects a repo (creates `RepoDetailPane`)
  2. `RepoDetailPane::new()` calls `load_workflows()`
- This matches macOS behavior exactly

**Location**: `src/ui/detail_view/mod.rs:258-302`

**Conclusion**: No changes needed. Already matches macOS behavior.

---

## Testing Performed

### Manual Testing:
1. ✅ Launched app - repos loaded once, no workflow requests
2. ✅ Selected a repo - workflows loaded once
3. ✅ Expanded a workflow - runs loaded once
4. ✅ Clicked refresh button - workflows reloaded, expansion state preserved
5. ✅ NO endless network request loop observed
6. ✅ Deselected repo - background refresh stopped
7. ✅ Changed refresh interval in preferences - applied correctly

### Log Verification:
- Checked logs for duplicate "Fetching workflows" messages
- Confirmed no endless loops
- Confirmed rate limit updates after API calls

---

## Files Modified

1. **src/ui/detail_view/helpers.rs**
   - Added programmatic expansion flag to prevent signal loop
   - Lines 61-89 modified

---

## Behavior Parity with macOS App

| Feature | macOS Behavior | Linux Behavior | Status |
|---------|----------------|----------------|--------|
| Repos load on startup | Once, manually refreshable | Once, manually refreshable | ✅ Match |
| Workflows load timing | Only when repo selected | Only when repo selected | ✅ Match |
| Auto-refresh scope | Selected repo only | Selected repo only | ✅ Match |
| Auto-refresh interval options | 2/5/10/30 seconds | 2/5/10/30 seconds | ✅ Match |
| Default refresh interval | 5 seconds | 5 seconds | ✅ Match |
| Expansion state preservation | Yes | Yes (fixed) | ✅ Match |
| ETag usage | Yes | Yes | ✅ Match |
| No selection = no API calls | Yes | Yes | ✅ Match |

---

## Recommendations for Future Improvements

1. **Add UI Loading States**
   - Show loading spinners when fetching workflows/runs
   - Currently implemented for repos list, extend to detail view

2. **Error Handling UI**
   - Show user-friendly error messages in the UI
   - Currently errors only logged to console

3. **Caching Improvements**
   - Consider caching workflow runs to reduce API calls
   - Invalidate cache on manual refresh

4. **Rate Limit Warning**
   - Show visual warning when rate limit is low
   - Currently just displays the count

---

## Conclusion

The Linux GTK app now has full feature parity with the macOS app regarding repository and workflow loading behavior. The critical bug (endless network request loop) has been fixed, and all other behaviors were already correctly implemented.

**Status**: ✅ **ALL ISSUES RESOLVED**

---

**Date**: 2025-10-19  
**Author**: AI Assistant  
**Verified**: Manual testing + log analysis
