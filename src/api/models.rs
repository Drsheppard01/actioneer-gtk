// Allow unused code for models that will be used when more UI is implemented
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    #[serde(rename = "private")]
    pub is_private: bool,
    pub permissions: Option<RepoPermissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoPermissions {
    pub admin: bool,
    pub push: bool,
    pub pull: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workflow {
    pub id: i64,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowsResponse {
    pub total_count: i64,
    pub workflows: Vec<Workflow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: i64,
    pub run_number: Option<i64>,
    pub name: Option<String>,
    pub display_title: Option<String>,
    pub head_branch: Option<String>,
    pub status: Option<String>,
    pub conclusion: Option<String>,
    pub run_started_at: Option<String>,
    pub event: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunsResponse {
    pub total_count: i64,
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub run_id: i64,
    pub status: Option<String>,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub name: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsResponse {
    pub total_count: i64,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: i64,
    pub remaining: i64,
    pub reset: i64, // Unix timestamp
}

impl RateLimitInfo {
    pub fn is_low(&self) -> bool {
        self.remaining < 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_deserialization() {
        let json = r#"{
            "id": 123,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {
                "login": "owner"
            },
            "private": false,
            "permissions": {
                "admin": true,
                "push": true,
                "pull": true
            }
        }"#;

        let repo: Repo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.id, 123);
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.owner.login, "owner");
    }

    #[test]
    fn test_workflow_deserialization() {
        let json = r#"{
            "id": 456,
            "name": "CI",
            "path": ".github/workflows/ci.yml"
        }"#;

        let workflow: Workflow = serde_json::from_str(json).unwrap();
        assert_eq!(workflow.id, 456);
        assert_eq!(workflow.name, "CI");
    }

    #[test]
    fn test_rate_limit_is_low() {
        let high = RateLimitInfo {
            limit: 5000,
            remaining: 4500,
            reset: 1234567890,
        };
        assert!(!high.is_low());

        let low = RateLimitInfo {
            limit: 5000,
            remaining: 50,
            reset: 1234567890,
        };
        assert!(low.is_low());
    }
}
