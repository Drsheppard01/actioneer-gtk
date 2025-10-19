/// Workflow run operations
use super::error::GitHubError;
use super::http::{add_auth_header, ResponseHandler, GITHUB_API_BASE};
use crate::api::models::{WorkflowRun, WorkflowRunsResponse};
use reqwest::Client;
use tracing::info;

/// List workflow runs
pub async fn list_runs(
    client: &Client,
    token: &Option<String>,
    response_handler: &ResponseHandler,
    owner: &str,
    repo: &str,
    workflow_id: i64,
) -> Result<Vec<WorkflowRun>, GitHubError> {
    info!("Fetching runs for workflow {}", workflow_id);

    let cache_key = format!(
        "GET /repos/{}/{}/actions/workflows/{}/runs?per_page=50",
        owner, repo, workflow_id
    );
    let request = client
        .get(format!(
            "{}/repos/{}/{}/actions/workflows/{}/runs",
            GITHUB_API_BASE, owner, repo, workflow_id
        ))
        .query(&[("per_page", "50")]);

    let request = add_auth_header(request, token);
    let request = response_handler.apply_cache_headers(request, Some(&cache_key));
    let response = request.send().await?;
    let runs_response: WorkflowRunsResponse = response_handler
        .handle_response(response, Some(&cache_key))
        .await?;
    Ok(runs_response.workflow_runs)
}

/// Rerun a workflow
pub async fn rerun_workflow(
    client: &Client,
    token: &Option<String>,
    owner: &str,
    repo: &str,
    run_id: i64,
) -> Result<(), GitHubError> {
    info!("Rerunning workflow run {}", run_id);

    let request = client.post(format!(
        "{}/repos/{}/{}/actions/runs/{}/rerun",
        GITHUB_API_BASE, owner, repo, run_id
    ));

    let request = add_auth_header(request, token);
    let response = request.send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(GitHubError::ApiError(format!(
            "Failed to rerun workflow: {}",
            response.status()
        )))
    }
}

/// Rerun failed jobs in a workflow run
pub async fn rerun_failed_jobs(
    client: &Client,
    token: &Option<String>,
    owner: &str,
    repo: &str,
    run_id: i64,
) -> Result<(), GitHubError> {
    info!("Rerunning failed jobs for run {}", run_id);

    let request = client.post(format!(
        "{}/repos/{}/{}/actions/runs/{}/rerun-failed-jobs",
        GITHUB_API_BASE, owner, repo, run_id
    ));

    let request = add_auth_header(request, token);
    let response = request.send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(GitHubError::ApiError(format!(
            "Failed to rerun failed jobs: {}",
            response.status()
        )))
    }
}

/// Cancel a workflow run
pub async fn cancel_run(
    client: &Client,
    token: &Option<String>,
    owner: &str,
    repo: &str,
    run_id: i64,
) -> Result<(), GitHubError> {
    use reqwest::StatusCode;

    info!("Cancelling run {}", run_id);

    let request = client.post(format!(
        "{}/repos/{}/{}/actions/runs/{}/cancel",
        GITHUB_API_BASE, owner, repo, run_id
    ));

    let request = add_auth_header(request, token);
    let response = request.send().await?;

    if response.status() == StatusCode::ACCEPTED {
        info!("Run cancelled successfully");
        Ok(())
    } else {
        Err(GitHubError::ApiError(format!(
            "Failed to cancel run: {}",
            response.status()
        )))
    }
}
