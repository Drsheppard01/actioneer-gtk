use super::detail_placeholder::schedule_status_page_update;
use super::detail_view::RepoDetailPane;
use super::sidebar::{find_label_by_name, rebuild_repo_list, row_matches_query};
use super::WelcomeScreen;
use crate::api::models::{RateLimitInfo, Repo};
use crate::api::{GitHubClient, GitHubError};
use crate::cache::DataCache;
use crate::favorites::FavoritesManager;
use crate::notifications::NotificationManager;
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
use std::time::Instant;
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
    refresh_button: gtk::Button,
    header_bar: adw::HeaderBar,
    header_spinner: Rc<RefCell<Option<gtk::Spinner>>>,
    favorites_manager: Option<Arc<FavoritesManager>>,
    favorites: Arc<Mutex<HashSet<i64>>>,
    cache: Arc<DataCache>,
    actions_states: Arc<Mutex<HashMap<i64, RepoActionsState>>>,
    actions_checked_at: Arc<Mutex<HashMap<i64, Instant>>>,
    workflow_counts: Arc<Mutex<HashMap<i64, WorkflowStatusCounts>>>,
    preferences_manager: Option<Arc<PreferencesManager>>,
    selected_repo_id: Arc<Mutex<Option<i64>>>,
    rate_limit_info: Arc<Mutex<Option<RateLimitInfo>>>,
    detail_status_page: adw::StatusPage,
    detail_stack: gtk::Stack,
    root_stack: gtk::Stack,
    active_detail: Rc<RefCell<Option<RepoDetailPane>>>,
    background_refresh_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    handling_selection: Arc<Mutex<bool>>,
    notification_manager: Option<NotificationManager>,
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

        let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_button.set_tooltip_text(Some("Refresh repositories"));

        let header_bar = adw::HeaderBar::new();

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
        let root_stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::Crossfade)
            .transition_duration(200)
            .build();
        root_stack.set_hexpand(true);
        root_stack.set_vexpand(true);
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
        let actions_checked_at = Arc::new(Mutex::new(HashMap::new()));
        let workflow_counts = Arc::new(Mutex::new(HashMap::new()));
        let selected_repo_id = Arc::new(Mutex::new(None));
        let rate_limit_info = Arc::new(Mutex::new(None));
        let background_refresh_task = Arc::new(Mutex::new(None));
        let handling_selection = Arc::new(Mutex::new(false));
        let header_spinner = Rc::new(RefCell::new(None));
        let notification_manager = Some(NotificationManager::new("me.spaceinbox.actioneer"));

        let main_window = Self {
            window: window.clone(),
            client: client.clone(),
            repos: repos.clone(),
            repo_list: repo_list.clone(),
            search_entry: search_entry.clone(),
            rate_limit_label: rate_limit_label.clone(),
            refresh_button: refresh_button.clone(),
            header_bar: header_bar.clone(),
            header_spinner: header_spinner.clone(),
            favorites_manager: favorites_manager.clone(),
            favorites: favorites.clone(),
            cache: Arc::new(DataCache::new()),
            actions_states: actions_states.clone(),
            actions_checked_at: actions_checked_at.clone(),
            workflow_counts: workflow_counts.clone(),
            preferences_manager: preferences_manager.clone(),
            selected_repo_id: selected_repo_id.clone(),
            rate_limit_info: rate_limit_info.clone(),
            detail_status_page: detail_status_page.clone(),
            detail_stack: detail_stack.clone(),
            root_stack: root_stack.clone(),
            active_detail: active_detail.clone(),
            background_refresh_task: background_refresh_task.clone(),
            handling_selection: handling_selection.clone(),
            notification_manager: notification_manager.clone(),
        };

        main_window.build_ui();
        main_window.prime_favorites();
        main_window.observe_favorites();
        main_window.setup_focus_handler();
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
        let header = self.header_bar.clone();
        let root_stack = self.root_stack.clone();

        let refresh_button = self.refresh_button.clone();
        header.pack_start(&refresh_button);

        let notification_button = gtk::Button::from_icon_name("dialog-information-symbolic");
        notification_button.add_css_class("flat");
        notification_button.set_tooltip_text(Some("Send test notification"));
        header.pack_start(&notification_button);

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

        let sidebar_clamp = adw::ClampScrollable::new();
        sidebar_clamp.set_maximum_size(420);
        sidebar_clamp.set_hexpand(false);
        // Keep the sidebar width consistent once content loads so the detail pane has room.
        sidebar_clamp.set_child(Some(&sidebar_box));

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
            .start_child(&sidebar_clamp)
            .end_child(&detail_stack)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .build();
        // Keep the sidebar at its natural width and let the detail pane use remaining space.
        split_pane.set_resize_start_child(false);
        split_pane.set_resize_end_child(true);
        split_pane.set_position(360);

        main_box.append(&split_pane);

        root_stack.add_named(&main_box, Some("app"));

        let welcome_screen = WelcomeScreen::new();
        let welcome_widget = welcome_screen.widget();
        welcome_widget.set_margin_top(48);
        welcome_widget.set_margin_bottom(48);
        welcome_widget.set_margin_start(48);
        welcome_widget.set_margin_end(48);
        root_stack.add_named(welcome_widget, Some("welcome"));
        root_stack.set_visible_child_name("welcome");

        self.window.set_content(Some(&root_stack));

        let this = self.clone();
        welcome_screen.connect_signin(move || {
            this.show_auth_window();
        });

        let window_for_quit = self.window.clone();
        welcome_screen.connect_quit(move || {
            if let Some(app) = window_for_quit.application() {
                app.quit();
            } else {
                window_for_quit.close();
            }
        });

        self.connect_refresh_button(&refresh_button);
        self.connect_notification_test_button(&notification_button);
        self.connect_signout_button(&signout_button);
        self.connect_preferences_button(&preferences_button);
        self.connect_search();
        self.connect_repo_selection();
    }

    fn connect_notification_test_button(&self, button: &gtk::Button) {
        match self.notification_manager.clone() {
            Some(manager) => {
                button.connect_clicked(move |_| {
                    let manager = manager.clone();
                    crate::runtime_handle().spawn(async move {
                        if let Err(err) = manager
                            .notify_message(
                                "Actioneer notification test",
                                "If you can read this, GNOME notifications are working.",
                            )
                            .await
                        {
                            warn!("Failed to dispatch test notification: {}", err);
                        }
                    });
                });
            }
            None => {
                button.set_sensitive(false);
                button.set_tooltip_text(Some("Notifications unavailable"));
            }
        }
    }

    fn check_authentication(&self) {
        match TokenStorage::new() {
            Ok(storage) => match storage.get_token() {
                Ok(token) => {
                    info!("Found existing token, initializing client");
                    if !self.initialize_client(token) {
                        self.enter_signed_out_state();
                    }
                }
                Err(_) => {
                    info!("No token found, presenting welcome screen");
                    self.enter_signed_out_state();
                }
            },
            Err(e) => {
                error!("Failed to access token storage: {}", e);
                self.enter_signed_out_state();
            }
        }
    }

    fn show_auth_window(&self) {
        let auth_window = AuthWindow::new();
        auth_window.present(Some(&self.window));
    }

    fn initialize_client(&self, token: String) -> bool {
        match GitHubClient::new(Some(token)) {
            Ok(client) => {
                {
                    let mut client_guard = self.client.lock();
                    *client_guard = Some(client);
                }

                {
                    let mut info_guard = self.rate_limit_info.lock();
                    *info_guard = None;
                }

                self.update_rate_limit_display(None);
                self.show_authenticated_ui();
                self.load_repositories();
                true
            }
            Err(e) => {
                error!("Failed to create GitHub client: {}", e);
                false
            }
        }
    }

    fn show_authenticated_ui(&self) {
        let stack = self.root_stack.clone();
        glib::idle_add_local_once(move || {
            stack.set_visible_child_name("app");
        });
    }

    fn enter_signed_out_state(&self) {
        info!("Switching to signed-out state");
        self.stop_background_refresh();

        {
            let mut handling = self.handling_selection.lock();
            *handling = false;
        }

        {
            let mut selected = self.selected_repo_id.lock();
            *selected = None;
        }

        {
            let mut client_guard = self.client.lock();
            *client_guard = None;
        }

        {
            let mut repos_guard = self.repos.lock();
            repos_guard.clear();
        }

        self.actions_states.lock().clear();
        self.actions_checked_at.lock().clear();
        self.workflow_counts.lock().clear();

        {
            let mut info_guard = self.rate_limit_info.lock();
            *info_guard = None;
        }

        self.schedule_repo_list_refresh();
        self.show_detail_placeholder();
        self.update_rate_limit_display(None);
        self.show_header_loading(false);

        let list_box = self.repo_list.clone();
        glib::idle_add_local_once(move || {
            list_box.unselect_all();
        });

        let search_entry = self.search_entry.clone();
        glib::idle_add_local_once(move || {
            search_entry.set_text("");
        });

        let stack = self.root_stack.clone();
        glib::idle_add_local_once(move || {
            stack.set_visible_child_name("welcome");
        });
    }

    fn setup_focus_handler(&self) {
        let this = self.clone();
        self.window.connect_is_active_notify(move |window| {
            if !window.is_active() {
                return;
            }

            let storage = match TokenStorage::new() {
                Ok(storage) => storage,
                Err(err) => {
                    error!("Failed to access token storage during focus check: {}", err);
                    this.enter_signed_out_state();
                    return;
                }
            };

            match storage.get_token() {
                Ok(token) => {
                    let needs_client = this.client.lock().is_none();
                    if needs_client {
                        info!("Token available after auth, initializing client");
                        if !this.initialize_client(token) {
                            this.enter_signed_out_state();
                        }
                    }
                }
                Err(_) => {
                    let had_client = {
                        let mut guard = this.client.lock();
                        let had = guard.is_some();
                        *guard = None;
                        had
                    };

                    if had_client {
                        info!("Token missing after focus, returning to welcome screen");
                    }

                    this.enter_signed_out_state();
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

            // Show spinner in header
            self.show_header_loading(true);

            let (sender, receiver) =
                glib::MainContext::default()
                    .channel::<(Result<Vec<Repo>, GitHubError>, Option<RateLimitInfo>)>(
                        glib::Priority::default(),
                    );
            let this = self.clone();

            receiver.attach(None, move |(repos_result, rate_info)| {
                // Hide spinner and show refresh button
                this.show_header_loading(false);

                match repos_result {
                    Ok(repos) => {
                        info!("✅ Loaded {} repositories, updating UI", repos.len());
                        this.refresh_repository_view(repos, rate_info);
                    }
                    Err(e) => {
                        error!("Failed to load repositories: {}", e);
                        // Still update rate limit even on error
                        if let Some(info) = rate_info {
                            this.update_rate_limit_display(Some(info));
                        }
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
                actions.entry(repo.id).or_insert(state);
            }
        }

        {
            let mut checked = self.actions_checked_at.lock();
            checked.retain(|repo_id, _| repos.iter().any(|repo| repo.id == *repo_id));
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
        let parent = self.window.clone();
        let this = self.clone();

        button.connect_clicked(move |_| {
            let dialog = gtk::MessageDialog::new(
                Some(&parent),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Warning,
                gtk::ButtonsType::YesNo,
                "Are you sure you want to sign out?\n\nYou will need to sign in again to continue.",
            );

            let this_inner = this.clone();

            dialog.connect_response(move |dialog, response| {
                dialog.close();

                if response == gtk::ResponseType::Yes {
                    match TokenStorage::new() {
                        Ok(storage) => {
                            if let Err(err) = storage.delete_token() {
                                error!("Failed to delete token: {}", err);
                            } else {
                                info!("Signed out successfully");
                                this_inner.enter_signed_out_state();
                            }
                        }
                        Err(err) => {
                            error!("Failed to access token storage: {}", err);
                        }
                    }
                }
            });

            dialog.present();
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

                // Check if selection actually changed
                let current_selection = *window.selected_repo_id.lock();
                let new_selection = repo.as_ref().map(|r| r.id);

                // Check if we're already handling a selection
                if *window.handling_selection.lock() {
                    info!("Already handling selection, ignoring signal");
                    return;
                }

                info!(
                    "Selection signal: current={:?}, new={:?}, repo={:?}",
                    current_selection,
                    new_selection,
                    repo.as_ref().map(|r| r.full_name.as_str())
                );

                // Ignore transient deselection events if we have an active detail pane
                // This happens during widget manipulation (stack remove/add)
                if new_selection.is_none() && window.active_detail.borrow().is_some() {
                    info!("Ignoring transient deselection (detail pane is active)");
                    return;
                }

                if current_selection != new_selection {
                    window.handle_repo_selection(repo);
                }
            });
    }

    fn handle_repo_selection(&self, repo: Option<Repo>) {
        *self.handling_selection.lock() = true;

        match repo {
            Some(repo) => {
                info!("Handling repo selection: {}", repo.full_name);
                *self.selected_repo_id.lock() = Some(repo.id);
                self.start_background_refresh(repo.clone());
                self.present_repo_detail(repo);
            }
            None => {
                info!("Deselecting repo");
                *self.selected_repo_id.lock() = None;
                self.stop_background_refresh();
                self.show_detail_placeholder();
            }
        }

        *self.handling_selection.lock() = false;
    }

    fn present_repo_detail(&self, repo: Repo) {
        info!("Creating detail pane for: {}", repo.full_name);

        // Check if we already have a pane for this repo to avoid recreating
        {
            let active = self.active_detail.borrow();
            if let Some(existing_pane) = active.as_ref() {
                if existing_pane.repo().id == repo.id {
                    info!("Pane already exists for this repo, skipping creation");
                    return;
                }
            }
        }

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
                    self.preferences_manager.clone(),
                    self.cache.clone(),
                    self.favorites.clone(),
                    self.notification_manager.clone(),
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

    fn start_background_refresh(&self, _repo: Repo) {
        // Stop any existing refresh task
        self.stop_background_refresh();

        let preferences_manager = match &self.preferences_manager {
            Some(manager) => manager.clone(),
            None => return,
        };

        let client_arc = self.client.clone();
        let active_detail = self.active_detail.clone();
        let rate_limit_label = self.rate_limit_label.clone();

        let (sender, receiver) =
            glib::MainContext::default().channel::<()>(glib::Priority::default());

        receiver.attach(None, move |_| {
            // Trigger a refresh on the active detail pane
            if let Some(pane) = active_detail.borrow().as_ref() {
                pane.refresh_workflows_silent();
            }
            glib::ControlFlow::Continue
        });

        let (rate_sender, rate_receiver) =
            glib::MainContext::default().channel::<RateLimitInfo>(glib::Priority::default());

        let rate_label_clone = rate_limit_label.clone();
        rate_receiver.attach(None, move |info| {
            update_rate_limit_label(&rate_label_clone, Some(info));
            glib::ControlFlow::Continue
        });

        let handle = crate::runtime_handle().spawn(async move {
            loop {
                // Get the current refresh interval
                let interval = preferences_manager.get().await.refresh_interval;

                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

                // Check if we still have a client
                let client_opt = {
                    let guard = client_arc.lock();
                    guard.clone()
                };

                if let Some(client) = client_opt {
                    // Signal the UI to refresh
                    if sender.send(()).is_err() {
                        break;
                    }

                    // Update rate limit display
                    if let Some(rate_info) = client.rate_limit_info() {
                        if rate_sender.send(rate_info).is_err() {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        *self.background_refresh_task.lock() = Some(handle);
    }

    fn stop_background_refresh(&self) {
        if let Some(handle) = self.background_refresh_task.lock().take() {
            handle.abort();
        }
    }

    fn show_header_loading(&self, loading: bool) {
        let header = self.header_bar.clone();
        let refresh_button = self.refresh_button.clone();
        let spinner_ref = self.header_spinner.clone();

        glib::idle_add_local_once(move || {
            if loading {
                info!("🔄 Showing header loading spinner");
                // Hide refresh button
                refresh_button.set_visible(false);

                // Remove any existing spinner
                if let Some(old_spinner) = spinner_ref.borrow_mut().take() {
                    header.remove(&old_spinner);
                }

                // Create and add new spinner
                let spinner = gtk::Spinner::new();
                spinner.start(); // Start animation
                spinner.set_size_request(24, 24);
                spinner.set_tooltip_text(Some("Loading repositories..."));
                header.pack_start(&spinner);
                spinner.set_visible(true); // Ensure visible

                // Store reference
                *spinner_ref.borrow_mut() = Some(spinner);
            } else {
                info!("✅ Hiding header loading spinner");
                // Remove spinner if it exists
                if let Some(spinner) = spinner_ref.borrow_mut().take() {
                    info!("Removing spinner from header");
                    header.remove(&spinner);
                } else {
                    warn!("No header spinner found to remove!");
                }

                // Show refresh button
                refresh_button.set_visible(true);
            }
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}
