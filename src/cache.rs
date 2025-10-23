use crate::api::models::{Job, Workflow, WorkflowRun};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct WorkflowCache {
    runs: Vec<WorkflowRun>,
    jobs: HashMap<i64, Vec<Job>>, // run_id -> jobs
}

#[derive(Debug, Clone)]
struct RepoCacheEntry {
    workflows: Vec<Workflow>,
    workflow_data: HashMap<i64, WorkflowCache>, // workflow_id -> cache
}

/// In-memory data cache to reduce API calls
/// Similar to DataCache.swift in macOS version
#[derive(Debug, Clone)]
pub struct DataCache {
    repositories: Arc<RwLock<HashMap<String, RepoCacheEntry>>>,
}

impl DataCache {
    pub fn new() -> Self {
        Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached workflows for a repository
    pub async fn workflows(&self, key: &str) -> Option<Vec<Workflow>> {
        let repos = self.repositories.read().await;
        repos.get(key).map(|entry| entry.workflows.clone())
    }

    /// Store workflows for a repository
    pub async fn store_workflows(&self, workflows: Vec<Workflow>, key: &str) {
        let mut repos = self.repositories.write().await;
        let entry = repos.entry(key.to_string()).or_insert(RepoCacheEntry {
            workflows: Vec::new(),
            workflow_data: HashMap::new(),
        });
        entry.workflows = workflows;
    }

    /// Get cached runs for a workflow
    pub async fn runs(&self, key: &str, workflow_id: i64) -> Option<Vec<WorkflowRun>> {
        let repos = self.repositories.read().await;
        repos
            .get(key)
            .and_then(|entry| entry.workflow_data.get(&workflow_id))
            .map(|cache| cache.runs.clone())
    }

    /// Store runs for a workflow
    pub async fn store_runs(&self, runs: Vec<WorkflowRun>, key: &str, workflow_id: i64) {
        let mut repos = self.repositories.write().await;
        let entry = repos.entry(key.to_string()).or_insert(RepoCacheEntry {
            workflows: Vec::new(),
            workflow_data: HashMap::new(),
        });

        let workflow_cache = entry
            .workflow_data
            .entry(workflow_id)
            .or_insert(WorkflowCache {
                runs: Vec::new(),
                jobs: HashMap::new(),
            });

        // Update runs and clean up stale job data
        let valid_run_ids: std::collections::HashSet<_> = runs.iter().map(|r| r.id).collect();
        workflow_cache.runs = runs;
        workflow_cache
            .jobs
            .retain(|run_id, _| valid_run_ids.contains(run_id));
    }

    /// Get cached jobs for a run
    pub async fn jobs(&self, key: &str, workflow_id: i64, run_id: i64) -> Option<Vec<Job>> {
        let repos = self.repositories.read().await;
        repos
            .get(key)
            .and_then(|entry| entry.workflow_data.get(&workflow_id))
            .and_then(|cache| cache.jobs.get(&run_id))
            .cloned()
    }

    /// Store jobs for a run
    pub async fn store_jobs(&self, jobs: Vec<Job>, key: &str, workflow_id: i64, run_id: i64) {
        let mut repos = self.repositories.write().await;
        let entry = repos.entry(key.to_string()).or_insert(RepoCacheEntry {
            workflows: Vec::new(),
            workflow_data: HashMap::new(),
        });

        let workflow_cache = entry
            .workflow_data
            .entry(workflow_id)
            .or_insert(WorkflowCache {
                runs: Vec::new(),
                jobs: HashMap::new(),
            });

        workflow_cache.jobs.insert(run_id, jobs);
    }

    /// Clear cache for a specific repository
    pub async fn clear_repo(&self, key: &str) {
        let mut repos = self.repositories.write().await;
        repos.remove(key);
    }

    /// Clear all cached data
    pub async fn clear_all(&self) {
        let mut repos = self.repositories.write().await;
        repos.clear();
    }
}

impl Default for DataCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_caching() {
        let cache = DataCache::new();
        let key = "owner/repo";

        // Initially empty
        assert!(cache.workflows(key).await.is_none());

        // Store and retrieve
        let workflows = vec![];
        cache.store_workflows(workflows.clone(), key).await;
        assert_eq!(cache.workflows(key).await, Some(workflows));
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let cache = DataCache::new();
        let key = "owner/repo";

        cache.store_workflows(vec![], key).await;
        assert!(cache.workflows(key).await.is_some());

        cache.clear_repo(key).await;
        assert!(cache.workflows(key).await.is_none());
    }
}
