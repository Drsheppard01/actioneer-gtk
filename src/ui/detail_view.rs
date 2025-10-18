use crate::api::{GitHubClient, GitHubError};
use crate::api::models::{Repo, Workflow};
use crate::favorites::FavoritesManager;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::Mutex;
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

                glib::MainContext::default().spawn_local(async move {
                    // Run on tokio runtime
                    let result = crate::runtime_handle().spawn(async move {
                        if is_active {
                            manager.add_favorite(repo_id).await
                        } else {
                            manager.remove_favorite(repo_id).await
                        }
                    }).await.unwrap();

                    if let Err(err) = result {
                        warn!("Failed to update favorite {}: {}", repo_id, err);
                        button_clone.set_active(!is_active);
                        update_detail_favorite_button(&button_clone, !is_active);
                        return;
                    }

                    let mut favorites = favorites_state.lock();
                    if is_active {
                        favorites.insert(repo_id);
                    } else {
                        favorites.remove(&repo_id);
                    }
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

            // Run watcher: spawn on tokio, but don't capture GTK widgets
            glib::MainContext::default().spawn_local(async move {
                let mut receiver_local = receiver;
                loop {
                    // This await needs tokio context, so we wrap it
                    let changed_result = crate::runtime_handle().spawn(async move {
                        let result = receiver_local.changed().await;
                        (receiver_local, result)
                    }).await.unwrap();
                    
                    receiver_local = changed_result.0;
                    if changed_result.1.is_err() {
                        break;
                    }
                    
                    let snapshot = receiver_local.borrow().clone();
                    let is_favorite = snapshot.contains(&repo_id);

                    if button.is_active() != is_favorite {
                        button.set_active(is_favorite);
                    }
                    update_detail_favorite_button(&button, is_favorite);
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

        glib::MainContext::default().spawn_local(async move {
            let client_clone = client.lock().clone();
            
            // Run HTTP call on tokio runtime
            let result = crate::runtime_handle().spawn(async move {
                fetch_workflows(&client_clone, &owner, &repo_name).await
            }).await.unwrap();
            
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

            glib::MainContext::default().spawn_local(async move {
                let client_clone = client.lock().clone();
                
                // Run HTTP call on tokio runtime
                let result = crate::runtime_handle().spawn(async move {
                    fetch_workflows(&client_clone, &owner, &repo_name).await
                }).await.unwrap();
                
                match result {
                    Ok(wf_list) => {
                        info!("Refreshed {} workflows", wf_list.len());
                        *workflows.lock() = wf_list.clone();
                        update_workflows_list(&list_box, &wf_list);
                    }
                    Err(e) => {
                        error!("Failed to refresh workflows: {}", e);
                    }
                }
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
            let workflows = workflows.clone();
            let parent = parent.clone();
            let client = client.clone();
            let repo = repo.clone();

            glib::MainContext::default().spawn_local(async move {
                let workflows_lock = workflows.lock();
                if let Some(workflow) = workflows_lock.get(index) {
                    let runs_window = super::workflow_runs_window::WorkflowRunsWindow::new(
                        &parent,
                        repo.clone(),
                        workflow.clone(),
                        client.clone(),
                    );
                    runs_window.present();
                }
            });
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
