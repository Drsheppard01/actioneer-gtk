mod error;
mod http;
mod repos;
mod workflows;
mod runs;
mod jobs;

pub mod client;
pub mod models;

// Re-export main types for convenience
pub use client::GitHubClient;
pub use error::GitHubError;
