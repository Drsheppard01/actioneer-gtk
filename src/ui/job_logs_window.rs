use crate::api::models::{Job, Repo};
use crate::api::{GitHubClient, GitHubError};
use crate::ui::utils::MainContextChannelExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{error, info};

pub struct JobLogsWindow {
    window: adw::Window,
    repo: Repo,
    job: Job,
    client: Arc<Mutex<GitHubClient>>,
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

        let logs_window = Self {
            window: window.clone(),
            repo: repo.clone(),
            job: job.clone(),
            client: client.clone(),
        };

        logs_window.build_ui();
        logs_window.load_logs();
        logs_window
    }

    fn build_ui(&self) {
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

        let text_view = gtk::TextView::builder()
            .editable(false)
            .monospace(true)
            .left_margin(12)
            .right_margin(12)
            .top_margin(12)
            .bottom_margin(12)
            .build();

        scrolled.set_child(Some(&text_view));
        main_box.append(&scrolled);

        self.window.set_content(Some(&main_box));

        // Connect signals
        self.connect_refresh_button(&refresh_button, &text_view);
    }

    fn load_logs(&self) {
        let client = self.client.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let job_id = self.job.id;
        let window = self.window.clone();

        let (sender, receiver) = glib::MainContext::default()
            .channel::<Result<String, GitHubError>>(glib::Priority::default());

        receiver.attach(None, move |result| {
            match result {
                Ok(logs) => {
                    info!("Loaded logs ({} bytes)", logs.len());

                    if let Some(content) = window.content() {
                        if let Ok(main_box) = content.downcast::<gtk::Box>() {
                            let mut child = main_box.first_child();
                            while let Some(widget) = child {
                                if let Ok(scrolled) =
                                    widget.clone().downcast::<gtk::ScrolledWindow>()
                                {
                                    if let Some(text_view) = scrolled.child() {
                                        if let Ok(tv) = text_view.downcast::<gtk::TextView>() {
                                            let buffer = tv.buffer();
                                            buffer.set_text(&logs);
                                        }
                                    }
                                }
                                child = widget.next_sibling();
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load logs: {}", e);
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

    fn connect_refresh_button(&self, button: &gtk::Button, text_view: &gtk::TextView) {
        let client = self.client.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let job_id = self.job.id;
        let text_view = text_view.clone();

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
                        let buffer = tv_for_ui.buffer();
                        buffer.set_text(&logs);
                    }
                    Err(e) => {
                        error!("Failed to refresh logs: {}", e);
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
