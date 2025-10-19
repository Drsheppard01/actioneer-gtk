#![allow(dead_code)]
use crate::api::client::GitHubClient;
use crate::api::models::{Repo, Workflow, WorkflowRun};
use crate::api::GitHubError;
use crate::runtime_handle;
use crate::ui::utils::MainContextChannelExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{error, info};

pub struct WorkflowRunsWindow {
    window: adw::Window,
    repo: Repo,
    workflow: Workflow,
    client: Arc<Mutex<GitHubClient>>,
    runs: Arc<Mutex<Vec<WorkflowRun>>>,
}

impl WorkflowRunsWindow {
    pub fn new(
        parent: &impl IsA<gtk::Window>,
        repo: Repo,
        workflow: Workflow,
        client: Arc<Mutex<GitHubClient>>,
    ) -> Self {
        let window = adw::Window::builder()
            .title(format!("{} - Runs", workflow.name))
            .modal(false)
            .default_width(900)
            .default_height(700)
            .transient_for(parent)
            .build();

        let runs = Arc::new(Mutex::new(Vec::new()));

        let runs_window = Self {
            window: window.clone(),
            repo: repo.clone(),
            workflow: workflow.clone(),
            client: client.clone(),
            runs: runs.clone(),
        };

        runs_window.build_ui();
        runs_window.load_runs();
        runs_window
    }

    fn build_ui(&self) {
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();

        // Refresh button
        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh runs"));
        header.pack_start(&refresh_button);

        main_box.append(&header);

        // Workflow info
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        info_box.set_margin_top(12);
        info_box.set_margin_bottom(12);
        info_box.set_margin_start(12);
        info_box.set_margin_end(12);

        let workflow_label = gtk::Label::new(Some(&self.workflow.name));
        workflow_label.add_css_class("title-2");
        workflow_label.set_halign(gtk::Align::Start);
        info_box.append(&workflow_label);

        let repo_label = gtk::Label::new(Some(&self.repo.full_name));
        repo_label.add_css_class("dim-label");
        repo_label.set_halign(gtk::Align::Start);
        info_box.append(&repo_label);

        main_box.append(&info_box);

        // Separator
        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        main_box.append(&separator);

        // Runs list
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();

        let list_box = gtk::ListBox::new();
        list_box.add_css_class("boxed-list");
        list_box.set_margin_top(12);
        list_box.set_margin_bottom(12);
        list_box.set_margin_start(12);
        list_box.set_margin_end(12);

        scrolled.set_child(Some(&list_box));

        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(900);
        clamp.set_child(Some(&scrolled));

        main_box.append(&clamp);

        self.window.set_content(Some(&main_box));

        // Connect signals
        self.connect_refresh_button(&refresh_button, &list_box);
        self.connect_run_selected(&list_box);
    }

