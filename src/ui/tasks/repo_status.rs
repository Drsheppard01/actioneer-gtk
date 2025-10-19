/// Background task: Check repository status (Actions enabled, workflow counts)
use super::super::state::{RepoActionsState, WorkflowStatusCounts};
use crate::api::models::Repo;
use crate::api::GitHubClient;
use crate::ui::sidebar::gather_workflow_status_counts;
use crate::ui::utils::MainContextChannelExt;
use gtk4::glib;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

const MAX_REPOS_FOR_STATUS: usize = 20;

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
    let (sender, receiver) = glib::MainContext::default().channel::<()>(glib::Priority::default());
    let mut callback = Some(on_complete);

    receiver.attach(None, move |_| {
        if let Some(done) = callback.take() {
            done();
        }
        glib::ControlFlow::Break
    });

    let repos: Vec<Repo> = repos.into_iter().take(MAX_REPOS_FOR_STATUS).collect();

    crate::runtime_handle().spawn(async move {
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

        let _ = sender.send(());
    });
}
