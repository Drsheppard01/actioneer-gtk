use crate::api::models::{Repo, Workflow};
use crate::api::{GitHubClient, GitHubError};
use crate::favorites::FavoritesManager;
use crate::ui::utils::MainContextChannelExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info, warn};

mod helpers;
use helpers::create_workflow_expander_row;

pub struct RepoDetailPane {
    #[allow(dead_code)]
    parent: adw::ApplicationWindow,
    repo: Repo,
    client: Arc<Mutex<GitHubClient>>,
    workflows: Arc<Mutex<Vec<Workflow>>>,
    #[allow(dead_code)]
    expanded_workflows: Arc<Mutex<HashSet<i64>>>,
    favorites_manager: Option<Arc<FavoritesManager>>,
    favorites: Arc<Mutex<HashSet<i64>>>,
    favorite_button: gtk::ToggleButton,
    refresh_button: gtk::Button,
    buttons_box: gtk::Box,
    list_box: gtk::ListBox,
    root: gtk::Box,
}

impl RepoDetailPane {
    pub fn new(
        parent: adw::ApplicationWindow,
        repo: Repo,
        client: Arc<Mutex<GitHubClient>>,
        favorites_manager: Option<Arc<FavoritesManager>>,
        favorites: Arc<Mutex<HashSet<i64>>>,
    ) -> Self {
        info!("Creating RepoDetailPane for: {}", repo.full_name);
        let workflows = Arc::new(Mutex::new(Vec::new()));
        let expanded_workflows = Arc::new(Mutex::new(HashSet::new()));

        let favorite_button = gtk::ToggleButton::new();
        favorite_button.set_icon_name("emblem-favorite-symbolic");
        favorite_button.add_css_class("flat");
        favorite_button.set_tooltip_text(Some("Toggle favorite"));

        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh workflows"));
        refresh_button.add_css_class("flat");

        let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        buttons_box.set_valign(gtk::Align::Center);

        let list_box = gtk::ListBox::new();
        list_box.add_css_class("boxed-list");
        list_box.set_margin_top(12);
        list_box.set_margin_bottom(12);
        list_box.set_margin_start(12);
        list_box.set_margin_end(12);

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let pane = Self {
            parent: parent.clone(),
            repo: repo.clone(),
            client: client.clone(),
            workflows: workflows.clone(),
            expanded_workflows: expanded_workflows.clone(),
            favorites_manager: favorites_manager.clone(),
            favorites: favorites.clone(),
            favorite_button: favorite_button.clone(),
            refresh_button: refresh_button.clone(),
            buttons_box: buttons_box.clone(),
            list_box: list_box.clone(),
            root: root.clone(),
        };

        pane.build_ui();
        pane.setup_favorite_button();
        pane.observe_favorites();
        pane.load_workflows();
        pane
    }

