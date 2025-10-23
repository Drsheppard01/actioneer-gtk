#![allow(dead_code)]
use crate::api::models::{Job, Repo};
use crate::api::{GitHubClient, GitHubError};
use crate::ui::utils::MainContextChannelExt;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct JobLogsWindow {
    window: adw::Window,
    repo: Repo,
    job: Job,
    client: Arc<Mutex<GitHubClient>>,
    text_view: gtk::TextView,
    toast_overlay: adw::ToastOverlay,
}

struct ActionButtons {
    refresh: gtk::Button,
    copy: gtk::Button,
    save: gtk::Button,
}

impl JobLogsWindow {
    pub fn new(
        parent: &impl IsA<gtk::Window>,
        repo: Repo,
        job: Job,
        client: Arc<Mutex<GitHubClient>>,
    ) -> Self {
        let job_name = job.name.as_deref().unwrap_or("Job");

        let window = adw::Window::builder()
            .title(format!("{} - Logs", job_name))
            .modal(false)
            .default_width(1000)
            .default_height(700)
            .transient_for(parent)
            .build();
        let text_view = gtk::TextView::builder()
            .editable(false)
            .monospace(true)
            .left_margin(12)
            .right_margin(12)
            .top_margin(12)
            .bottom_margin(12)
            .wrap_mode(gtk::WrapMode::Word)
            .build();
        text_view.buffer().set_text("Fetching logs…");

        let toast_overlay = adw::ToastOverlay::new();
        let logs_window = Self {
            window: window.clone(),
            repo: repo.clone(),
            job: job.clone(),
            client: client.clone(),
            text_view: text_view.clone(),
            toast_overlay: toast_overlay.clone(),
        };

        let buttons = logs_window.build_ui(&toast_overlay, &text_view);
        logs_window.connect_refresh_button(&buttons.refresh);
        logs_window.connect_copy_button(&buttons.copy);
        logs_window.connect_save_button(&buttons.save);
        logs_window.load_logs();

        logs_window
    }

    fn build_ui(
        &self,
        toast_overlay: &adw::ToastOverlay,
        text_view: &gtk::TextView,
    ) -> ActionButtons {
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();
        // Refresh button
        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh logs"));
        header.pack_start(&refresh_button);

        // Copy button
        let copy_button = gtk::Button::from_icon_name("edit-copy-symbolic");
        copy_button.set_tooltip_text(Some("Copy logs to clipboard"));
        header.pack_end(&copy_button);

        // Save button
        let save_button = gtk::Button::from_icon_name("document-save-symbolic");
        save_button.set_tooltip_text(Some("Save logs to file"));
        header.pack_end(&save_button);
        main_box.append(&header);

        // Job info
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        info_box.set_margin_top(12);
        info_box.set_margin_bottom(12);
        info_box.set_margin_start(12);
        info_box.set_margin_end(12);

        let job_name = self.job.name.as_deref().unwrap_or("Job");
        let job_label = gtk::Label::new(Some(job_name));
        job_label.add_css_class("title-2");
        job_label.set_halign(gtk::Align::Start);
        info_box.append(&job_label);
        let repo_label = gtk::Label::new(Some(&self.repo.full_name));
        repo_label.add_css_class("dim-label");
        repo_label.set_halign(gtk::Align::Start);
        info_box.append(&repo_label);

        main_box.append(&info_box);
        // Separator
        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        main_box.append(&separator);
        // Logs view
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();
        scrolled.set_child(Some(text_view));
        main_box.append(&scrolled);

        toast_overlay.set_child(Some(&main_box));
        self.window.set_content(Some(toast_overlay));

        ActionButtons {
            refresh: refresh_button,
            copy: copy_button,
            save: save_button,
        }
    }

    fn load_logs(&self) {
        let client = self.client.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let job_id = self.job.id;
        let text_view = self.text_view.clone();

        let (sender, receiver) = glib::MainContext::default()
            .channel::<Result<String, GitHubError>>(glib::Priority::default());

        receiver.attach(None, move |result| {
            match result {
                Ok(logs) => {
                    info!("Loaded logs ({} bytes)", logs.len());
                    text_view.set_sensitive(true);
                    text_view.buffer().set_text(&logs);
                }
                Err(GitHubError::NotFound) => {
                    warn!("Job logs unavailable; job may still be running");
                    text_view.set_sensitive(false);
                    text_view.buffer().set_text(
                        "Logs are not yet available for this job. GitHub only provides logs once the job starts streaming output or completes. Try refreshing in a few moments.",
                    );
                }
                Err(e) => {
                    error!("Failed to load logs: {}", e);
                    text_view.set_sensitive(false);
                    text_view.buffer().set_text(&format!(
                        "Unable to load logs right now. Please try again later.\n\nDetails: {}",
                        e
                    ));
                }
            }
            glib::ControlFlow::Break
        });

        crate::runtime_handle().spawn(async move {
            let client_clone = client.lock().clone();
            let result = client_clone.get_job_logs(&owner, &repo_name, job_id).await;
            let _ = sender.send(result);
        });
    }

