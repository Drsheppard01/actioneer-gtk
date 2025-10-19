// Helper functions for the detail view expandable UI
use crate::api::models::{Job, JobSummary, Workflow, WorkflowRun};
use crate::api::{GitHubClient, GitHubError};
use crate::ui::utils::MainContextChannelExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::error;

pub fn create_workflow_expander_row(
    workflow: &Workflow,
    client: &Arc<Mutex<GitHubClient>>,
    owner: &str,
    repo: &str,
    should_expand: bool,
    parent_window: &adw::ApplicationWindow,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.set_selectable(false);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Create a horizontal box for the workflow name and status badge
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header_box.set_margin_top(8);
    header_box.set_margin_bottom(8);
    header_box.set_margin_start(12);
    header_box.set_margin_end(12);

    // Workflow name label
    let workflow_name_label = gtk::Label::new(Some(&workflow.name));
    workflow_name_label.set_halign(gtk::Align::Start);
    workflow_name_label.set_hexpand(true);
    header_box.append(&workflow_name_label);

    // Status badge (will be populated when runs are loaded)
    let status_badge = gtk::Label::new(None);
    status_badge.add_css_class("caption");
    status_badge.add_css_class("badge");
    status_badge.set_halign(gtk::Align::End);
    status_badge.set_visible(false);
    header_box.append(&status_badge);

    // Expander for the workflow
    let expander = gtk::Expander::new(None);
    expander.set_label_widget(Some(&header_box));

    // Set widget name so we can identify this expander when checking expansion state
    expander.set_widget_name(&format!("workflow_{}", workflow.id));

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
    let parent_window = parent_window.clone();
    let status_badge_clone = status_badge.clone();

    // Track if we're programmatically expanding (to avoid triggering load)
    let is_programmatic_expand = std::rc::Rc::new(std::cell::Cell::new(false));
    let is_programmatic_for_signal = is_programmatic_expand.clone();

    expander.connect_expanded_notify(move |exp| {
        if !exp.is_expanded() {
            return;
        }

        // Skip if this is a programmatic expansion (restoration)
        if is_programmatic_for_signal.get() {
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
                    parent_window.clone(),
                    Some(status_badge_clone.clone()),
                );
            }
        }
    });

    // Expand if it was previously expanded (programmatic - don't trigger load)
    if should_expand {
        is_programmatic_expand.set(true);
        expander.set_expanded(true);
        is_programmatic_expand.set(false);
    }

    row.set_child(Some(&main_box));
    row
}

