/// State types for the main window
use crate::api::models::Repo;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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

/// Shared state for repository list
#[derive(Clone)]
#[allow(dead_code)] // Will be used when refactoring is complete
pub struct RepoListState {
    pub repos: Arc<Mutex<Vec<Repo>>>,
    pub favorites: Arc<Mutex<HashSet<i64>>>,
    pub actions_states: Arc<Mutex<HashMap<i64, RepoActionsState>>>,
    pub workflow_counts: Arc<Mutex<HashMap<i64, WorkflowStatusCounts>>>,
    pub selected_repo_id: Arc<Mutex<Option<i64>>>,
}

impl RepoListState {
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn new() -> Self {
        Self {
            repos: Arc::new(Mutex::new(Vec::new())),
            favorites: Arc::new(Mutex::new(HashSet::new())),
            actions_states: Arc::new(Mutex::new(HashMap::new())),
            workflow_counts: Arc::new(Mutex::new(HashMap::new())),
            selected_repo_id: Arc::new(Mutex::new(None)),
        }
    }

    /// Get a snapshot of current repositories
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn get_repos_snapshot(&self) -> Vec<Repo> {
        self.repos.lock().clone()
    }

    /// Get a snapshot of favorites
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn get_favorites_snapshot(&self) -> HashSet<i64> {
        self.favorites.lock().clone()
    }

    /// Get a snapshot of actions states
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn get_actions_snapshot(&self) -> HashMap<i64, RepoActionsState> {
        self.actions_states.lock().clone()
    }

    /// Get a snapshot of workflow counts
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn get_workflow_counts_snapshot(&self) -> HashMap<i64, WorkflowStatusCounts> {
        self.workflow_counts.lock().clone()
    }

    /// Get selected repository ID
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn get_selected_id(&self) -> Option<i64> {
        *self.selected_repo_id.lock()
    }

    /// Set selected repository ID
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn set_selected_id(&self, id: Option<i64>) {
        *self.selected_repo_id.lock() = id;
    }

    /// Update repositories
    #[allow(dead_code)] // Will be used when refactoring is complete
    pub fn set_repos(&self, repos: Vec<Repo>) {
        *self.repos.lock() = repos;
    }
}

impl Default for RepoListState {
    fn default() -> Self {
        Self::new()
    }
}
