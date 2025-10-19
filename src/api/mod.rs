mod error;
mod http;
mod jobs;
mod repos;
mod runs;
mod workflows;

pub mod client;
pub mod models;

// Re-export main types for convenience
pub use client::GitHubClient;
pub use error::GitHubError;
