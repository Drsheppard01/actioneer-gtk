/// Utility functions for workflow runs
use crate::api::models::WorkflowRun;

/// Check if a workflow run is currently active
pub fn is_run_active(run: &WorkflowRun) -> bool {
    run.is_active()
}

/// Check if a workflow run has failed
pub fn is_run_failure(run: &WorkflowRun) -> bool {
    run.conclusion
        .as_deref()
        .map(|c| matches!(c, "failure" | "cancelled" | "timed_out"))
        .unwrap_or(false)
}
