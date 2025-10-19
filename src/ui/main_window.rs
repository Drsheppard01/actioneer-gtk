use super::detail_placeholder::schedule_status_page_update;
use super::detail_view::RepoDetailPane;
use super::sidebar::{
    find_label_by_name, gather_workflow_status_counts, rebuild_repo_list, row_matches_query,
};
use crate::api::models::{RateLimitInfo, Repo};
use crate::api::{GitHubClient, GitHubError};
use crate::favorites::FavoritesManager;
use crate::preferences::PreferencesManager;
use crate::storage::TokenStorage;
use crate::ui::auth_window::AuthWindow;
use crate::ui::preferences_window::PreferencesWindow;
use crate::ui::utils::{update_rate_limit_label, MainContextChannelExt};
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

// Import refactored modules
use crate::ui::state::{RepoActionsState, WorkflowStatusCounts};

#[derive(Clone)]
pub struct MainWindow {
    window: adw::ApplicationWindow,
    client: Arc<Mutex<Option<GitHubClient>>>,
    repos: Arc<Mutex<Vec<Repo>>>,
    repo_list: gtk::ListBox,
    search_entry: gtk::SearchEntry,
    rate_limit_label: gtk::Label,
    favorites_manager: Option<Arc<FavoritesManager>>,
    favorites: Arc<Mutex<HashSet<i64>>>,
    actions_states: Arc<Mutex<HashMap<i64, RepoActionsState>>>,
    workflow_counts: Arc<Mutex<HashMap<i64, WorkflowStatusCounts>>>,
    preferences_manager: Option<Arc<PreferencesManager>>,
    selected_repo_id: Arc<Mutex<Option<i64>>>,
    rate_limit_info: Arc<Mutex<Option<RateLimitInfo>>>,
    detail_status_page: adw::StatusPage,
    detail_stack: gtk::Stack,
    active_detail: Rc<RefCell<Option<RepoDetailPane>>>,
    auto_refresh_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    current_refresh_interval: Arc<Mutex<u64>>,
}