fn load_workflow_runs(
    client: Arc<Mutex<GitHubClient>>,
    owner: String,
    repo: String,
    workflow_id: i64,
    runs_box: gtk::Box,
    parent_window: adw::ApplicationWindow,
    status_badge: Option<gtk::Label>,
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
    let parent_window_clone = parent_window.clone();

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
                // Update workflow status badge based on most recent run
                if let Some(ref badge) = status_badge {
                    if let Some(latest_run) = runs.first() {
                        update_workflow_status_badge(badge, latest_run);
                    }
                }

                let run_count = runs.len();

                // Add a header showing run count
                let count_label = gtk::Label::new(Some(&format!("Recent runs ({})", run_count)));
                count_label.add_css_class("dim-label");
                count_label.add_css_class("caption");
                count_label.set_halign(gtk::Align::Start);
                count_label.set_margin_bottom(8);
                runs_box.append(&count_label);

                for run in runs.iter().take(10) {
                    let run_row =
                        create_run_expander_row(run, &client, &owner, &repo, &parent_window_clone);
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
    parent_window: &adw::ApplicationWindow,
) -> gtk::Box {
    let run_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    run_box.set_margin_top(4);
    run_box.set_margin_bottom(4);

    // Main row container
    let row_container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_container.set_margin_start(0);
    row_container.set_margin_end(0);
    row_container.set_valign(gtk::Align::Center);

    // Expander for the run
    let run_title = format_run_title(run);
    let expander = gtk::Expander::new(Some(&run_title));
    expander.set_margin_start(0);
    expander.set_hexpand(true);
    expander.set_valign(gtk::Align::Center);

    // Create custom label widget with status icon
    let label_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    label_box.set_valign(gtk::Align::Center);

    let status_icon = gtk::Image::from_icon_name(get_run_status_icon(run));
    status_icon.add_css_class(get_run_status_class(run));
    status_icon.set_valign(gtk::Align::Center);
    label_box.append(&status_icon);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_box.set_valign(gtk::Align::Center);

    let title_label = gtk::Label::new(Some(&run_title));
    title_label.set_halign(gtk::Align::Start);
    title_label.set_valign(gtk::Align::Center);
    text_box.append(&title_label);

    let subtitle_label = gtk::Label::new(Some(&format_run_subtitle(run)));
    subtitle_label.add_css_class("dim-label");
    subtitle_label.add_css_class("caption");
    subtitle_label.set_halign(gtk::Align::Start);
    subtitle_label.set_valign(gtk::Align::Center);
    text_box.append(&subtitle_label);

    label_box.append(&text_box);
    expander.set_label_widget(Some(&label_box));

    row_container.append(&expander);

    // Job summary badges (placeholder that will be filled when jobs are loaded)
    let badges_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    badges_box.set_valign(gtk::Align::Center);
    badges_box.set_halign(gtk::Align::End);
    badges_box.set_margin_end(8);

    // Store badges_box reference for later update
    let badges_box_for_load = badges_box.clone();

    row_container.append(&badges_box);

    // Action buttons box
    let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    buttons_box.set_valign(gtk::Align::Center);
    buttons_box.set_halign(gtk::Align::End);

    // Open in GitHub button
    if let Some(ref url) = run.html_url {
        let open_btn = gtk::Button::from_icon_name("adw-external-link-symbolic");
        open_btn.set_tooltip_text(Some("Open in GitHub"));
        open_btn.add_css_class("flat");
        open_btn.add_css_class("circular");

        let url_clone = url.clone();
        open_btn.connect_clicked(move |_| {
            if let Err(e) = open::that(&url_clone) {
                error!("Failed to open URL: {}", e);
            }
        });

        buttons_box.append(&open_btn);
    }

    // Re-run button (only for completed runs)
    if run.is_rerunnable() {
        let rerun_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
        rerun_btn.set_tooltip_text(Some("Re-run workflow"));
        rerun_btn.add_css_class("flat");
        rerun_btn.add_css_class("circular");
        rerun_btn.add_css_class("warning");

        let client_clone = client.clone();
        let owner = owner.to_string();
        let repo_name = repo.to_string();
        let run_id = run.id;
        let parent_window = parent_window.clone();
        let run_title = format_run_title(run);

        rerun_btn.connect_clicked(move |btn| {
            // Show confirmation dialog
            let dialog = gtk::MessageDialog::new(
                Some(&parent_window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Question,
                gtk::ButtonsType::YesNo,
                format!("Do you want to re-run \"{}\"?", run_title),
            );
            dialog.set_title(Some("Re-run Workflow"));

            let btn_clone = btn.clone();
            let client = client_clone.clone();
            let owner = owner.clone();
            let repo = repo_name.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();
                if response == gtk::ResponseType::Yes {
                    btn_clone.set_sensitive(false);
                    let client = client.clone();
                    let owner = owner.clone();
                    let repo = repo.clone();

                    crate::runtime_handle().spawn(async move {
                        let client_guard = client.lock().clone();
                        if let Err(e) = client_guard.rerun_workflow(&owner, &repo, run_id).await {
                            error!("Failed to re-run workflow: {}", e);
                        }
                    });
                }
            });

            dialog.present();
        });

        buttons_box.append(&rerun_btn);
    }

    // Re-run failed jobs button
    if run.has_failed_jobs() {
        let rerun_failed_btn = gtk::Button::from_icon_name("system-reboot-symbolic");
        rerun_failed_btn.set_tooltip_text(Some("Re-run failed jobs"));
        rerun_failed_btn.add_css_class("flat");
        rerun_failed_btn.add_css_class("circular");
        rerun_failed_btn.add_css_class("error");

        let client_clone = client.clone();
        let owner = owner.to_string();
        let repo_name = repo.to_string();
        let run_id = run.id;
        let parent_window = parent_window.clone();
        let run_title = format_run_title(run);

        rerun_failed_btn.connect_clicked(move |btn| {
            // Show confirmation dialog
            let dialog = gtk::MessageDialog::new(
                Some(&parent_window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Warning,
                gtk::ButtonsType::YesNo,
                format!(
                    "Do you want to re-run all failed jobs in \"{}\"?",
                    run_title
                ),
            );
            dialog.set_title(Some("Re-run Failed Jobs"));

            let btn_clone = btn.clone();
            let client = client_clone.clone();
            let owner = owner.clone();
            let repo = repo_name.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();
                if response == gtk::ResponseType::Yes {
                    btn_clone.set_sensitive(false);
                    let client = client.clone();
                    let owner = owner.clone();
                    let repo = repo.clone();

                    crate::runtime_handle().spawn(async move {
                        let client_guard = client.lock().clone();
                        if let Err(e) = client_guard.rerun_failed_jobs(&owner, &repo, run_id).await
                        {
                            error!("Failed to re-run failed jobs: {}", e);
                        }
                    });
                }
            });

            dialog.present();
        });

        buttons_box.append(&rerun_failed_btn);
    }

    // Cancel button (only for in-progress/queued runs)
    if run.is_cancellable() {
        let cancel_btn = gtk::Button::from_icon_name("process-stop-symbolic");
        cancel_btn.set_tooltip_text(Some("Cancel run"));
        cancel_btn.add_css_class("flat");
        cancel_btn.add_css_class("circular");
        cancel_btn.add_css_class("destructive-action");

        let client_clone = client.clone();
        let owner = owner.to_string();
        let repo_name = repo.to_string();
        let run_id = run.id;
        let parent_window = parent_window.clone();
        let run_title = format_run_title(run);

        cancel_btn.connect_clicked(move |btn| {
            // Show confirmation dialog
            let dialog = gtk::MessageDialog::new(
                Some(&parent_window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Warning,
                gtk::ButtonsType::YesNo,
                format!("Do you want to cancel the in-progress run \"{}\"?\n\nThis action cannot be undone.", run_title),
            );
            dialog.set_title(Some("Cancel Workflow Run"));

            let btn_clone = btn.clone();
            let client = client_clone.clone();
            let owner = owner.clone();
            let repo = repo_name.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();
                if response == gtk::ResponseType::Yes {
                    btn_clone.set_sensitive(false);
                    let client = client.clone();
                    let owner = owner.clone();
                    let repo = repo.clone();

                    crate::runtime_handle().spawn(async move {
                        let client_guard = client.lock().clone();
                        if let Err(e) = client_guard.cancel_run(&owner, &repo, run_id).await {
                            error!("Failed to cancel run: {}", e);
                        }
                    });
                }
            });

            dialog.present();
        });

        buttons_box.append(&cancel_btn);
    }

    row_container.append(&buttons_box);
    run_box.append(&row_container);

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
                    Some(badges_box_for_load.clone()),
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
    badges_box: Option<gtk::Box>,
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
                // Update badges with job summary
                if let Some(ref badges) = badges_box {
                    update_job_summary_badges(badges, &jobs);
                }

                let total_jobs = jobs.len();
                for job in jobs.iter() {
                    let job_row = create_job_row_simple(job);
                    jobs_box.append(&job_row);
                }

                // Show job count info if there are jobs
                if total_jobs > 0 {
                    let count_label = gtk::Label::new(Some(&format!(
                        "Showing {} job{}",
                        total_jobs,
                        if total_jobs == 1 { "" } else { "s" }
                    )));
                    count_label.add_css_class("dim-label");
                    count_label.add_css_class("caption");
                    count_label.set_halign(gtk::Align::Start);
                    count_label.set_margin_top(8);
                    count_label.set_margin_bottom(4);
                    jobs_box.append(&count_label);
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

fn create_job_row_simple(job: &Job) -> gtk::Box {
    let job_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    job_box.set_margin_top(4);
    job_box.set_margin_bottom(4);
    job_box.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name(get_job_status_icon(job));
    icon.add_css_class(get_job_status_class(job));
    icon.set_valign(gtk::Align::Center);
    job_box.append(&icon);

    let job_name_label = gtk::Label::new(Some(job.name.as_deref().unwrap_or("Unnamed job")));
    job_name_label.set_halign(gtk::Align::Start);
    job_name_label.set_hexpand(true);
    job_name_label.set_valign(gtk::Align::Center);
    job_box.append(&job_name_label);

    let status_text = format_job_status(job);
    let status_label = gtk::Label::new(Some(&status_text));
    status_label.add_css_class("dim-label");
    status_label.add_css_class("caption");
    status_label.set_valign(gtk::Align::Center);
    job_box.append(&status_label);

    // Show duration if available
    if let Some(duration) = job.duration_string() {
        let duration_label = gtk::Label::new(Some(&duration));
        duration_label.add_css_class("dim-label");
        duration_label.add_css_class("caption");
        duration_label.set_valign(gtk::Align::Center);
        duration_label.set_margin_start(4);
        job_box.append(&duration_label);
    }

    // Note: View logs button would require parent window reference and client
    // For now, we only show the "Open in GitHub" button which also allows viewing logs

    // Open in GitHub button (this also allows viewing logs)
    if let Some(ref url) = job.html_url {
        let open_btn = gtk::Button::from_icon_name("adw-external-link-symbolic");
        open_btn.set_tooltip_text(Some("Open job in GitHub"));
        open_btn.add_css_class("flat");
        open_btn.add_css_class("circular");
        open_btn.set_valign(gtk::Align::Center);

        let url_clone = url.clone();
        open_btn.connect_clicked(move |_| {
            if let Err(e) = open::that(&url_clone) {
                error!("Failed to open URL: {}", e);
            }
        });

        job_box.append(&open_btn);
    }

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

    // Add status text (e.g., "completed", "in_progress")
    if let Some(status) = &run.status {
        parts.push(status.clone());
    }

    // Add conclusion if available (e.g., "success", "failure")
    if run.conclusion.is_some() {
        parts.push(run.friendly_conclusion());
    }

    // Add branch name
    if let Some(branch) = &run.head_branch {
        parts.push(branch.clone());
    }

    // Add relative time
    let time_str = run.relative_time_string();
    if !time_str.is_empty() {
        parts.push(time_str);
    }

    parts.join(" • ")
}

fn format_job_status(job: &Job) -> String {
    job.friendly_status()
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

fn update_job_summary_badges(badges_box: &gtk::Box, jobs: &[Job]) {
    // Clear existing badges
    while let Some(child) = badges_box.first_child() {
        badges_box.remove(&child);
    }

    let summary = JobSummary::from_jobs(jobs);

    // Only show badges if there are jobs
    if summary.is_empty() {
        return;
    }

    // Completed jobs badge (green checkmark)
    if summary.completed > 0 {
        let badge = create_job_badge(
            "emblem-default-symbolic",
            &summary.completed.to_string(),
            "success",
        );
        badges_box.append(&badge);
    }

    // Running jobs badge (blue bolt)
    if summary.running > 0 {
        let badge = create_job_badge(
            "media-playback-start-symbolic",
            &summary.running.to_string(),
            "accent",
        );
        badges_box.append(&badge);
    }

    // Queued jobs badge (orange clock)
    if summary.queued > 0 {
        let badge = create_job_badge("alarm-symbolic", &summary.queued.to_string(), "warning");
        badges_box.append(&badge);
    }
}

fn create_job_badge(icon_name: &str, count: &str, css_class: &str) -> gtk::Box {
    let badge = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    badge.add_css_class("badge");
    badge.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(12);
    icon.add_css_class(css_class);
    badge.append(&icon);

    let label = gtk::Label::new(Some(count));
    label.add_css_class("caption");
    label.add_css_class(css_class);
    badge.append(&label);

    badge
}

fn update_workflow_status_badge(badge: &gtk::Label, latest_run: &WorkflowRun) {
    let (text, css_class) = match (
        latest_run.status.as_deref(),
        latest_run.conclusion.as_deref(),
    ) {
        (Some("completed"), Some("success")) => ("passing", "success"),
        (Some("completed"), Some("failure")) => ("failing", "error"),
        (Some("completed"), Some("cancelled")) => ("cancelled", "warning"),
        (Some("in_progress"), _) => ("running", "accent"),
        (Some("queued"), _) | (Some("waiting"), _) => ("queued", "warning"),
        _ => ("unknown", "dim-label"),
    };

    badge.set_text(text);

    // Remove all previous CSS classes
    let classes = ["success", "error", "warning", "accent", "dim-label"];
    for class in classes {
        badge.remove_css_class(class);
    }

    badge.add_css_class(css_class);
    badge.set_visible(true);
}
