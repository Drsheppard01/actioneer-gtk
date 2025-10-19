/// Job operations
use super::error::GitHubError;
use super::http::{add_auth_header, ResponseHandler, GITHUB_API_BASE};
use crate::api::models::{Job, JobsResponse};
use reqwest::Client;
use reqwest::StatusCode;
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

    let cache_key = format!("GET /repos/{}/{}/actions/runs/{}/jobs", owner, repo, run_id);
    let request = client.get(format!(
        "{}/repos/{}/{}/actions/runs/{}/jobs",
        GITHUB_API_BASE, owner, repo, run_id
    ));

    let request = add_auth_header(request, token);
    let request = response_handler.apply_cache_headers(request, Some(&cache_key));
    let response = request.send().await?;
    let jobs_response: JobsResponse = response_handler
        .handle_response(response, Some(&cache_key))
        .await?;
    Ok(jobs_response.jobs)
}

/// Get logs for a job
pub async fn get_job_logs(
    client: &Client,
    token: &Option<String>,
    response_handler: &ResponseHandler,
    owner: &str,
    repo: &str,
    job_id: i64,
) -> Result<String, GitHubError> {
    info!("Fetching logs for job {}", job_id);

    let cache_key = format!("GET /repos/{}/{}/actions/jobs/{}/logs", owner, repo, job_id);
    let request = client.get(format!(
        "{}/repos/{}/{}/actions/jobs/{}/logs",
        GITHUB_API_BASE, owner, repo, job_id
    ));

    let request = add_auth_header(request, token);
    let request = response_handler.apply_cache_headers(request, Some(&cache_key));
    let response = request.send().await?;

    let status = response.status();
    let headers = response.headers().clone();
    response_handler.update_rate_limit(&headers);

    if status == StatusCode::OK {
        let body = response.bytes().await?;
        response_handler.store_cache_entry(&cache_key, &headers, &body);
        let logs = String::from_utf8(body.to_vec()).map_err(|error| {
            GitHubError::ApiError(format!("Failed to parse job logs as UTF-8: {}", error))
        })?;
        Ok(logs)
    } else if status == StatusCode::NOT_MODIFIED {
        if let Some(cached) = response_handler.cached_body(&cache_key) {
            let logs = String::from_utf8((*cached).clone()).map_err(|error| {
                GitHubError::ApiError(format!(
                    "Failed to parse cached job logs as UTF-8: {}",
                    error
                ))
            })?;
            Ok(logs)
        } else {
            Err(GitHubError::ApiError(
                "Job logs cache miss after 304 Not Modified".to_string(),
            ))
        }
    } else if status.is_success() {
        // Accept other success statuses like 202
        let body = response.bytes().await?;
        let logs = String::from_utf8(body.to_vec()).map_err(|error| {
            GitHubError::ApiError(format!(
                "Failed to parse job logs response as UTF-8: {}",
                error
            ))
        })?;
        Ok(logs)
    } else {
        Err(GitHubError::ApiError(format!(
            "Failed to fetch logs: {}",
            status
        )))
    }
}
