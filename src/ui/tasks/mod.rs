pub mod favorites_observer;
pub mod repo_status;

// Re-export for when they're needed in the future
#[allow(unused_imports)]
pub use favorites_observer::observe_favorites;
#[allow(unused_imports)]
pub use repo_status::spawn_repo_status_tasks;
