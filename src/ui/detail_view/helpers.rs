// Helper functions for the detail view expandable UI
use crate::api::models::{Job, Workflow, WorkflowRun};
use crate::api::{GitHubClient, GitHubError};
use crate::ui::utils::MainContextChannelExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::error;

pub fn create_workflow_expander_row(
    workflow: &Workflow,
    client: &Arc<Mutex<GitHubClient>>,
    owner: &str,
    repo: &str,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.set_selectable(false);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Expander for the workflow
    let expander = gtk::Expander::new(Some(&workflow.name));
    expander.set_margin_top(8);
    expander.set_margin_bottom(8);
    expander.set_margin_start(12);
    expander.set_margin_end(12);

    // Content box for runs
    let runs_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    runs_box.set_margin_start(24);
    runs_box.set_margin_top(8);
    runs_box.set_margin_bottom(8);

    // Show placeholder initially
    let placeholder = gtk::Label::new(Some("Click to load runs..."));
    placeholder.add_css_class("dim-label");
    placeholder.set_halign(gtk::Align::Start);
    placeholder.set_margin_top(4);
    placeholder.set_margin_bottom(4);
    runs_box.append(&placeholder);

    expander.set_child(Some(&runs_box));
    main_box.append(&expander);

    // Load runs when expander is activated
    let client_clone = client.clone();
    let owner = owner.to_string();
    let repo_name = repo.to_string();
    let workflow_id = workflow.id;
    let runs_box_clone = runs_box.clone();

    expander.connect_expanded_notify(move |exp| {
        if !exp.is_expanded() {
            return;
        }

        // Check if already loaded
        let first_child = runs_box_clone.first_child();
        if let Some(child) = first_child {
            if child.is::<gtk::Label>() {
                // Still has placeholder, load runs
                load_workflow_runs(
                    client_clone.clone(),
                    owner.clone(),
                    repo_name.clone(),
                    workflow_id,
                    runs_box_clone.clone(),
                );
            }
        }
    });

    row.set_child(Some(&main_box));
    row
}

fn load_workflow_runs(
    client: Arc<Mutex<GitHubClient>>,
    owner: String,
    repo: String,
    workflow_id: i64,
    runs_box: gtk::Box,
) {
    // Show loading indicator
    while let Some(child) = runs_box.first_child() {
        runs_box.remove(&child);
    }

    let spinner = gtk::Spinner::new();
    spinner.start();
    spinner.set_margin_top(8);
    spinner.set_margin_bottom(8);
    runs_box.append(&spinner);

    let (sender, receiver) = glib::MainContext::default()
        .channel::<Result<Vec<WorkflowRun>, GitHubError>>(glib::Priority::default());

    // Clone for the spawn closure
    let client_for_spawn = client.clone();
    let owner_for_spawn = owner.clone();
    let repo_for_spawn = repo.clone();

    receiver.attach(None, move |result| {
        // Remove spinner
        while let Some(child) = runs_box.first_child() {
            runs_box.remove(&child);
        }

        match result {
            Ok(runs) if runs.is_empty() => {
                let label = gtk::Label::new(Some("No recent runs"));
                label.add_css_class("dim-label");
                label.set_halign(gtk::Align::Start);
                label.set_margin_top(4);
                label.set_margin_bottom(4);
                runs_box.append(&label);
            }
            Ok(runs) => {
                for run in runs.iter().take(10) {
                    let run_row = create_run_expander_row(run, &client, &owner, &repo);
                    runs_box.append(&run_row);
                }
            }
            Err(e) => {
                error!("Failed to load runs: {}", e);
                let label = gtk::Label::new(Some("Failed to load runs"));
                label.add_css_class("dim-label");
                label.set_halign(gtk::Align::Start);
                runs_box.append(&label);
            }
        }

        glib::ControlFlow::Break
    });

    crate::runtime_handle().spawn(async move {
        let client_guard = client_for_spawn.lock().clone();
        let result = client_guard
            .list_runs(&owner_for_spawn, &repo_for_spawn, workflow_id)
            .await;
        let _ = sender.send(result);
    });
}

