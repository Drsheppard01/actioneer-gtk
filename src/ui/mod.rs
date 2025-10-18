mod detail_placeholder;
mod detail_view;
mod sidebar;

pub mod auth_window;
pub mod job_logs_window;
pub mod main_window;
pub mod preferences_window;
pub mod run_jobs_window;
pub mod workflow_runs_window;

// New modules for better organization
pub mod state;
pub mod tasks;
pub mod utils;

pub use main_window::MainWindow;