impl MainWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Actioneer")
            .default_width(1000)
            .default_height(700)
            .build();

        let client = Arc::new(Mutex::new(None));
        let repos = Arc::new(Mutex::new(Vec::new()));

        let repo_list = gtk::ListBox::new();
        repo_list.add_css_class("boxed-list");
        repo_list.set_margin_top(0);
        repo_list.set_margin_bottom(12);
        repo_list.set_margin_start(12);
        repo_list.set_margin_end(12);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search repositories..."));
        search_entry.set_margin_top(12);
        search_entry.set_margin_bottom(12);
        search_entry.set_margin_start(12);
        search_entry.set_margin_end(12);

        let rate_limit_label = gtk::Label::new(Some("Rate limit: –"));
        rate_limit_label.add_css_class("dim-label");
        rate_limit_label.add_css_class("caption");
        let detail_status_page = adw::StatusPage::builder()
            .title("Select a repository")
            .description(
                "Choose a repository from the sidebar to browse its workflows and runs here.",
            )
            .icon_name("system-search-symbolic")
            .build();
        let detail_stack = gtk::Stack::new();
        detail_stack.add_named(&detail_status_page, Some("placeholder"));
        detail_stack.set_visible_child_name("placeholder");
        detail_stack.set_vexpand(true);
        detail_stack.set_hexpand(true);
        let active_detail: Rc<RefCell<Option<RepoDetailPane>>> = Rc::new(RefCell::new(None));
        let favorites_manager = match FavoritesManager::new() {
            Ok(manager) => Some(Arc::new(manager)),
            Err(err) => {
                warn!("Failed to initialize FavoritesManager: {}", err);
                None
            }
        };
        let preferences_manager = match PreferencesManager::new() {
            Ok(manager) => Some(Arc::new(manager)),
            Err(err) => {
                warn!("Failed to initialize PreferencesManager: {}", err);
                None
            }
        };

        let favorites = Arc::new(Mutex::new(HashSet::new()));
        let actions_states = Arc::new(Mutex::new(HashMap::new()));
        let workflow_counts = Arc::new(Mutex::new(HashMap::new()));
        let selected_repo_id = Arc::new(Mutex::new(None));
        let rate_limit_info = Arc::new(Mutex::new(None));
        let auto_refresh_task = Arc::new(Mutex::new(None));
        let current_refresh_interval = Arc::new(Mutex::new(0));

        let main_window = Self {
            window: window.clone(),
            client: client.clone(),
            repos: repos.clone(),
            repo_list: repo_list.clone(),
            search_entry: search_entry.clone(),
            rate_limit_label: rate_limit_label.clone(),
            favorites_manager: favorites_manager.clone(),
            favorites: favorites.clone(),
            actions_states: actions_states.clone(),
            workflow_counts: workflow_counts.clone(),
            preferences_manager: preferences_manager.clone(),
            selected_repo_id: selected_repo_id.clone(),
            rate_limit_info: rate_limit_info.clone(),
            detail_status_page: detail_status_page.clone(),
            detail_stack: detail_stack.clone(),
            active_detail: active_detail.clone(),
            auto_refresh_task: auto_refresh_task.clone(),
            current_refresh_interval: current_refresh_interval.clone(),
        };

        main_window.build_ui();
        main_window.prime_favorites();
        main_window.observe_favorites();
        main_window.observe_preferences();
        main_window.check_authentication();
        main_window
    }

    fn prime_favorites(&self) {
        if let Some(manager) = &self.favorites_manager {
            let favorites = self.favorites.clone();
            let manager = manager.clone();

            let (sender, receiver) =
                glib::MainContext::default().channel::<HashSet<i64>>(glib::Priority::default());
            let favorites_clone = favorites.clone();

            receiver.attach(None, move |favorite_ids| {
                let mut favorites_guard = favorites_clone.lock();
                *favorites_guard = favorite_ids;
                glib::ControlFlow::Break
            });

            crate::runtime_handle().spawn(async move {
                let favorite_ids = manager.get_all().await;
                let _ = sender.send(favorite_ids);
            });
        }
    }

    fn build_ui(&self) {
        let header = adw::HeaderBar::new();

        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh repositories"));
        header.pack_start(&refresh_button);

        let preferences_button = gtk::Button::from_icon_name("emblem-system-symbolic");
        preferences_button.set_tooltip_text(Some("Preferences"));
        header.pack_end(&preferences_button);

        let rate_limit_label = self.rate_limit_label.clone();
        rate_limit_label.set_halign(gtk::Align::End);
        let rate_limit_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        rate_limit_box.add_css_class("linked");
        rate_limit_box.append(&rate_limit_label);
        header.pack_end(&rate_limit_box);

        let signout_button = gtk::Button::from_icon_name("system-log-out-symbolic");
        signout_button.set_tooltip_text(Some("Sign out"));
        header.pack_end(&signout_button);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.append(&header);

        let search_entry = self.search_entry.clone();
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();

        let list_box = self.repo_list.clone();
        scrolled.set_child(Some(&list_box));

        let sidebar_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar_box.append(&search_entry);
        sidebar_box.append(&scrolled);

        let detail_status_page = self.detail_status_page.clone();
        detail_status_page.set_vexpand(true);
        detail_status_page.set_hexpand(true);
        detail_status_page.set_margin_top(24);
        detail_status_page.set_margin_bottom(24);
        detail_status_page.set_margin_start(24);
        detail_status_page.set_margin_end(24);

        let detail_stack = self.detail_stack.clone();
        if detail_stack.child_by_name("placeholder").is_none() {
            detail_stack.add_named(&detail_status_page, Some("placeholder"));
        }
        detail_stack.set_visible_child_name("placeholder");

        let split_pane = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .wide_handle(true)
            .start_child(&sidebar_box)
            .end_child(&detail_stack)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .build();

        main_box.append(&split_pane);

        self.window.set_content(Some(&main_box));

        self.connect_refresh_button(&refresh_button);
        self.connect_signout_button(&signout_button);
        self.connect_preferences_button(&preferences_button);
        self.connect_search();
        self.connect_repo_selection();
    }

    fn check_authentication(&self) {
        let storage = TokenStorage::new();

        match storage {
            Ok(storage) => {
                if let Ok(token) = storage.get_token() {
                    info!("Found existing token, initializing client");
                    let client_result = GitHubClient::new(Some(token));

                    match client_result {
                        Ok(client) => {
                            let client_arc = self.client.clone();
                            *client_arc.lock() = Some(client);
                            self.load_repositories();
                        }
                        Err(e) => {
                            error!("Failed to create GitHub client: {}", e);
                            self.show_auth_window();
                        }
                    }
                } else {
                    info!("No token found, showing auth window");
                    self.show_auth_window();
                }
            }
            Err(e) => {
                error!("Failed to access token storage: {}", e);
                self.show_auth_window();
            }
        }
    }

    fn show_auth_window(&self) {
        let auth_window = AuthWindow::new(Some(&self.window));
        let client_arc = self.client.clone();
        let this = self.clone();

        auth_window.present();

        // Reload when main window gets focus back after auth
        self.window.connect_is_active_notify(move |window| {
            if window.is_active() {
                let storage = TokenStorage::new().ok();
                if let Some(storage) = storage {
                    if let Ok(token) = storage.get_token() {
                        info!("Token found after auth, reinitializing client and loading repos");
                        if let Ok(client) = GitHubClient::new(Some(token)) {
                            let client_clone = client_arc.clone();
                            let this = this.clone();

                            {
                                let mut client_guard = client_clone.lock();
                                *client_guard = Some(client.clone());
                            }

                            let (sender, receiver) = glib::MainContext::default().channel::<(
                                Result<Vec<Repo>, GitHubError>,
                                Option<RateLimitInfo>,
                            )>(
                                glib::Priority::default(),
                            );
                            let this_ui = this.clone();

                            receiver.attach(None, move |(repos_result, rate_info)| {
                                match repos_result {
                                    Ok(repos) => {
                                        info!("Loaded {} repositories after auth", repos.len());
                                        this_ui.refresh_repository_view(repos, rate_info);
                                    }
                                    Err(e) => {
                                        error!("Failed to load repositories: {}", e);
                                    }
                                }
                                glib::ControlFlow::Break
                            });

                            crate::runtime_handle().spawn(async move {
                                let repos_result = client.list_repos().await;
                                let rate_info = client.rate_limit_info();
                                let _ = sender.send((repos_result, rate_info));
                            });
                        }
                    }
                }
            }
        });
    }

    fn load_repositories(&self) {
        let client_opt = {
            let guard = self.client.lock();
            guard.clone()
        };

        if let Some(client) = client_opt {
            info!("Starting to load repositories...");

            let (sender, receiver) =
                glib::MainContext::default()
                    .channel::<(Result<Vec<Repo>, GitHubError>, Option<RateLimitInfo>)>(
                        glib::Priority::default(),
                    );
            let this = self.clone();

            receiver.attach(None, move |(repos_result, rate_info)| {
                match repos_result {
                    Ok(repos) => {
                        info!("✅ Loaded {} repositories, updating UI", repos.len());
                        this.refresh_repository_view(repos, rate_info);
                    }
                    Err(e) => {
                        error!("Failed to load repositories: {}", e);
                    }
                }

                glib::ControlFlow::Break
            });

            crate::runtime_handle().spawn(async move {
                let repos_result = client.list_repos().await;
                let rate_info = client.rate_limit_info();

                let _ = sender.send((repos_result, rate_info));
            });
        } else {
            error!("No client available to load repositories");
        }
    }

    fn refresh_repository_view(&self, repos: Vec<Repo>, rate_info: Option<RateLimitInfo>) {
        // Update state synchronously - no async needed with parking_lot
        {
            let mut repos_guard = self.repos.lock();
            *repos_guard = repos.clone();
        }

        {
            let mut actions = self.actions_states.lock();
            actions.retain(|repo_id, _| repos.iter().any(|repo| repo.id == *repo_id));
            for repo in &repos {
                let state = match repo.permissions.as_ref() {
                    Some(perms) if perms.push || perms.admin => RepoActionsState::Enabled,
                    Some(_) => RepoActionsState::Disabled,
                    None => RepoActionsState::Unknown,
                };
                actions.insert(repo.id, state);
            }
        }

        {
            let mut counts = self.workflow_counts.lock();
            counts.retain(|repo_id, _| repos.iter().any(|repo| repo.id == *repo_id));
            for repo in &repos {
                counts.entry(repo.id).or_default();
            }
        }

        {
            let mut info_guard = self.rate_limit_info.lock();
            *info_guard = rate_info.clone();
        }

        self.schedule_repo_list_refresh();
        self.spawn_repo_status_tasks(repos);
        self.update_rate_limit_display(rate_info);
    }

    fn update_rate_limit_display(&self, info: Option<RateLimitInfo>) {
        let label = self.rate_limit_label.clone();
        glib::idle_add_local_once(move || update_rate_limit_label(&label, info.clone()));
    }

    fn schedule_repo_list_refresh(&self) {
        let repos_snapshot = self.repos.lock().clone();
        let favorites_snapshot = self.favorites.lock().clone();
        let actions_snapshot = self.actions_states.lock().clone();
        let workflow_snapshot = self.workflow_counts.lock().clone();
        let selected = *self.selected_repo_id.lock();
        let favorites_arc = self.favorites.clone();
        let favorites_manager = self.favorites_manager.clone();
        let list_box = self.repo_list.clone();

        // Schedule UI update on glib main thread
        glib::idle_add_local_once(move || {
            rebuild_repo_list(
                list_box,
                repos_snapshot,
                favorites_snapshot,
                actions_snapshot,
                workflow_snapshot,
                favorites_arc,
                favorites_manager,
                selected,
            );
        });
    }

    fn spawn_repo_status_tasks(&self, repos: Vec<Repo>) {
        let client_arc = self.client.clone();
        let actions_state = self.actions_states.clone();
        let workflow_state = self.workflow_counts.clone();
        let this = self.clone();

        // Clone client for tokio task
        let client_opt = client_arc.lock().clone();

        if let Some(client) = client_opt {
            let (sender, receiver) =
                glib::MainContext::default().channel::<()>(glib::Priority::default());
            let this_ui = this.clone();

            receiver.attach(None, move |_| {
                this_ui.schedule_repo_list_refresh();
                glib::ControlFlow::Break
            });

            // Run all status checks in parallel on tokio runtime
            crate::runtime_handle().spawn(async move {
                // Process repos in parallel using futures
                use futures::stream::{self, StreamExt};

                stream::iter(repos)
                    .for_each_concurrent(5, |repo| {
                        let client = client.clone();
                        let actions_state = actions_state.clone();
                        let workflow_state = workflow_state.clone();

                        async move {
                            let owner = repo.owner.login.clone();
                            let repo_name = repo.name.clone();
                            let repo_id = repo.id;

                            // Check actions enabled
                            match client.is_actions_enabled(&owner, &repo_name).await {
                                Ok(enabled) => {
                                    let mut actions = actions_state.lock();
                                    actions.insert(
                                        repo_id,
                                        if enabled {
                                            RepoActionsState::Enabled
                                        } else {
                                            RepoActionsState::Disabled
                                        },
                                    );
                                }
                                Err(err) => {
                                    warn!(
                                        "Failed to fetch actions status for {}/{}: {}",
                                        owner, repo_name, err
                                    );
                                }
                            }

                            // Get workflow counts
                            match gather_workflow_status_counts(&client, &owner, &repo_name).await {
                                Ok(counts) => {
                                    let mut workflows = workflow_state.lock();
                                    workflows.insert(repo_id, counts);
                                }
                                Err(err) => {
                                    warn!(
                                        "Failed to fetch workflow status for {}/{}: {}",
                                        owner, repo_name, err
                                    );
                                }
                            }
                        }
                    })
                    .await;

                let _ = sender.send(());
            });
        }
    }

    fn observe_favorites(&self) {
        if let Some(manager) = &self.favorites_manager {
            let receiver = manager.subscribe();
            let favorites_state = self.favorites.clone();
            let this = self.clone();

            let (sender, receiver_channel) =
                glib::MainContext::default().channel::<HashSet<i64>>(glib::Priority::default());

            receiver_channel.attach(None, move |latest| {
                {
                    let mut favorites = favorites_state.lock();
                    *favorites = latest;
                }
                this.schedule_repo_list_refresh();

                glib::ControlFlow::Continue
            });

            crate::runtime_handle().spawn(async move {
                let mut receiver_local = receiver;

                if sender.send(receiver_local.borrow().clone()).is_err() {
                    return;
                }

                loop {
                    if receiver_local.changed().await.is_err() {
                        break;
                    }

                    if sender.send(receiver_local.borrow().clone()).is_err() {
                        break;
                    }
                }
            });
        }
    }

    fn observe_preferences(&self) {
        if let Some(manager) = &self.preferences_manager {
            let mut receiver = manager.subscribe();
            let initial_interval = receiver.borrow().refresh_interval;
            self.configure_auto_refresh(initial_interval);

            let (sender, receiver_channel) =
                glib::MainContext::default().channel::<u64>(glib::Priority::default());
            let this = self.clone();

            receiver_channel.attach(None, move |interval| {
                this.configure_auto_refresh(interval);
                glib::ControlFlow::Continue
            });

            crate::runtime_handle().spawn(async move {
                loop {
                    if receiver.changed().await.is_err() {
                        break;
                    }

                    let interval = receiver.borrow().refresh_interval;
                    if sender.send(interval).is_err() {
                        break;
                    }
                }
            });
        }
    }

    fn configure_auto_refresh(&self, interval_seconds: u64) {
        {
            let mut current = self.current_refresh_interval.lock();
            if *current == interval_seconds {
                return;
            }
            *current = interval_seconds;
        }

        if let Some(handle) = self.auto_refresh_task.lock().take() {
            handle.abort();
        }

        if interval_seconds == 0 {
            return;
        }

        let (sender, receiver) = glib::MainContext::default()
            .channel::<(Result<Vec<Repo>, GitHubError>, Option<RateLimitInfo>)>(
                glib::Priority::default(),
            );
        let this = self.clone();

        receiver.attach(None, move |(repos_result, rate_info)| {
            match repos_result {
                Ok(repos) => this.refresh_repository_view(repos, rate_info),
                Err(err) => {
                    error!("Failed to auto-refresh repositories: {}", err);
                    this.update_rate_limit_display(rate_info);
                }
            }

            glib::ControlFlow::Continue
        });

        let client_arc = self.client.clone();
        let sender_clone = sender.clone();

        let handle = crate::runtime_handle().spawn(async move {
            let sender = sender_clone;

            loop {
                tokio::time::sleep(Duration::from_secs(interval_seconds)).await;

                let client_opt = {
                    let guard = client_arc.lock();
                    guard.clone()
                };

                let Some(client) = client_opt else {
                    continue;
                };

                let repos_result = client.list_repos().await;
                let rate_info = client.rate_limit_info();

                if sender.send((repos_result, rate_info)).is_err() {
                    break;
                }
            }
        });

        *self.auto_refresh_task.lock() = Some(handle);
    }

    fn connect_refresh_button(&self, button: &gtk::Button) {
        let client_arc = self.client.clone();
        let this = self.clone();

        button.connect_clicked(move |_| {
            let client = client_arc.clone();
            let this = this.clone();
            let client_clone = client.lock().clone();

            if let Some(github_client) = client_clone {
                let (sender, receiver) =
                    glib::MainContext::default()
                        .channel::<(Result<Vec<Repo>, GitHubError>, Option<RateLimitInfo>)>(
                            glib::Priority::default(),
                        );
                let this_ui = this.clone();

                receiver.attach(None, move |(repos_result, rate_info)| {
                    match repos_result {
                        Ok(new_repos) => {
                            info!("Refreshed {} repositories", new_repos.len());
                            this_ui.refresh_repository_view(new_repos, rate_info);
                        }
                        Err(e) => {
                            error!("Failed to refresh repositories: {}", e);
                        }
                    }

                    glib::ControlFlow::Break
                });

                crate::runtime_handle().spawn(async move {
                    let repos_result = github_client.list_repos().await;
                    let rate_info = github_client.rate_limit_info();
                    let _ = sender.send((repos_result, rate_info));
                });
            }
        });
    }

    fn connect_signout_button(&self, button: &gtk::Button) {
        let window = self.window.clone();

        button.connect_clicked(move |_| {
            let storage = TokenStorage::new();

            if let Ok(storage) = storage {
                if let Err(e) = storage.delete_token() {
                    error!("Failed to delete token: {}", e);
                } else {
                    info!("Signed out successfully");

                    // Show dialog
                    let dialog = gtk::MessageDialog::new(
                        Some(&window),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Info,
                        gtk::ButtonsType::Ok,
                        "You have been signed out. Restart the application to sign in again.",
                    );
                    dialog.connect_response(|dialog, _| {
                        dialog.close();
                    });
                    dialog.present();
                }
            }
        });
    }

    fn connect_preferences_button(&self, button: &gtk::Button) {
        if let Some(manager) = &self.preferences_manager {
            let parent = self.window.clone();
            let manager = manager.clone();
            button.connect_clicked(move |_| {
                let window = PreferencesWindow::new(&parent, manager.clone());
                window.present();
            });
        } else {
            button.set_sensitive(false);
            button.set_tooltip_text(Some("Preferences unavailable"));
        }
    }

    fn connect_search(&self) {
        let list_box = self.repo_list.clone();

        self.search_entry.connect_search_changed(move |entry| {
            let text = entry.text().to_lowercase();
            let query = text.clone();
            list_box.set_filter_func(move |row: &gtk::ListBoxRow| row_matches_query(row, &query));
            list_box.invalidate_filter();
        });
    }

    fn connect_repo_selection(&self) {
        let window = self.clone();

        self.repo_list
            .connect_selected_rows_changed(move |list_box| {
                let window = window.clone();
                let repo_name = list_box
                    .selected_row()
                    .and_then(|row| row.child())
                    .and_then(|child| find_label_by_name(&child, "repo-name-label"))
                    .map(|label| label.text().to_string());

                let repo = match repo_name {
                    Some(name) => {
                        let repos = window.repos.lock();
                        repos.iter().find(|repo| repo.full_name == name).cloned()
                    }
                    None => None,
                };

                window.handle_repo_selection(repo);
            });
    }

    fn handle_repo_selection(&self, repo: Option<Repo>) {
        match repo {
            Some(repo) => {
                *self.selected_repo_id.lock() = Some(repo.id);
                self.present_repo_detail(repo);
            }
            None => {
                *self.selected_repo_id.lock() = None;
                self.show_detail_placeholder();
            }
        }
    }

    fn present_repo_detail(&self, repo: Repo) {
        let client_opt = {
            let guard = self.client.lock();
            guard.clone()
        };

        match client_opt {
            Some(client) => {
                let pane = RepoDetailPane::new(
                    self.window.clone(),
                    repo,
                    Arc::new(Mutex::new(client)),
                    self.favorites_manager.clone(),
                    self.favorites.clone(),
                );
                let stack = self.detail_stack.clone();
                let active_detail = self.active_detail.clone();

                glib::idle_add_local_once(move || {
                    if let Some(existing_child) = stack.child_by_name("detail") {
                        stack.remove(&existing_child);
                    }

                    let widget = pane.widget();
                    stack.add_named(&widget, Some("detail"));
                    stack.set_visible_child_name("detail");
                    active_detail.borrow_mut().replace(pane);
                });
            }
            None => {
                warn!("Cannot show repository details without an authenticated client");
                self.show_detail_placeholder();
            }
        }
    }

    fn show_detail_placeholder(&self) {
        let stack = self.detail_stack.clone();
        let active_detail = self.active_detail.clone();
        let status_page = self.detail_status_page.clone();

        glib::idle_add_local_once(move || {
            if let Some(detail_child) = stack.child_by_name("detail") {
                stack.remove(&detail_child);
            }
            stack.set_visible_child_name("placeholder");
            active_detail.borrow_mut().take();
        });

        schedule_status_page_update(status_page, None);
    }

    pub fn present(&self) {
        self.window.present();
    }
}
