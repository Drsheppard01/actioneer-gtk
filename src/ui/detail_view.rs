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

pub struct RepoDetailPane {
    parent: adw::ApplicationWindow,
    repo: Repo,
    client: Arc<Mutex<GitHubClient>>,
    workflows: Arc<Mutex<Vec<Workflow>>>,
    favorites_manager: Option<Arc<FavoritesManager>>,
    favorites: Arc<Mutex<HashSet<i64>>>,
    favorite_button: gtk::ToggleButton,
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
        let workflows = Arc::new(Mutex::new(Vec::new()));

        let favorite_button = gtk::ToggleButton::new();
        favorite_button.set_icon_name("emblem-favorite-symbolic");
        favorite_button.add_css_class("flat");
        favorite_button.set_tooltip_text(Some("Toggle favorite"));

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
            favorites_manager: favorites_manager.clone(),
            favorites: favorites.clone(),
            favorite_button: favorite_button.clone(),
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

    fn build_ui(&self) {
        let header = adw::HeaderBar::new();

        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh workflows"));
        header.pack_start(&refresh_button);

        let favorite_button = self.favorite_button.clone();
        favorite_button.set_valign(gtk::Align::Center);
        header.pack_end(&favorite_button);

        self.root.append(&header);

        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        info_box.set_margin_top(12);
        info_box.set_margin_bottom(12);
        info_box.set_margin_start(12);
        info_box.set_margin_end(12);

        let repo_label = gtk::Label::new(Some(&self.repo.full_name));
        repo_label.add_css_class("title-2");
        repo_label.set_halign(gtk::Align::Start);
        info_box.append(&repo_label);

        if self.repo.is_private {
            let private_label = gtk::Label::new(Some("Private Repository"));
            private_label.add_css_class("dim-label");
            private_label.set_halign(gtk::Align::Start);
            info_box.append(&private_label);
        }

        self.root.append(&info_box);

        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
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

        let (sender, receiver) = glib::MainContext::default()
            .channel::<Result<Vec<Workflow>, GitHubError>>(glib::Priority::default());

        receiver.attach(None, move |result| {
            match result {
                Ok(wf_list) => {
                    info!("Loaded {} workflows", wf_list.len());
                    *workflows.lock() = wf_list.clone();
                    update_workflows_list(&list_box, &wf_list);
                }
                Err(e) => {
                    error!("Failed to load workflows: {}", e);
                }
            }

            glib::ControlFlow::Break
        });

        crate::runtime_handle().spawn(async move {
            let client_clone = client.lock().clone();
            let result = fetch_workflows(&client_clone, &owner, &repo_name).await;
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

        receiver.attach(None, move |result| {
            match result {
                Ok(wf_list) => {
                    // Only update if there are changes (ETag will prevent unnecessary updates)
                    let current = workflows.lock().clone();
                    if workflows_differ(&current, &wf_list) {
                        info!("Silent refresh detected workflow changes");
                        *workflows.lock() = wf_list.clone();
                        update_workflows_list(&list_box, &wf_list);
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
            let client_clone = client.lock().clone();
            let result = fetch_workflows(&client_clone, &owner, &repo_name).await;
            let _ = sender.send(result);
        });
    }

    fn connect_refresh_button(&self, button: &gtk::Button) {
        let client = self.client.clone();
        let workflows = self.workflows.clone();
        let owner = self.repo.owner.login.clone();
        let repo_name = self.repo.name.clone();
        let list_box = self.list_box.clone();

        button.connect_clicked(move |_| {
            let client = client.clone();
            let workflows = workflows.clone();
            let owner = owner.clone();
            let repo_name = repo_name.clone();
            let list_box = list_box.clone();

            let (sender, receiver) = glib::MainContext::default()
                .channel::<Result<Vec<Workflow>, GitHubError>>(glib::Priority::default());
            let list_box_for_ui = list_box.clone();
            let workflows_for_ui = workflows.clone();

            receiver.attach(None, move |result| {
                match result {
                    Ok(wf_list) => {
                        info!("Refreshed {} workflows", wf_list.len());
                        *workflows_for_ui.lock() = wf_list.clone();
                        update_workflows_list(&list_box_for_ui, &wf_list);
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
        let parent = self.parent.clone();
        let client = self.client.clone();
        let workflows = self.workflows.clone();
        let repo = self.repo.clone();
        let list_box = self.list_box.clone();

        list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let workflow = {
                let workflows_lock = workflows.lock();
                workflows_lock.get(index).cloned()
            };

            if let Some(workflow) = workflow {
                let runs_window = super::workflow_runs_window::WorkflowRunsWindow::new(
                    &parent,
                    repo.clone(),
                    workflow,
                    client.clone(),
                );
                runs_window.present();
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

fn update_workflows_list(list_box: &gtk::ListBox, workflows: &[Workflow]) {
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
        let row = create_workflow_row(workflow);
        list_box.append(&row);
    }
}

fn create_workflow_row(workflow: &Workflow) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.set_margin_top(12);
    hbox.set_margin_bottom(12);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);

    let icon = gtk::Image::from_icon_name("media-playback-start-symbolic");
    icon.set_pixel_size(24);
    hbox.append(&icon);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);

    let name_label = gtk::Label::new(Some(&workflow.name));
    name_label.set_halign(gtk::Align::Start);
    name_label.add_css_class("heading");
    vbox.append(&name_label);

    let path_label = gtk::Label::new(Some(&workflow.path));
    path_label.set_halign(gtk::Align::Start);
    path_label.add_css_class("dim-label");
    path_label.add_css_class("caption");
    vbox.append(&path_label);

    hbox.append(&vbox);

    row.set_child(Some(&hbox));
    row
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
