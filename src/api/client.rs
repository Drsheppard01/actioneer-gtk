use super::error::GitHubError;
use super::http::ResponseHandler;
use super::{jobs, repos, runs, workflows};
use crate::api::models::*;
use anyhow::Result;
use reqwest::Client;
use std::sync::{Arc, Mutex as StdMutex};

#[derive(Clone)]
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
    response_handler: Arc<ResponseHandler>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("Actioneer-Linux/0.1.0")
            .build()?;

        let rate_limit = Arc::new(StdMutex::new(None));
        let response_handler = Arc::new(ResponseHandler::new(rate_limit));

        Ok(Self {
            client,
            token,
            response_handler,
        })
    }

    pub fn rate_limit_info(&self) -> Option<RateLimitInfo> {
        self.response_handler.get_rate_limit()
    }

    // Repository operations
    pub async fn list_repos(&self) -> Result<Vec<Repo>, GitHubError> {
        repos::list_repos(&self.client, &self.token, &self.response_handler).await
    }

    pub async fn is_actions_enabled(&self, owner: &str, repo: &str) -> Result<bool, GitHubError> {
        repos::is_actions_enabled(
            &self.client,
            &self.token,
            &self.response_handler,
            owner,
            repo,
        )
        .await
    }

    pub async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>, GitHubError> {
        repos::list_branches(
            &self.client,
            &self.token,
            &self.response_handler,
            owner,
            repo,
        )
        .await
    }

    // Workflow operations
    pub async fn list_workflows(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Workflow>, GitHubError> {
        workflows::list_workflows(
            &self.client,
            &self.token,
            &self.response_handler,
            owner,
            repo,
        )
        .await
    }

    pub async fn dispatch_workflow(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: &str,
        ref_name: &str,
        inputs: Option<serde_json::Value>,
    ) -> Result<(), GitHubError> {
        workflows::dispatch_workflow(
            &self.client,
            &self.token,
            owner,
            repo,
            workflow_id,
            ref_name,
            inputs,
        )
        .await
    }

    // Run operations
    pub async fn list_runs(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: i64,
    ) -> Result<Vec<WorkflowRun>, GitHubError> {
        runs::list_runs(
            &self.client,
            &self.token,
            &self.response_handler,
            owner,
            repo,
            workflow_id,
        )
        .await
    }

    pub async fn rerun_workflow(
        &self,
        owner: &str,
        repo: &str,
        run_id: i64,
    ) -> Result<(), GitHubError> {
        runs::rerun_workflow(&self.client, &self.token, owner, repo, run_id).await
    }

    pub async fn rerun_failed_jobs(
        &self,
        owner: &str,
        repo: &str,
        run_id: i64,
    ) -> Result<(), GitHubError> {
        runs::rerun_failed_jobs(&self.client, &self.token, owner, repo, run_id).await
    }

    pub async fn cancel_run(
        &self,
        owner: &str,
        repo: &str,
        run_id: i64,
    ) -> Result<(), GitHubError> {
        runs::cancel_run(&self.client, &self.token, owner, repo, run_id).await
    }

    // Job operations
    pub async fn list_jobs(
        &self,
        owner: &str,
        repo: &str,
        run_id: i64,
    ) -> Result<Vec<Job>, GitHubError> {
        jobs::list_jobs(
            &self.client,
            &self.token,
            &self.response_handler,
            owner,
            repo,
            run_id,
        )
        .await
    }

    pub async fn get_job_logs(
        &self,
        owner: &str,
        repo: &str,
        job_id: i64,
    ) -> Result<String, GitHubError> {
        jobs::get_job_logs(
            &self.client,
            &self.token,
            &self.response_handler,
            owner,
            repo,
            job_id,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new(Some("test_token".to_string()));
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_without_token() {
        let client = GitHubClient::new(None);
        assert!(client.is_ok());
    }
}
