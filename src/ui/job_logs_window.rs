#![allow(dead_code)]
use crate::api::models::{Job, Repo};
use crate::api::{GitHubClient, GitHubError};
use crate::ui::utils::MainContextChannelExt;
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

        let logs_window = Self {
            window: window.clone(),
            repo: repo.clone(),
            job: job.clone(),
            client: client.clone(),
            text_view: text_view.clone(),
        };

        let refresh_button = logs_window.build_ui();
        logs_window.connect_refresh_button(&refresh_button);
        logs_window.load_logs();
        logs_window
    }

    fn build_ui(&self) -> gtk::Button {
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();

        // Refresh button
        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh logs"));
        header.pack_start(&refresh_button);

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

        scrolled.set_child(Some(&self.text_view));
        main_box.append(&scrolled);

        self.window.set_content(Some(&main_box));
        refresh_button
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

    pub fn present(&self) {
        self.window.present();
    }
}
