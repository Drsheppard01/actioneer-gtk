/// Repository actions state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepoActionsState {
    Enabled,
    Disabled,
    Unknown,
}

/// Workflow status counts for a repository
#[derive(Clone, Debug, Default)]
pub struct WorkflowStatusCounts {
    pub active: usize,
    pub failed: usize,
}
