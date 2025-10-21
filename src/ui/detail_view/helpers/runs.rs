use super::context::JobContextMap;
use super::formatting::{
    format_run_subtitle, format_run_title, get_run_status_class, get_run_status_icon,
    update_workflow_status_badge,
};
use super::jobs::{load_run_jobs, LoadJobsParams};
use crate::api::models::WorkflowRun;
use crate::api::{GitHubClient, GitHubError};
use crate::cache::DataCache;
use crate::ui::utils::MainContextChannelExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info};

pub(super) struct LoadRunsParams {
    pub client: Arc<Mutex<GitHubClient>>,
    pub owner: String,
    pub repo: String,
    pub workflow_id: i64,
    pub runs_box: gtk::Box,
    pub parent_window: adw::ApplicationWindow,
    pub status_badge: Option<gtk::Label>,
    pub expander: gtk::Expander,
    pub cache: Arc<DataCache>,
    pub toast_overlay: adw::ToastOverlay,
    pub bypass_cache: bool,
    pub job_contexts: JobContextMap,
    pub expanded_run_ids: Vec<i64>,
}

pub(super) fn load_workflow_runs(params: LoadRunsParams) {
    let LoadRunsParams {
        client,
        owner,
        repo,
        workflow_id,
        runs_box,
        parent_window,
        status_badge,
        expander,
        cache,
        toast_overlay,
        bypass_cache,
        job_contexts,
        expanded_run_ids,
    } = params;

    let expanded_run_ids: HashSet<i64> = expanded_run_ids.into_iter().collect();
    let expanded_run_ids = std::rc::Rc::new(expanded_run_ids);

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

    let client_for_spawn = client.clone();
    let owner_for_spawn = owner.clone();
    let repo_for_spawn = repo.clone();
    let parent_window_clone = parent_window.clone();
    let expander_for_retry = expander.clone();
    let cache_for_spawn = cache.clone();
    let job_contexts_for_retry = job_contexts.clone();
    let toast_overlay_for_retry = toast_overlay.clone();

    receiver.attach(None, move |result| {
        while let Some(child) = runs_box.first_child() {
            runs_box.remove(&child);
        }

        let expanded_run_ids_for_ui = expanded_run_ids.clone();

        match result {
            Ok(runs) if runs.is_empty() => {
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
                vbox.set_halign(gtk::Align::Start);
                vbox.set_margin_top(4);
                vbox.set_margin_bottom(4);

                let label = gtk::Label::new(Some("No recent runs"));
                label.add_css_class("dim-label");
                label.set_halign(gtk::Align::Start);
                vbox.append(&label);

                let info_label =
                    gtk::Label::new(Some("Triggered runs may take 10-30 seconds to appear"));
                info_label.add_css_class("dim-label");
                info_label.add_css_class("caption");
                info_label.set_halign(gtk::Align::Start);
                vbox.append(&info_label);

                runs_box.append(&vbox);
            }
            Ok(runs) => {
                let active_run_ids: HashSet<i64> = runs.iter().map(|run| run.id).collect();
                {
                    let mut contexts = job_contexts.lock();
                    contexts.retain(|_, ctx| {
                        if ctx.workflow_id() != workflow_id {
                            return true;
                        }
                        active_run_ids.contains(&ctx.run_id())
                    });
                }

                let cache_store = cache.clone();
                let cache_key = format!("{}/{}", owner, repo);
                let runs_cache = runs.clone();
                crate::runtime_handle().spawn(async move {
                    cache_store
                        .store_runs(runs_cache, &cache_key, workflow_id)
                        .await;
                });

                if let Some(ref badge) = status_badge {
                    if let Some(latest_run) = runs.first() {
                        update_workflow_status_badge(badge, latest_run);
                    }
                }

                let has_active_runs = runs.iter().any(|run| {
                    matches!(
                        run.status.as_deref(),
                        Some("in_progress") | Some("queued") | Some("waiting")
                    )
                });

                let widget_name = expander.widget_name();
                let base_name = widget_name.as_str().trim_end_matches("_ACTIVE");

                if has_active_runs {
                    expander.set_widget_name(&format!("{}_ACTIVE", base_name));
                    info!("Workflow {} has active runs", workflow_id);
                } else {
                    expander.set_widget_name(base_name);
                    info!("Workflow {} has no active runs", workflow_id);
                }

                let run_count = runs.len();

                let count_label = gtk::Label::new(Some(&format!("Recent runs ({})", run_count)));
                count_label.add_css_class("dim-label");
                count_label.add_css_class("caption");
                count_label.set_halign(gtk::Align::Start);
                count_label.set_margin_bottom(8);
                runs_box.append(&count_label);

                let expanded_run_ids_for_runs = expanded_run_ids_for_ui.clone();

                for run in runs.iter().take(10) {
                    let expand_jobs = expanded_run_ids_for_runs.contains(&run.id);
                    let run_row = create_run_expander_row(
                        run,
                        &client,
                        &owner,
                        &repo,
                        &parent_window_clone,
                        &cache,
                        workflow_id,
                        &toast_overlay,
                        job_contexts.clone(),
                        expand_jobs,
                    );
                    runs_box.append(&run_row);
                }
            }
            Err(e) => {
                error!("Failed to load runs: {}", e);

                let error_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
                error_box.set_halign(gtk::Align::Start);
                error_box.set_margin_top(8);
                error_box.set_margin_bottom(8);

                let error_label = gtk::Label::new(Some("Unable to load workflow runs"));
                error_label.add_css_class("dim-label");
                error_label.set_halign(gtk::Align::Start);
                error_box.append(&error_label);

                let detail_label = gtk::Label::new(Some(&format!("Error: {}", e)));
                detail_label.add_css_class("caption");
                detail_label.add_css_class("dim-label");
                detail_label.set_halign(gtk::Align::Start);
                error_box.append(&detail_label);

                let retry_button = gtk::Button::with_label("Retry");
                retry_button.add_css_class("suggested-action");
                retry_button.set_halign(gtk::Align::Start);
                retry_button.set_margin_top(8);

                let client_retry = client.clone();
                let owner_retry = owner.clone();
                let repo_retry = repo.clone();
                let runs_box_retry = runs_box.clone();
                let parent_window_retry = parent_window_clone.clone();
                let expander_retry = expander_for_retry.clone();
                let cache_retry = cache.clone();
                let toast_overlay_retry = toast_overlay_for_retry.clone();
                let job_contexts_retry = job_contexts_for_retry.clone();

                retry_button.connect_clicked(move |_| {
                    while let Some(child) = runs_box_retry.first_child() {
                        runs_box_retry.remove(&child);
                    }
                    load_workflow_runs(LoadRunsParams {
                        client: client_retry.clone(),
                        owner: owner_retry.clone(),
                        repo: repo_retry.clone(),
                        workflow_id,
                        runs_box: runs_box_retry.clone(),
                        parent_window: parent_window_retry.clone(),
                        status_badge: None,
                        expander: expander_retry.clone(),
                        cache: cache_retry.clone(),
                        toast_overlay: toast_overlay_retry.clone(),
                        bypass_cache: true,
                        job_contexts: job_contexts_retry.clone(),
                        expanded_run_ids: Vec::new(),
                    });
                });

                error_box.append(&retry_button);
                runs_box.append(&error_box);
            }
        }

        glib::ControlFlow::Break
    });

    crate::runtime_handle().spawn(async move {
        let cache_key = format!("{}/{}", owner_for_spawn, repo_for_spawn);

        if !bypass_cache {
            if let Some(cached_runs) = cache_for_spawn.runs(&cache_key, workflow_id).await {
                if !cached_runs.is_empty() {
                    info!("Using cached runs for workflow {}", workflow_id);
                    let _ = sender.send(Ok(cached_runs));
                    return;
                } else {
                    info!(
                        "Cache invalidated for workflow {}, fetching fresh data",
                        workflow_id
                    );
                }
            }
        } else {
            info!("Bypassing run cache for workflow {}", workflow_id);
        }

        let client_guard = client_for_spawn.lock().clone();
        let result = client_guard
            .list_runs(&owner_for_spawn, &repo_for_spawn, workflow_id)
            .await;

        if let Ok(ref runs) = result {
            cache_for_spawn
                .store_runs(runs.clone(), &cache_key, workflow_id)
                .await;
        }

        let _ = sender.send(result);
    });
}