fn create_run_expander_row(
    run: &WorkflowRun,
    client: &Arc<Mutex<GitHubClient>>,
    owner: &str,
    repo: &str,
) -> gtk::Box {
    let run_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    run_box.set_margin_top(4);
    run_box.set_margin_bottom(4);

    // Expander for the run
    let run_title = format_run_title(run);
    let expander = gtk::Expander::new(Some(&run_title));
    expander.set_margin_start(0);

    // Create custom label widget with status icon
    let label_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let status_icon = gtk::Image::from_icon_name(get_run_status_icon(run));
    status_icon.add_css_class(get_run_status_class(run));
    label_box.append(&status_icon);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);

    let title_label = gtk::Label::new(Some(&run_title));
    title_label.set_halign(gtk::Align::Start);
    text_box.append(&title_label);

    let subtitle_label = gtk::Label::new(Some(&format_run_subtitle(run)));
    subtitle_label.add_css_class("dim-label");
    subtitle_label.add_css_class("caption");
    subtitle_label.set_halign(gtk::Align::Start);
    text_box.append(&subtitle_label);

    label_box.append(&text_box);
    expander.set_label_widget(Some(&label_box));

    // Jobs box
    let jobs_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    jobs_box.set_margin_start(24);
    jobs_box.set_margin_top(4);
    jobs_box.set_margin_bottom(4);

    let placeholder = gtk::Label::new(Some("Click to load jobs..."));
    placeholder.add_css_class("dim-label");
    placeholder.set_halign(gtk::Align::Start);
    jobs_box.append(&placeholder);

    expander.set_child(Some(&jobs_box));
    run_box.append(&expander);

    // Load jobs when expanded
    let client_clone = client.clone();
    let owner = owner.to_string();
    let repo_name = repo.to_string();
    let run_id = run.id;
    let jobs_box_clone = jobs_box.clone();

    expander.connect_expanded_notify(move |exp| {
        if !exp.is_expanded() {
            return;
        }

        let first_child = jobs_box_clone.first_child();
        if let Some(child) = first_child {
            if child.is::<gtk::Label>() {
                load_run_jobs(
                    client_clone.clone(),
                    owner.clone(),
                    repo_name.clone(),
                    run_id,
                    jobs_box_clone.clone(),
                );
            }
        }
    });

    run_box
}

fn load_run_jobs(
    client: Arc<Mutex<GitHubClient>>,
    owner: String,
    repo: String,
    run_id: i64,
    jobs_box: gtk::Box,
) {
    // Show loading
    while let Some(child) = jobs_box.first_child() {
        jobs_box.remove(&child);
    }

    let spinner = gtk::Spinner::new();
    spinner.start();
    jobs_box.append(&spinner);

    let (sender, receiver) = glib::MainContext::default()
        .channel::<Result<Vec<Job>, GitHubError>>(glib::Priority::default());

    receiver.attach(None, move |result| {
        while let Some(child) = jobs_box.first_child() {
            jobs_box.remove(&child);
        }

        match result {
            Ok(jobs) if jobs.is_empty() => {
                let label = gtk::Label::new(Some("No jobs found"));
                label.add_css_class("dim-label");
                label.set_halign(gtk::Align::Start);
                jobs_box.append(&label);
            }
            Ok(jobs) => {
                for job in jobs.iter() {
                    let job_row = create_job_row(job);
                    jobs_box.append(&job_row);
                }
            }
            Err(e) => {
                error!("Failed to load jobs: {}", e);
                let label = gtk::Label::new(Some("Failed to load jobs"));
                label.add_css_class("dim-label");
                label.set_halign(gtk::Align::Start);
                jobs_box.append(&label);
            }
        }

        glib::ControlFlow::Break
    });

    crate::runtime_handle().spawn(async move {
        let client_guard = client.lock().clone();
        let result = client_guard.list_jobs(&owner, &repo, run_id).await;
        let _ = sender.send(result);
    });
}

