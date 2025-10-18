/// Background task: Check repository status (Actions enabled, workflow counts)
use super::super::state::{RepoActionsState, WorkflowStatusCounts};
use crate::api::models::Repo;
use crate::api::GitHubClient;
use crate::ui::sidebar::gather_workflow_status_counts;
use gtk4::glib;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

/// Spawn tasks to check repository status for all repos
/// Runs checks in parallel (5 concurrent) on tokio runtime
#[allow(dead_code)] // Will be used when fully integrated
pub fn spawn_repo_status_tasks<F>(
    repos: Vec<Repo>,
    client: GitHubClient,
    actions_state: Arc<Mutex<HashMap<i64, RepoActionsState>>>,
    workflow_state: Arc<Mutex<HashMap<i64, WorkflowStatusCounts>>>,
    on_complete: F,
) where
    F: FnOnce() + Send + 'static,
{
    // Run all status checks in parallel on tokio runtime
    let handle = crate::runtime_handle().spawn(async move {
        use futures::stream::{self, StreamExt};

        stream::iter(repos)
            .for_each_concurrent(5, |repo| {
                let client = client.clone();
                let actions_state = actions_state.clone();
                let workflow_state = workflow_state.clone();

                async move {
                    let owner = repo.owner.login.clone();
                    let repo_name = repo.name.clone();
                    let repo_id = repo.id;

                    // Check actions enabled
                    match client.is_actions_enabled(&owner, &repo_name).await {
                        Ok(enabled) => {
                            let mut actions = actions_state.lock();
                            actions.insert(
                                repo_id,
                                if enabled {
                                    RepoActionsState::Enabled
                                } else {
                                    RepoActionsState::Disabled
                                },
                            );
                        }
                        Err(err) => {
                            warn!(
                                "Failed to fetch actions status for {}/{}: {}",
                                owner, repo_name, err
                            );
                        }
                    }

                    // Get workflow counts
                    match gather_workflow_status_counts(&client, &owner, &repo_name).await {
                        Ok(counts) => {
                            let mut workflows = workflow_state.lock();
                            workflows.insert(repo_id, counts);
                        }
                        Err(err) => {
                            warn!(
                                "Failed to fetch workflow status for {}/{}: {}",
                                owner, repo_name, err
                            );
                        }
                    }
                }
            })
            .await;
    });

    // Trigger callback once complete
    glib::MainContext::default().spawn_local(async move {
        let _ = handle.await;
        on_complete();
    });
}