    pub fn widget(&self) -> gtk::Widget {
        self.root.clone().upcast::<gtk::Widget>()
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    fn build_ui(&self) {
        // Header section with repo info, favorite and refresh buttons
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        header_box.set_margin_top(24);
        header_box.set_margin_bottom(12);
        header_box.set_margin_start(24);
        header_box.set_margin_end(24);

        // Left side: repo info
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info_box.set_hexpand(true);

        let repo_label = gtk::Label::new(Some(&self.repo.full_name));
        repo_label.add_css_class("title-2");
        repo_label.set_halign(gtk::Align::Start);
        info_box.append(&repo_label);

        if self.repo.is_private {
            let private_label = gtk::Label::new(Some("Private Repository"));
            private_label.add_css_class("dim-label");
            private_label.add_css_class("caption");
            private_label.set_halign(gtk::Align::Start);
            info_box.append(&private_label);
        }

        header_box.append(&info_box);

        // Right side: buttons (use stored references)
        let buttons_box = self.buttons_box.clone();
        let refresh_button = self.refresh_button.clone();
        buttons_box.append(&refresh_button);

        let favorite_button = self.favorite_button.clone();
        favorite_button.set_valign(gtk::Align::Center);
        buttons_box.append(&favorite_button);

        header_box.append(&buttons_box);

        self.root.append(&header_box);

        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        separator.set_margin_start(12);
        separator.set_margin_end(12);
        self.root.append(&separator);

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();
        scrolled.set_child(Some(&self.list_box));

        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(800);
        clamp.set_child(Some(&scrolled));

        self.root.append(&clamp);

        self.connect_refresh_button(&refresh_button);
        self.connect_workflow_selected();
    }

    fn setup_favorite_button(&self) {
        let button = self.favorite_button.clone();
        update_detail_favorite_button(&button, button.is_active());

        if let Some(manager) = &self.favorites_manager {
            let repo_id = self.repo.id;
            let manager_for_toggle = manager.clone();
            let favorites_state = self.favorites.clone();

            button.connect_toggled(move |button| {
                let is_active = button.is_active();
                update_detail_favorite_button(button, is_active);

                let manager = manager_for_toggle.clone();
                let favorites_state = favorites_state.clone();
                let button_clone = button.clone();
                let (sender, receiver) = glib::MainContext::default()
                    .channel::<Result<(), anyhow::Error>>(glib::Priority::default());

                receiver.attach(None, move |result| {
                    match result {
                        Ok(()) => {
                            let mut favorites = favorites_state.lock();
                            if is_active {
                                favorites.insert(repo_id);
                            } else {
                                favorites.remove(&repo_id);
                            }
                        }
                        Err(err) => {
                            warn!("Failed to update favorite {}: {}", repo_id, err);
                            let revert_state = !is_active;
                            button_clone.set_active(revert_state);
                            update_detail_favorite_button(&button_clone, revert_state);
                        }
                    }

                    glib::ControlFlow::Break
                });

                crate::runtime_handle().spawn(async move {
                    let outcome = if is_active {
                        manager.add_favorite(repo_id).await
                    } else {
                        manager.remove_favorite(repo_id).await
                    };

                    let _ = sender.send(outcome);
                });
            });
        } else {
            button.set_sensitive(false);
            button.set_tooltip_text(Some("Favorites unavailable"));
        }
    }

    fn observe_favorites(&self) {
        if let Some(manager) = &self.favorites_manager {
            let receiver = manager.subscribe();
            let button = self.favorite_button.clone();
            let repo_id = self.repo.id;

            let (sender, receiver_channel) =
                glib::MainContext::default().channel::<bool>(glib::Priority::default());

            receiver_channel.attach(None, move |is_favorite| {
                if button.is_active() != is_favorite {
                    button.set_active(is_favorite);
                }
                update_detail_favorite_button(&button, is_favorite);

                glib::ControlFlow::Continue
            });

            crate::runtime_handle().spawn(async move {
                let mut receiver_local = receiver;

                if sender
                    .send(receiver_local.borrow().contains(&repo_id))
                    .is_err()
                {
                    return;
                }

                loop {
                    if receiver_local.changed().await.is_err() {
                        break;
                    }

                    let is_favorite = receiver_local.borrow().contains(&repo_id);
                    if sender.send(is_favorite).is_err() {
                        break;
                    }
                }
            });
        } else {
            update_detail_favorite_button(&self.favorite_button, false);
            self.favorite_button.set_sensitive(false);
        }
    }

    fn load_workflows(&self) {
        let client = self.client.clone();
        let workflows = self.workflows.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let list_box = self.list_box.clone();

        // Show loading spinner
        self.show_loading(true);
        let callback_refs = self.clone_for_callbacks();

        let (sender, receiver) = glib::MainContext::default()
            .channel::<Result<Vec<Workflow>, GitHubError>>(glib::Priority::default());

        // Clone for spawn closure
        let client_for_spawn = client.clone();
        let owner_for_spawn = owner.clone();
        let repo_name_for_spawn = repo_name.clone();

        receiver.attach(None, move |result| {
            // Hide loading spinner
            callback_refs.show_loading(false);

            match result {
                Ok(wf_list) => {
                    info!("Loaded {} workflows", wf_list.len());
                    *workflows.lock() = wf_list.clone();
                    update_workflows_list(&list_box, &wf_list, &client, &owner, &repo_name);
                }
                Err(e) => {
                    error!("Failed to load workflows: {}", e);
                }
            }

            glib::ControlFlow::Break
        });

        crate::runtime_handle().spawn(async move {
            let client_clone = client_for_spawn.lock().clone();
            let result =
                fetch_workflows(&client_clone, &owner_for_spawn, &repo_name_for_spawn).await;
            let _ = sender.send(result);
        });
    }

    pub fn refresh_workflows_silent(&self) {
        let client = self.client.clone();
        let workflows = self.workflows.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let list_box = self.list_box.clone();

        let (sender, receiver) = glib::MainContext::default()
            .channel::<Result<Vec<Workflow>, GitHubError>>(glib::Priority::default());

        // Clone for spawn
        let client_for_spawn = client.clone();
        let owner_for_spawn = owner.clone();
        let repo_name_for_spawn = repo_name.clone();

        receiver.attach(None, move |result| {
            match result {
                Ok(wf_list) => {
                    // Only update if there are changes (ETag will prevent unnecessary updates)
                    let current = workflows.lock().clone();
                    if workflows_differ(&current, &wf_list) {
                        info!("Silent refresh detected workflow changes");
                        *workflows.lock() = wf_list.clone();
                        update_workflows_list(&list_box, &wf_list, &client, &owner, &repo_name);
                    }
                }
                Err(e) => {
                    // Silent refresh failures are logged but not shown to user
                    if !matches!(e, GitHubError::ApiError(ref msg) if msg.contains("Not modified"))
                    {
                        warn!("Silent workflow refresh failed: {}", e);
                    }
                }
            }

            glib::ControlFlow::Break
        });

        crate::runtime_handle().spawn(async move {
            let client_clone = client_for_spawn.lock().clone();
            let result =
                fetch_workflows(&client_clone, &owner_for_spawn, &repo_name_for_spawn).await;
            let _ = sender.send(result);
        });
    }

    fn connect_refresh_button(&self, button: &gtk::Button) {
        let client = self.client.clone();
        let workflows = self.workflows.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let list_box = self.list_box.clone();
        let callback_refs = self.clone_for_callbacks();

        button.connect_clicked(move |_| {
            let client = client.clone();
            let workflows = workflows.clone();
            let owner = owner.clone();
            let repo_name = repo_name.clone();
            let list_box = list_box.clone();
            let callback_refs = callback_refs.clone();

            // Show loading spinner
            callback_refs.show_loading(true);

            let (sender, receiver) = glib::MainContext::default()
                .channel::<Result<Vec<Workflow>, GitHubError>>(glib::Priority::default());
            let list_box_for_ui = list_box.clone();
            let workflows_for_ui = workflows.clone();
            let client_for_ui = client.clone();
            let owner_for_ui = owner.clone();
            let repo_name_for_ui = repo_name.clone();
            let callback_refs_for_ui = callback_refs.clone();

            receiver.attach(None, move |result| {
                // Hide loading spinner
                callback_refs_for_ui.show_loading(false);

                match result {
                    Ok(wf_list) => {
                        info!("Refreshed {} workflows", wf_list.len());
                        *workflows_for_ui.lock() = wf_list.clone();
                        update_workflows_list(
                            &list_box_for_ui,
                            &wf_list,
                            &client_for_ui,
                            &owner_for_ui,
                            &repo_name_for_ui,
                        );
                    }
                    Err(e) => {
                        error!("Failed to refresh workflows: {}", e);
                    }
                }

                glib::ControlFlow::Break
            });

            crate::runtime_handle().spawn(async move {
                let client_clone = client.lock().clone();
                let result = fetch_workflows(&client_clone, &owner, &repo_name).await;
                let _ = sender.send(result);
            });
        });
    }

    fn connect_workflow_selected(&self) {
        // Workflows are now expanded in-place, no need to open a window
        // The row activation will be handled by the expander widget
    }

    fn clone_for_callbacks(&self) -> CallbackRefs {
        CallbackRefs {
            refresh_button: self.refresh_button.clone(),
            buttons_box: self.buttons_box.clone(),
        }
    }

    fn show_loading(&self, loading: bool) {
        let refresh_button = self.refresh_button.clone();
        let buttons_box = self.buttons_box.clone();

        glib::idle_add_local_once(move || {
            if loading {
                // Hide refresh button and add spinner
                refresh_button.set_visible(false);

                // Remove any existing spinner first
                let mut child = buttons_box.first_child();
                while let Some(widget) = child.as_ref() {
                    let next = widget.next_sibling();
                    if widget.widget_name().as_str() == "detail-spinner" {
                        buttons_box.remove(widget);
                    }
                    child = next;
                }

                let spinner = gtk::Spinner::new();
                spinner.start();
                spinner.set_tooltip_text(Some("Loading workflows..."));
                spinner.set_widget_name("detail-spinner");
                spinner.set_size_request(24, 24);
                buttons_box.prepend(&spinner);
                spinner.set_visible(true);
            } else {
                // Remove spinner and show refresh button
                let mut child = buttons_box.first_child();
                while let Some(widget) = child.as_ref() {
                    let next = widget.next_sibling();
                    if widget.widget_name().as_str() == "detail-spinner" {
                        buttons_box.remove(widget);
                    }
                    child = next;
                }

                refresh_button.set_visible(true);
            }
        });
    }
}

// Helper struct for callback closures
#[derive(Clone)]
struct CallbackRefs {
    refresh_button: gtk::Button,
    buttons_box: gtk::Box,
}

impl CallbackRefs {
    fn show_loading(&self, loading: bool) {
        let refresh_button = self.refresh_button.clone();
        let buttons_box = self.buttons_box.clone();

        glib::idle_add_local_once(move || {
            if loading {
                refresh_button.set_visible(false);

                // Remove any existing spinner first
                let mut child = buttons_box.first_child();
                while let Some(widget) = child.as_ref() {
                    let next = widget.next_sibling();
                    if widget.widget_name().as_str() == "detail-spinner" {
                        buttons_box.remove(widget);
                    }
                    child = next;
                }

                let spinner = gtk::Spinner::new();
                spinner.start();
                spinner.set_tooltip_text(Some("Loading workflows..."));
                spinner.set_widget_name("detail-spinner");
                spinner.set_size_request(24, 24);
                buttons_box.prepend(&spinner);
                spinner.set_visible(true);
            } else {
                // Remove all spinners
                let mut child = buttons_box.first_child();
                while let Some(widget) = child.as_ref() {
                    let next = widget.next_sibling();
                    if widget.widget_name().as_str() == "detail-spinner" {
                        buttons_box.remove(widget);
                    }
                    child = next;
                }

                refresh_button.set_visible(true);
            }
        });
    }
}

async fn fetch_workflows(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
) -> Result<Vec<Workflow>, GitHubError> {
    client.list_workflows(owner, repo).await
}

fn update_workflows_list(
    list_box: &gtk::ListBox,
    workflows: &[Workflow],
    client: &Arc<Mutex<GitHubClient>>,
    owner: &str,
    repo: &str,
) {
    use std::collections::HashSet;

    // First, collect which workflows are currently expanded
    let mut expanded_ids = HashSet::new();
    let mut child = list_box.first_child();
    while let Some(widget) = child.as_ref() {
        let next_sibling = widget.next_sibling();

        if let Ok(row) = widget.clone().downcast::<gtk::ListBoxRow>() {
            // Try to find an expander in this row
            if let Some(row_child) = row.child() {
                if let Some(box_widget) = row_child.downcast_ref::<gtk::Box>() {
                    let mut inner_child = box_widget.first_child();
                    while let Some(widget) = inner_child.as_ref() {
                        let next = widget.next_sibling();

                        if let Some(expander) = widget.downcast_ref::<gtk::Expander>() {
                            if expander.is_expanded() {
                                // Extract workflow ID from widget name
                                let name = expander.widget_name();
                                let name_str = name.as_str();
                                if let Some(id_str) = name_str.strip_prefix("workflow_") {
                                    if let Ok(id) = id_str.parse::<i64>() {
                                        expanded_ids.insert(id);
                                    }
                                }
                            }
                        }
                        inner_child = next;
                    }
                }
            }
        }
        child = next_sibling;
    }

    // Clear the list
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    if workflows.is_empty() {
        let row = gtk::ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(false);

        let placeholder = gtk::Label::new(Some("No workflows found."));
        placeholder.add_css_class("dim-label");
        placeholder.set_margin_top(24);
        placeholder.set_margin_bottom(24);
        placeholder.set_margin_start(12);
        placeholder.set_margin_end(12);

        row.set_child(Some(&placeholder));
        list_box.append(&row);
        return;
    }

    for workflow in workflows {
        let should_expand = expanded_ids.contains(&workflow.id);
        let expander_row =
            create_workflow_expander_row(workflow, client, owner, repo, should_expand);
        list_box.append(&expander_row);
    }
}

fn update_detail_favorite_button(button: &gtk::ToggleButton, is_active: bool) {
    if is_active {
        button.remove_css_class("flat");
        button.add_css_class("suggested-action");
        button.set_opacity(1.0);
    } else {
        button.remove_css_class("suggested-action");
        button.add_css_class("flat");
        button.set_opacity(0.5);
    }
}

fn workflows_differ(a: &[Workflow], b: &[Workflow]) -> bool {
    if a.len() != b.len() {
        return true;
    }

    let a_ids: std::collections::HashSet<_> = a.iter().map(|w| w.id).collect();
    let b_ids: std::collections::HashSet<_> = b.iter().map(|w| w.id).collect();

    a_ids != b_ids
}