    fn connect_refresh_button(&self, button: &gtk::Button) {
        let client = self.client.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let job_id = self.job.id;
        let text_view = self.text_view.clone();

        button.connect_clicked(move |_| {
            let client = client.clone();
            let owner = owner.clone();
            let repo_name = repo_name.clone();
            let tv = text_view.clone();

            let (sender, receiver) = glib::MainContext::default()
                .channel::<Result<String, GitHubError>>(glib::Priority::default());
            let tv_for_ui = tv.clone();

            receiver.attach(None, move |result| {
                match result {
                    Ok(logs) => {
                        info!("Refreshed logs ({} bytes)", logs.len());
                        tv_for_ui.set_sensitive(true);
                        tv_for_ui.buffer().set_text(&logs);
                    }
                    Err(GitHubError::NotFound) => {
                        warn!("Job logs still unavailable during refresh");
                        tv_for_ui.set_sensitive(false);
                        tv_for_ui.buffer().set_text(
                            "Logs are not yet available for this job. GitHub only provides logs once the job starts streaming output or completes. Try refreshing in a few moments.",
                        );
                    }
                    Err(e) => {
                        error!("Failed to refresh logs: {}", e);
                        tv_for_ui.set_sensitive(false);
                        tv_for_ui.buffer().set_text(&format!(
                            "Unable to load logs right now. Please try again later.\n\nDetails: {}",
                            e
                        ));
                    }
                }

                glib::ControlFlow::Break
            });
            crate::runtime_handle().spawn(async move {
                let client_clone = client.lock().clone();
                let result = client_clone.get_job_logs(&owner, &repo_name, job_id).await;
                let _ = sender.send(result);
            });
        });
    }

    fn connect_copy_button(&self, button: &gtk::Button) {
        let text_view = self.text_view.clone();
        let overlay = self.toast_overlay.clone();

        button.connect_clicked(move |_| {
            let buffer = text_view.buffer();
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), true)
                .to_string();

            if text.is_empty() {
                let toast = adw::Toast::new("Logs are empty; nothing to copy");
                toast.set_timeout(3);
                overlay.add_toast(toast);
                return;
            }

            if let Some(display) = gdk::Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_text(&text);
                let toast = adw::Toast::new("Logs copied to clipboard");
                toast.set_timeout(3);
                overlay.add_toast(toast);
            } else {
                let toast = adw::Toast::new("Clipboard unavailable on this system");
                toast.set_timeout(5);
                overlay.add_toast(toast);
            }
        });
    }

    fn connect_save_button(&self, button: &gtk::Button) {
        let window = self.window.clone();
        let text_view = self.text_view.clone();
        let overlay = self.toast_overlay.clone();

        button.connect_clicked(move |_| {
            let dialog = gtk::FileChooserNative::builder()
                .title("Save Logs")
                .accept_label("Save")
                .cancel_label("Cancel")
                .action(gtk::FileChooserAction::Save)
                .transient_for(&window)
                .modal(true)
                .build();

            let overlay_for_dialog = overlay.clone();
            let text_for_dialog = text_view.clone();

            dialog.connect_response(move |dialog, response| {
                if response != gtk::ResponseType::Accept {
                    dialog.destroy();
                    return;
                }

                let buffer = text_for_dialog.buffer();
                let text = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), true)
                    .to_string();

                if text.is_empty() {
                    let toast = adw::Toast::new("Logs are empty; nothing saved");
                    toast.set_timeout(3);
                    overlay_for_dialog.add_toast(toast);
                    dialog.destroy();
                    return;
                }

                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let text_to_write = text.clone();
                        let (sender, receiver) = glib::MainContext::default()
                            .channel::<Result<(), String>>(glib::Priority::default());
                        let overlay_for_result = overlay_for_dialog.clone();

                        receiver.attach(None, move |message| {
                            match message {
                                Ok(()) => {
                                    let toast = adw::Toast::new("Logs saved");
                                    toast.set_timeout(3);
                                    overlay_for_result.add_toast(toast);
                                }
                                Err(err) => {
                                    let toast =
                                        adw::Toast::new(&format!("Failed to save logs: {}", err));
                                    toast.set_timeout(5);
                                    overlay_for_result.add_toast(toast);
                                }
                            }
                            glib::ControlFlow::Break
                        });

                        crate::runtime_handle().spawn_blocking(move || {
                            let result = std::fs::write(&path, text_to_write);
                            let _ = sender.send(result.map_err(|e| e.to_string()));
                        });
                    } else {
                        let toast = adw::Toast::new("Unable to determine save location");
                        toast.set_timeout(5);
                        overlay_for_dialog.add_toast(toast);
                    }
                } else {
                    let toast = adw::Toast::new("No file selected");
                    toast.set_timeout(5);
                    overlay_for_dialog.add_toast(toast);
                }

                dialog.destroy();
            });

            dialog.show();
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}
