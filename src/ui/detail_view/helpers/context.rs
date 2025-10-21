use crate::api::GitHubClient;
use crate::cache::DataCache;
use gtk4::{self as gtk};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Shared state for refreshing job lists when background tasks update.
#[derive(Clone)]
pub(crate) struct JobRefreshContext {
    client: Arc<Mutex<GitHubClient>>,
    owner: String,
    repo: String,
    workflow_id: i64,
    run_id: i64,
    cache: Arc<DataCache>,
    jobs_box: gtk::Box,
    badges_box: Option<gtk::Box>,
}

impl JobRefreshContext {
    pub(crate) fn new(
        client: Arc<Mutex<GitHubClient>>,
        owner: String,
        repo: String,
        workflow_id: i64,
        run_id: i64,
        cache: Arc<DataCache>,
        jobs_box: gtk::Box,
        badges_box: Option<gtk::Box>,
    ) -> Self {
        Self {
            client,
            owner,
            repo,
            workflow_id,
            run_id,
            cache,
            jobs_box,
            badges_box,
        }
    }

    pub(crate) fn workflow_id(&self) -> i64 {
        self.workflow_id
    }

    pub(crate) fn run_id(&self) -> i64 {
        self.run_id
    }

    pub(crate) fn client(&self) -> Arc<Mutex<GitHubClient>> {
        self.client.clone()
    }

    pub(crate) fn owner(&self) -> String {
        self.owner.clone()
    }

    pub(crate) fn repo(&self) -> String {
        self.repo.clone()
    }

    pub(crate) fn cache(&self) -> Arc<DataCache> {
        self.cache.clone()
    }

    pub(crate) fn jobs_box(&self) -> gtk::Box {
        self.jobs_box.clone()
    }

    pub(crate) fn badges_box(&self) -> Option<gtk::Box> {
        self.badges_box.clone()
    }
}

pub(crate) type JobContextMap = Arc<Mutex<HashMap<i64, JobRefreshContext>>>;

pub(crate) fn take_job_context_run_ids(job_contexts: &JobContextMap, workflow_id: i64) -> Vec<i64> {
    let mut guard = job_contexts.lock();
    let run_ids: Vec<i64> = guard
        .iter()
        .filter_map(|(&run_id, ctx)| {
            if ctx.workflow_id() == workflow_id {
                Some(run_id)
            } else {
                None
            }
        })
        .collect();

    for run_id in &run_ids {
        guard.remove(run_id);
    }

    run_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::DataCache;

    #[test]
    fn take_job_context_run_ids_removes_entries_for_workflow() {
        gtk::init().ok();

        let client = Arc::new(Mutex::new(GitHubClient::new(None).unwrap()));
        let cache = Arc::new(DataCache::new());
        let jobs_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let context_one = JobRefreshContext::new(
            client.clone(),
            "owner".to_string(),
            "repo".to_string(),
            42,
            1,
            cache.clone(),
            jobs_box.clone(),
            None,
        );
        let context_two = JobRefreshContext::new(
            client.clone(),
            "owner".to_string(),
            "repo".to_string(),
            42,
            2,
            cache.clone(),
            jobs_box.clone(),
            None,
        );
        let context_other = JobRefreshContext::new(
            client.clone(),
            "owner".to_string(),
            "repo".to_string(),
            7,
            99,
            cache.clone(),
            jobs_box.clone(),
            None,
        );

        let job_contexts: JobContextMap = Arc::new(Mutex::new(HashMap::new()));
        {
            let mut guard = job_contexts.lock();
            guard.insert(1, context_one);
            guard.insert(2, context_two);
            guard.insert(99, context_other);
        }

        let removed_ids = take_job_context_run_ids(&job_contexts, 42);
        assert_eq!(removed_ids.len(), 2);
        assert!(removed_ids.contains(&1));
        assert!(removed_ids.contains(&2));

        let guard = job_contexts.lock();
        assert!(!guard.contains_key(&1));
        assert!(!guard.contains_key(&2));
        assert!(guard.contains_key(&99));
    }
}
