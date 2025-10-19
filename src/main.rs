mod api;
mod auth;
mod cache;
mod config;
mod favorites;
mod notifications;
mod preferences;
mod storage;
mod ui;

use gtk4::prelude::*;
use libadwaita as adw;
use std::sync::OnceLock;
use tokio::runtime::{Builder, Handle};
use tracing::info;
use ui::MainWindow;

const APP_ID: &str = "com.github.Actioneer";

// Global runtime handle
static RUNTIME_HANDLE: OnceLock<Handle> = OnceLock::new();

pub fn runtime_handle() -> &'static Handle {
    RUNTIME_HANDLE.get().expect("Runtime not initialized")
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Actioneer for Linux");

    // Start tokio runtime in background thread and keep it alive
    std::thread::spawn(|| {
        let rt = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
        let handle = rt.handle().clone();

        // Store the handle globally
        RUNTIME_HANDLE
            .set(handle)
            .expect("Failed to set runtime handle");

        // Keep the runtime alive
        rt.block_on(async {
            futures::future::pending::<()>().await;
        })
    });

    // Wait for runtime to be ready
    while RUNTIME_HANDLE.get().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    info!("Tokio runtime initialized");

    // Create GTK application
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Run the application
    app.run();
    Ok(())
}

fn build_ui(app: &adw::Application) {
    let main_window = MainWindow::new(app);
    main_window.present();
}
