use crate::auth::device::{poll_device_token, start_device_flow, AccessToken, DeviceFlowInfo};
use crate::config::Config;
use crate::storage::TokenStorage;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{error, info};

pub struct AuthWindow {
    window: adw::Window,
    device_info: Arc<Mutex<Option<DeviceFlowInfo>>>,
}

impl AuthWindow {
    pub fn new(parent: Option<&impl IsA<gtk::Window>>) -> Self {
        let window = adw::Window::builder()
            .title("Sign in to GitHub")
            .modal(true)
            .default_width(500)
            .default_height(400)
            .build();

        if let Some(parent) = parent {
            window.set_transient_for(Some(parent));
        }

        let device_info = Arc::new(Mutex::new(None));

        let auth_window = Self {
            window: window.clone(),
            device_info: device_info.clone(),
        };

        auth_window.build_ui();
        auth_window
    }

    fn build_ui(&self) {
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
        content_box.set_margin_top(48);
        content_box.set_margin_bottom(48);
        content_box.set_margin_start(48);
        content_box.set_margin_end(48);
        content_box.set_valign(gtk::Align::Center);

        // Title
        let title = gtk::Label::new(Some("Sign in to GitHub"));
        title.add_css_class("title-1");
        content_box.append(&title);

        // Status label
        let status_label = gtk::Label::new(Some("Initializing authentication..."));
        status_label.set_wrap(true);
        status_label.set_justify(gtk::Justification::Center);
        content_box.append(&status_label);

        // User code display (hidden initially)
        let code_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        code_box.set_visible(false);

        let code_label = gtk::Label::new(Some("Enter this code on GitHub:"));
        code_box.append(&code_label);

        let user_code = gtk::Label::new(Some(""));
        user_code.add_css_class("title-2");
        user_code.set_selectable(true);
        code_box.append(&user_code);

        content_box.append(&code_box);

        // Open browser button
        let open_button = gtk::Button::with_label("Open GitHub in Browser");
        open_button.add_css_class("suggested-action");
        open_button.add_css_class("pill");
        open_button.set_visible(false);
        content_box.append(&open_button);

        // Progress spinner
        let spinner = gtk::Spinner::new();
        spinner.set_visible(false);
        content_box.append(&spinner);

        // Cancel button
        let cancel_button = gtk::Button::with_label("Cancel");
        content_box.append(&cancel_button);

        self.window.set_content(Some(&content_box));

        // Clone references for closures
        let window_clone = self.window.clone();
        let device_info_clone = self.device_info.clone();
        let status_clone = status_label.clone();
        let code_box_clone = code_box.clone();
        let user_code_clone = user_code.clone();
        let open_button_clone = open_button.clone();
        let spinner_clone = spinner.clone();

        // Start authentication flow when window is shown
        self.window.connect_show(move |_| {
            let device_info = device_info_clone.clone();
            let status = status_clone.clone();
            let code_box = code_box_clone.clone();
            let user_code = user_code_clone.clone();
            let open_button = open_button_clone.clone();
            let spinner = spinner_clone.clone();
            let window = window_clone.clone();

            // Spawn in glib context (not tokio) to have access to GTK widgets
            glib::MainContext::default().spawn_local(async move {
                // Enter Tokio runtime context for HTTP calls

                match start_flow(&device_info, &status, &code_box, &user_code, &open_button).await {
                    Ok(verification_uri) => {
                        // Set up browser open button
                        open_button.connect_clicked(move |_| {
                            let _ = open::that(&verification_uri);
                        });

                        // Start polling
                        spinner.set_visible(true);
                        spinner.start();
                        status.set_text("Waiting for authorization...");

                        if let Err(e) = poll_for_token(&device_info, &window).await {
                            error!("Polling failed: {}", e);
                            status.set_text(&format!("Error: {}", e));
                            spinner.stop();
                            spinner.set_visible(false);
                        }
                    }
                    Err(e) => {
                        error!("Failed to start device flow: {}", e);
                        status.set_text(&format!("Error: {}", e));
                    }
                }
            });
        });

        // Cancel button handler
        let window_for_cancel = self.window.clone();
        cancel_button.connect_clicked(move |_| {
            window_for_cancel.close();
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}

async fn start_flow(
    device_info: &Arc<Mutex<Option<DeviceFlowInfo>>>,
    status_label: &gtk::Label,
    code_box: &gtk::Box,
    user_code_label: &gtk::Label,
    open_button: &gtk::Button,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Starting device flow authentication");

    let client_id = Config::github_client_id();
    
    // Wrap the HTTP call in tokio runtime context
    let flow_info = crate::runtime_handle()
        .spawn(async move {
            start_device_flow(client_id, &["repo", "workflow"]).await
        })
        .await??;
    
    let verification_uri = flow_info.verification_uri.clone();

    user_code_label.set_text(&flow_info.user_code);
    code_box.set_visible(true);
    open_button.set_visible(true);
    status_label.set_text("Sign in to GitHub with the code above");

    *device_info.lock() = Some(flow_info);

    Ok(verification_uri)
}

async fn poll_for_token(
    device_info: &Arc<Mutex<Option<DeviceFlowInfo>>>,
    window: &adw::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    let info = device_info.lock();
    let flow_info = info
        .as_ref()
        .ok_or("No device flow info available")?
        .clone();
    drop(info);

    let interval = std::time::Duration::from_secs(flow_info.interval as u64);
    let mut attempts = 0;
    let max_attempts = (flow_info.expires_in / flow_info.interval) as usize;

    loop {
        if attempts >= max_attempts {
            return Err("Authentication timeout".into());
        }

        // Wrap tokio operations in runtime context
        crate::runtime_handle()
            .spawn(async move {
                tokio::time::sleep(interval).await;
            })
            .await?;
        
        attempts += 1;

        let client_id = Config::github_client_id();
        let device_code = flow_info.device_code.clone();
        
        // Wrap the HTTP call in tokio runtime context
        let result = crate::runtime_handle()
            .spawn(async move {
                poll_device_token(client_id, &device_code).await
            })
            .await?;
        
        match result {
            Ok(token) => {
                info!("Authentication successful!");
                save_token_and_close(token, window)?;
                return Ok(());
            }
            Err(e) => {
                use crate::auth::device::AuthError;
                match e {
                    AuthError::AuthorizationPending => {
                        // Continue polling
                        continue;
                    }
                    AuthError::SlowDown => {
                        // Double the interval
                        crate::runtime_handle()
                            .spawn(async move {
                                tokio::time::sleep(interval).await;
                            })
                            .await?;
                        continue;
                    }
                    AuthError::ExpiredToken | AuthError::AccessDenied => {
                        return Err(e.into());
                    }
                    _ => {
                        return Err(e.into());
                    }
                }
            }
        }
    }
}

fn save_token_and_close(
    token: AccessToken,
    window: &adw::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    let storage = TokenStorage::new()?;
    storage.save_token(&token.token)?;
    info!("Token saved successfully");

    // Close the window
    window.close();

    Ok(())
}