fn create_job_row(job: &Job) -> gtk::Box {
    let job_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    job_box.set_margin_top(4);
    job_box.set_margin_bottom(4);

    let icon = gtk::Image::from_icon_name(get_job_status_icon(job));
    icon.add_css_class(get_job_status_class(job));
    job_box.append(&icon);

    let label = gtk::Label::new(Some(job.name.as_deref().unwrap_or("Unnamed job")));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    job_box.append(&label);

    let status_label = gtk::Label::new(Some(&format_job_status(job)));
    status_label.add_css_class("dim-label");
    status_label.add_css_class("caption");
    job_box.append(&status_label);

    job_box
}

// Helper functions for formatting
fn format_run_title(run: &WorkflowRun) -> String {
    let base = run
        .display_title
        .as_ref()
        .or(run.name.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("Workflow Run");

    if let Some(num) = run.run_number {
        format!("{} #{}", base, num)
    } else {
        base.to_string()
    }
}

fn format_run_subtitle(run: &WorkflowRun) -> String {
    let mut parts = Vec::new();

    if let Some(status) = &run.status {
        parts.push(status.clone());
    }

    if let Some(conclusion) = &run.conclusion {
        parts.push(conclusion.clone());
    }

    if let Some(branch) = &run.head_branch {
        parts.push(branch.clone());
    }

    parts.join(" • ")
}

fn format_job_status(job: &Job) -> String {
    if let Some(conclusion) = &job.conclusion {
        conclusion.clone()
    } else if let Some(status) = &job.status {
        status.clone()
    } else {
        "unknown".to_string()
    }
}

fn get_run_status_icon(run: &WorkflowRun) -> &'static str {
    if let Some(conclusion) = run.conclusion.as_deref() {
        match conclusion {
            "success" => "emblem-default-symbolic",
            "failure" => "dialog-error-symbolic",
            "cancelled" => "process-stop-symbolic",
            _ => "dialog-question-symbolic",
        }
    } else if let Some(status) = run.status.as_deref() {
        match status {
            "queued" | "waiting" => "alarm-symbolic",
            "in_progress" => "media-playback-start-symbolic",
            _ => "dialog-question-symbolic",
        }
    } else {
        "dialog-question-symbolic"
    }
}

fn get_run_status_class(run: &WorkflowRun) -> &'static str {
    if let Some(conclusion) = run.conclusion.as_deref() {
        return match conclusion {
            "success" => "success",
            "failure" => "error",
            "cancelled" => "warning",
            _ => "",
        };
    }
    if let Some("in_progress") = run.status.as_deref() {
        return "accent";
    }
    ""
}

fn get_job_status_icon(job: &Job) -> &'static str {
    if let Some(conclusion) = job.conclusion.as_deref() {
        match conclusion {
            "success" => "emblem-default-symbolic",
            "failure" => "dialog-error-symbolic",
            "cancelled" => "process-stop-symbolic",
            _ => "dialog-question-symbolic",
        }
    } else if let Some(status) = job.status.as_deref() {
        match status {
            "queued" | "waiting" => "alarm-symbolic",
            "in_progress" => "media-playback-start-symbolic",
            _ => "dialog-question-symbolic",
        }
    } else {
        "dialog-question-symbolic"
    }
}

fn get_job_status_class(job: &Job) -> &'static str {
    if let Some(conclusion) = job.conclusion.as_deref() {
        return match conclusion {
            "success" => "success",
            "failure" => "error",
            "cancelled" => "warning",
            _ => "",
        };
    }
    if let Some("in_progress") = job.status.as_deref() {
        return "accent";
    }
    ""
}