pub(super) fn create_run_expander_row(
    run: &WorkflowRun,
    client: &Arc<Mutex<GitHubClient>>,
    owner: &str,
    repo: &str,
    parent_window: &adw::ApplicationWindow,
    cache: &Arc<DataCache>,
    workflow_id: i64,
    toast_overlay: &adw::ToastOverlay,
    job_contexts: JobContextMap,
    expand_jobs: bool,
) -> gtk::Box {
    let run_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    run_box.set_margin_top(4);
    run_box.set_margin_bottom(4);
    run_box.add_css_class("card");
    run_box.set_margin_start(4);
    run_box.set_margin_end(4);

    let row_container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_container.set_margin_start(12);
    row_container.set_margin_end(12);
    row_container.set_margin_top(8);
    row_container.set_margin_bottom(8);
    row_container.set_valign(gtk::Align::Center);

    let run_title = format_run_title(run);
    let expander = gtk::Expander::new(Some(&run_title));
    expander.set_margin_start(0);
    expander.set_hexpand(true);
    expander.set_valign(gtk::Align::Center);

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

    let badges_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    badges_box.set_valign(gtk::Align::Start);
    badges_box.set_margin_start(12);
    badges_box.set_margin_top(2);
    let badges_box_for_load = badges_box.clone();
    label_box.append(&badges_box);

    expander.set_label_widget(Some(&label_box));

    row_container.append(&expander);

    let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    buttons_box.set_valign(gtk::Align::Start);
    buttons_box.set_halign(gtk::Align::End);

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
        let toast_overlay_clone = toast_overlay.clone();
        let cache_clone = cache.clone();

        rerun_btn.connect_clicked(move |btn| {
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
            let toast_overlay_for_dialog = toast_overlay_clone.clone();
            let run_title_for_dialog = run_title.clone();
            let cache_for_dialog = cache_clone.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();
                if response == gtk::ResponseType::Yes {
                    btn_clone.set_sensitive(false);
                    let client = client.clone();
                    let owner = owner.clone();
                    let repo = repo.clone();
                    let toast_overlay = toast_overlay_for_dialog.clone();
                    let run_title_clone = run_title_for_dialog.clone();
                    let cache = cache_for_dialog.clone();
                    let owner_for_cache = owner.clone();
                    let repo_for_cache = repo.clone();

                    let (sender, receiver) =
                        glib::MainContext::default().channel(glib::Priority::default());

                    receiver.attach(None, move |success: bool| {
                        if success {
                            let toast =
                                adw::Toast::new(&format!("✓ Re-running '{}'", run_title_clone));
                            toast.set_timeout(3);
                            toast_overlay.add_toast(toast);
                        } else {
                            let toast = adw::Toast::new("✗ Failed to re-run workflow");
                            toast.set_timeout(5);
                            toast_overlay.add_toast(toast);
                        }
                        glib::ControlFlow::Break
                    });

                    crate::runtime_handle().spawn(async move {
                        let client_guard = client.lock().clone();
                        match client_guard.rerun_workflow(&owner, &repo, run_id).await {
                            Ok(_) => {
                                let cache_key = format!("{}/{}", owner_for_cache, repo_for_cache);
                                cache.store_runs(Vec::new(), &cache_key, workflow_id).await;
                                let _ = sender.send(true);
                            }
                            Err(e) => {
                                error!("Failed to re-run workflow: {}", e);
                                let _ = sender.send(false);
                            }
                        }
                    });
                }
            });

            dialog.present();
        });

        buttons_box.append(&rerun_btn);
    }

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
        let toast_overlay_clone = toast_overlay.clone();
        let cache_clone = cache.clone();

        rerun_failed_btn.connect_clicked(move |btn| {
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
            let toast_overlay_for_dialog = toast_overlay_clone.clone();
            let run_title_for_dialog = run_title.clone();
            let cache_for_dialog = cache_clone.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();
                if response == gtk::ResponseType::Yes {
                    btn_clone.set_sensitive(false);
                    let client = client.clone();
                    let owner = owner.clone();
                    let repo = repo.clone();
                    let toast_overlay = toast_overlay_for_dialog.clone();
                    let run_title_clone = run_title_for_dialog.clone();
                    let cache = cache_for_dialog.clone();
                    let owner_for_cache = owner.clone();
                    let repo_for_cache = repo.clone();

                    let (sender, receiver) =
                        glib::MainContext::default().channel(glib::Priority::default());

                    receiver.attach(None, move |success: bool| {
                        if success {
                            let toast = adw::Toast::new(&format!(
                                "✓ Re-running failed jobs for '{}'",
                                run_title_clone
                            ));
                            toast.set_timeout(3);
                            toast_overlay.add_toast(toast);
                        } else {
                            let toast = adw::Toast::new("✗ Failed to re-run failed jobs");
                            toast.set_timeout(5);
                            toast_overlay.add_toast(toast);
                        }
                        glib::ControlFlow::Break
                    });

                    crate::runtime_handle().spawn(async move {
                        let client_guard = client.lock().clone();
                        match client_guard.rerun_failed_jobs(&owner, &repo, run_id).await {
                            Ok(_) => {
                                let cache_key = format!("{}/{}", owner_for_cache, repo_for_cache);
                                cache.store_runs(Vec::new(), &cache_key, workflow_id).await;
                                let _ = sender.send(true);
                            }
                            Err(e) => {
                                error!("Failed to re-run failed jobs: {}", e);
                                let _ = sender.send(false);
                            }
                        }
                    });
                }
            });

            dialog.present();
        });

        buttons_box.append(&rerun_failed_btn);
    }

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
        let toast_overlay_clone = toast_overlay.clone();
        let cache_clone = cache.clone();

        cancel_btn.connect_clicked(move |btn| {
            let dialog = gtk::MessageDialog::new(
                Some(&parent_window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Warning,
                gtk::ButtonsType::YesNo,
                format!(
                    "Do you want to cancel the in-progress run \"{}\"?\n\nThis action cannot be undone.",
                    run_title
                ),
            );
            dialog.set_title(Some("Cancel Workflow Run"));

            let btn_clone = btn.clone();
            let client = client_clone.clone();
            let owner = owner.clone();
            let repo = repo_name.clone();
            let toast_overlay_for_dialog = toast_overlay_clone.clone();
            let run_title_for_dialog = run_title.clone();
            let cache_for_dialog = cache_clone.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();
                if response == gtk::ResponseType::Yes {
                    btn_clone.set_sensitive(false);
                    let client = client.clone();
                    let owner = owner.clone();
                    let repo = repo.clone();
                    let toast_overlay = toast_overlay_for_dialog.clone();
                    let run_title_clone = run_title_for_dialog.clone();
                    let cache = cache_for_dialog.clone();
                    let owner_for_cache = owner.clone();
                    let repo_for_cache = repo.clone();

                    let (sender, receiver) =
                        glib::MainContext::default().channel(glib::Priority::default());

                    receiver.attach(None, move |success: bool| {
                        if success {
                            let toast = adw::Toast::new(&format!("✓ Cancelled run '{}'", run_title_clone));
                            toast.set_timeout(3);
                            toast_overlay.add_toast(toast);
                        } else {
                            let toast = adw::Toast::new("✗ Failed to cancel run");
                            toast.set_timeout(5);
                            toast_overlay.add_toast(toast);
                        }
                        glib::ControlFlow::Break
                    });

                    crate::runtime_handle().spawn(async move {
                        let client_guard = client.lock().clone();
                        match client_guard.cancel_run(&owner, &repo, run_id).await {
                            Ok(_) => {
                                let cache_key = format!("{}/{}", owner_for_cache, repo_for_cache);
                                cache.store_runs(Vec::new(), &cache_key, workflow_id).await;
                                let _ = sender.send(true);
                            }
                            Err(e) => {
                                error!("Failed to cancel run: {}", e);
                                let _ = sender.send(false);
                            }
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

    let jobs_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    jobs_box.set_margin_start(24);
    jobs_box.set_margin_end(12);
    jobs_box.set_margin_top(4);
    jobs_box.set_margin_bottom(4);
    jobs_box.set_hexpand(true);

    let placeholder = gtk::Label::new(Some("Click to load jobs..."));
    placeholder.add_css_class("dim-label");
    placeholder.set_halign(gtk::Align::Start);
    jobs_box.append(&placeholder);

    expander.set_child(Some(&jobs_box));

    let client_clone = client.clone();
    let owner = owner.to_string();
    let repo_name = repo.to_string();
    let run_id = run.id;
    let jobs_box_clone = jobs_box.clone();
    let cache_clone = cache.clone();
    let job_contexts_clone = job_contexts.clone();
    let badges_box_clone = badges_box_for_load.clone();

    expander.connect_expanded_notify(move |exp| {
        if !exp.is_expanded() {
            let mut contexts = job_contexts_clone.lock();
            contexts.remove(&run_id);
            return;
        }

        let first_child = jobs_box_clone.first_child();
        if let Some(child) = first_child {
            if child.is::<gtk::Label>() {
                load_run_jobs(LoadJobsParams {
                    client: client_clone.clone(),
                    owner: owner.clone(),
                    repo: repo_name.clone(),
                    run_id,
                    jobs_box: jobs_box_clone.clone(),
                    badges_box: Some(badges_box_clone.clone()),
                    cache: cache_clone.clone(),
                    workflow_id,
                    background: false,
                    bypass_cache: false,
                    job_contexts: job_contexts_clone.clone(),
                });
            }
        }
    });

    if expand_jobs {
        let expander_for_expand = expander.clone();
        glib::idle_add_local_once(move || {
            expander_for_expand.set_expanded(true);
        });
    }

    run_box
}
