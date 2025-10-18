/// Job operations
use super::error::GitHubError;
use super::http::{add_auth_header, ResponseHandler, GITHUB_API_BASE};
use crate::api::models::{Job, JobsResponse};
use reqwest::Client;
use tracing::info;

/// List jobs for a workflow run
pub async fn list_jobs(
    client: &Client,
    token: &Option<String>,
    response_handler: &ResponseHandler,
    owner: &str,
    repo: &str,
    run_id: i64,
) -> Result<Vec<Job>, GitHubError> {
    info!("Fetching jobs for run {}", run_id);

    let request = client.get(format!(
        "{}/repos/{}/{}/actions/runs/{}/jobs",
        GITHUB_API_BASE, owner, repo, run_id
    ));

    let request = add_auth_header(request, token);
    let response = request.send().await?;
    let jobs_response: JobsResponse = response_handler.handle_response(response).await?;
    Ok(jobs_response.jobs)
}

/// Get logs for a job
pub async fn get_job_logs(
    client: &Client,
    token: &Option<String>,
    owner: &str,
    repo: &str,
    job_id: i64,
) -> Result<String, GitHubError> {
    info!("Fetching logs for job {}", job_id);

    let request = client.get(format!(
        "{}/repos/{}/{}/actions/jobs/{}/logs",
        GITHUB_API_BASE, owner, repo, job_id
    ));

    let request = add_auth_header(request, token);
    let response = request.send().await?;

    if response.status().is_success() {
        let logs = response.text().await?;
        Ok(logs)
    } else {
        Err(GitHubError::ApiError(format!(
            "Failed to fetch logs: {}",
            response.status()
        )))
    }
}