    fn load_runs(&self) {
        let client = self.client.clone();
        let runs = self.runs.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let workflow_id = self.workflow.id;
        let window = self.window.clone();

        let (sender, receiver) = glib::MainContext::default()
            .channel::<Result<Vec<WorkflowRun>, GitHubError>>(glib::Priority::default());

        runtime_handle().spawn(async move {
            let client = {
                let guard = client.lock();
                guard.clone()
            };

            let result = client.list_runs(&owner, &repo_name, workflow_id).await;
            let _ = sender.send(result);
        });

        receiver.attach(None, move |result| {
            match result {
                Ok(runs_list) => {
                    info!("Loaded {} runs", runs_list.len());
                    *runs.lock() = runs_list.clone();

                    if let Some(content) = window.content() {
                        if let Ok(main_box) = content.downcast::<gtk::Box>() {
                            find_and_update_runs_list(&main_box, &runs_list);
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to load runs: {}", err);
                }
            }
            glib::ControlFlow::Break
        });
    }

    fn connect_refresh_button(&self, button: &gtk::Button, list_box: &gtk::ListBox) {
        let client = self.client.clone();
        let runs = self.runs.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let workflow_id = self.workflow.id;
        let list_box = list_box.clone();

        button.connect_clicked(move |btn| {
            let client = client.clone();
            let runs = runs.clone();
            let owner = owner.clone();
            let repo_name = repo_name.clone();
            let list = list_box.clone();
            let button_ref = btn.clone();
            let button_for_receiver = btn.clone();

            button_ref.set_sensitive(false);

            let (sender, receiver) =
                glib::MainContext::default()
                    .channel::<Result<Vec<WorkflowRun>, GitHubError>>(glib::Priority::default());

            runtime_handle().spawn(async move {
                let client = {
                    let guard = client.lock();
                    guard.clone()
                };

                let result = client.list_runs(&owner, &repo_name, workflow_id).await;
                let _ = sender.send(result);
            });

            receiver.attach(None, move |result| {
                button_for_receiver.set_sensitive(true);

                match result {
                    Ok(runs_list) => {
                        info!("Refreshed {} runs", runs_list.len());
                        *runs.lock() = runs_list.clone();
                        update_runs_list(&list, &runs_list);
                    }
                    Err(err) => {
                        error!("Failed to refresh runs: {}", err);
                    }
                }
                glib::ControlFlow::Break
            });
        });
    }

    fn connect_run_selected(&self, list_box: &gtk::ListBox) {
        let window = self.window.clone();
        let client = self.client.clone();
        let runs = self.runs.clone();
        let repo = self.repo.clone();

        list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let run = {
                let runs_lock = runs.lock();
                runs_lock.get(index).cloned()
            };

            if let Some(run) = run {
                let jobs_window = super::run_jobs_window::RunJobsWindow::new(
                    &window,
                    repo.clone(),
                    run,
                    client.clone(),
                );
                jobs_window.present();
            }
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn update_runs_list(list_box: &gtk::ListBox, runs: &[WorkflowRun]) {
    // Clear existing items
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    // Add new items
    for run in runs {
        let row = create_run_row(run);
        list_box.append(&row);
    }
}

fn find_and_update_runs_list(container: &gtk::Box, runs: &[WorkflowRun]) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        if let Ok(clamp) = widget.clone().downcast::<adw::Clamp>() {
            if let Some(scrolled) = clamp.child() {
                if let Ok(sw) = scrolled.downcast::<gtk::ScrolledWindow>() {
                    if let Some(list_box) = sw.child() {
                        if let Ok(lb) = list_box.downcast::<gtk::ListBox>() {
                            update_runs_list(&lb, runs);
                            return;
                        }
                    }
                }
            }
        }
        child = widget.next_sibling();
    }
}

fn create_run_row(run: &WorkflowRun) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.set_margin_top(12);
    hbox.set_margin_bottom(12);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);

    // Status icon
    let (icon_name, css_class) = match (run.status.as_deref(), run.conclusion.as_deref()) {
        (Some("completed"), Some("success")) => ("emblem-ok-symbolic", "success"),
        (Some("completed"), Some("failure")) => ("dialog-error-symbolic", "error"),
        (Some("completed"), Some("cancelled")) => ("process-stop-symbolic", "warning"),
        (Some("in_progress"), _) => ("media-playback-start-symbolic", "accent"),
        _ => ("help-about-symbolic", ""),
    };

    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(24);
    if !css_class.is_empty() {
        icon.add_css_class(css_class);
    }
    hbox.append(&icon);

    // Run info
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);

    let title = run
        .display_title
        .as_deref()
        .or(run.name.as_deref())
        .unwrap_or("Workflow Run");
    let name_label = gtk::Label::new(Some(title));
    name_label.set_halign(gtk::Align::Start);
    name_label.add_css_class("heading");
    vbox.append(&name_label);

    let mut details = Vec::new();
    if let Some(branch) = &run.head_branch {
        details.push(format!("Branch: {}", branch));
    }
    if let Some(event) = &run.event {
        details.push(format!("Event: {}", event));
    }
    if let Some(num) = run.run_number {
        details.push(format!("Run #{}", num));
    }

    if !details.is_empty() {
        let details_label = gtk::Label::new(Some(&details.join(" • ")));
        details_label.set_halign(gtk::Align::Start);
        details_label.add_css_class("dim-label");
        details_label.add_css_class("caption");
        vbox.append(&details_label);
    }

    hbox.append(&vbox);

    row.set_child(Some(&hbox));
    row
}
