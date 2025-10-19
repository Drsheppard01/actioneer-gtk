use crate::auth::device::{
    poll_device_token, start_device_flow, AccessToken, AuthError, DeviceFlowInfo,
};
use crate::config::Config;
use crate::runtime_handle;
use crate::storage::TokenStorage;
use crate::ui::utils::MainContextChannelExt;
use glib::ControlFlow;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

enum AuthMessage {
    FlowReady(DeviceFlowInfo),
    FlowError(String),
    PollSuccess(AccessToken),
    PollError(String),
}

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
            device_info,
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

        let title = gtk::Label::new(Some("Sign in to GitHub"));
        title.add_css_class("title-1");
        content_box.append(&title);

        let status_label = gtk::Label::new(Some("Initializing authentication..."));
        status_label.set_wrap(true);
        status_label.set_justify(gtk::Justification::Center);
        content_box.append(&status_label);

        let code_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        code_box.set_visible(false);

        let code_label = gtk::Label::new(Some("Enter this code on GitHub:"));
        code_box.append(&code_label);

        let user_code = gtk::Label::new(Some(""));
        user_code.add_css_class("title-2");
        user_code.set_selectable(true);
        code_box.append(&user_code);

        content_box.append(&code_box);

        let open_button = gtk::Button::with_label("Open GitHub in Browser");
        open_button.add_css_class("suggested-action");
        open_button.add_css_class("pill");
        open_button.set_visible(false);
        content_box.append(&open_button);

        let spinner = gtk::Spinner::new();
        spinner.set_visible(false);
        content_box.append(&spinner);

        let cancel_button = gtk::Button::with_label("Cancel");
        content_box.append(&cancel_button);

        self.window.set_content(Some(&content_box));

        let device_info_for_open = self.device_info.clone();
        open_button.connect_clicked(move |_| {
            if let Some(info) = device_info_for_open.lock().clone() {
                let _ = open::that(&info.verification_uri);
            }
        });

        let cancel_window = self.window.clone();
        cancel_button.connect_clicked(move |_| {
            cancel_window.close();
        });

        let device_info_clone = self.device_info.clone();
        let status_clone = status_label.clone();
        let code_box_clone = code_box.clone();
        let user_code_clone = user_code.clone();
        let open_button_clone = open_button.clone();
        let spinner_clone = spinner.clone();
        let window_clone = self.window.clone();

        self.window.connect_show(move |_| {
            *device_info_clone.lock() = None;
            status_clone.set_text("Initializing authentication...");
            code_box_clone.set_visible(false);
            user_code_clone.set_text("");
            open_button_clone.set_visible(false);
            spinner_clone.stop();
            spinner_clone.set_visible(false);

            let (sender, receiver) =
                glib::MainContext::default().channel::<AuthMessage>(glib::Priority::default());

            let start_sender = sender.clone();
            runtime_handle().spawn(async move {
                info!("Starting device flow authentication");
                let scopes = ["repo", "workflow"];
                let message = match start_device_flow(Config::github_client_id(), &scopes).await {
                    Ok(flow_info) => AuthMessage::FlowReady(flow_info),
                    Err(err) => AuthMessage::FlowError(err.to_string()),
                };

                if start_sender.send(message).is_err() {
                    error!("Failed to deliver authentication flow result to UI");
                }
            });

            let poll_sender = sender.clone();

            receiver.attach(None, {
                let device_info = device_info_clone.clone();
                let status = status_clone.clone();
                let code_box = code_box_clone.clone();
                let user_code = user_code_clone.clone();
                let open_button = open_button_clone.clone();
                let spinner = spinner_clone.clone();
                let window = window_clone.clone();

                move |message| match message {
                    AuthMessage::FlowReady(info) => {
                        *device_info.lock() = Some(info.clone());
                        user_code.set_text(&info.user_code);
                        code_box.set_visible(true);
                        open_button.set_visible(true);
                        spinner.set_visible(true);
                        spinner.start();
                        status.set_text("Open GitHub in your browser and enter the code.");

                        let poll_info = info.clone();
                        let sender_for_polling = poll_sender.clone();

                        runtime_handle().spawn(async move {
                            let interval_secs = poll_info.interval.max(1) as u64;
                            let interval = Duration::from_secs(interval_secs);
                            let max_attempts =
                                (poll_info.expires_in / poll_info.interval.max(1)).max(1) as usize;
                            let mut attempts = 0usize;

                            loop {
                                if attempts >= max_attempts {
                                    let _ = sender_for_polling.send(AuthMessage::PollError(
                                        "Authentication timeout".into(),
                                    ));
                                    break;
                                }

                                tokio::time::sleep(interval).await;
                                attempts += 1;

                                match poll_device_token(
                                    Config::github_client_id(),
                                    &poll_info.device_code,
                                )
                                .await
                                {
                                    Ok(token) => {
                                        info!("Authentication successful");
                                        if sender_for_polling
                                            .send(AuthMessage::PollSuccess(token))
                                            .is_err()
                                        {
                                            error!(
                                                "Failed to deliver authentication success to UI"
                                            );
                                        }
                                        break;
                                    }
                                    Err(AuthError::AuthorizationPending) => continue,
                                    Err(AuthError::SlowDown) => {
                                        tokio::time::sleep(interval).await;
                                    }
                                    Err(AuthError::ExpiredToken) => {
                                        let _ = sender_for_polling.send(AuthMessage::PollError(
                                            "Authentication timeout".into(),
                                        ));
                                        break;
                                    }
                                    Err(AuthError::AccessDenied) => {
                                        let _ = sender_for_polling
                                            .send(AuthMessage::PollError("Access denied".into()));
                                        break;
                                    }
                                    Err(AuthError::RequestFailed(err)) => {
                                        let _ = sender_for_polling
                                            .send(AuthMessage::PollError(err.to_string()));
                                        break;
                                    }
                                    Err(AuthError::Unknown(err)) => {
                                        let _ =
                                            sender_for_polling.send(AuthMessage::PollError(err));
                                        break;
                                    }
                                }
                            }
                        });

                        status.set_text("Waiting for authorization...");
                        ControlFlow::Continue
                    }
                    AuthMessage::FlowError(err) => {
                        error!("Failed to start device flow: {}", err);
                        status.set_text(&format!("Error: {}", err));
                        spinner.stop();
                        spinner.set_visible(false);
                        ControlFlow::Break
                    }
                    AuthMessage::PollSuccess(token) => {
                        spinner.stop();
                        spinner.set_visible(false);
                        match save_token_and_close(token, &window) {
                            Ok(()) => status.set_text("Signed in successfully"),
                            Err(err) => {
                                error!("Failed to save token: {}", err);
                                status.set_text(&format!("Error saving token: {}", err));
                            }
                        }
                        ControlFlow::Break
                    }
                    AuthMessage::PollError(err) => {
                        error!("Polling failed: {}", err);
                        spinner.stop();
                        spinner.set_visible(false);
                        status.set_text(&format!("Error: {}", err));
                        ControlFlow::Break
                    }
                }
            });
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn save_token_and_close(
    token: AccessToken,
    window: &adw::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    let storage = TokenStorage::new()?;
    storage.save_token(&token.token)?;
    info!("Token saved successfully");

    window.close();

    Ok(())
}
