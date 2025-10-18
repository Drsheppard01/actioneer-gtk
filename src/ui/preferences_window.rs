use crate::preferences::PreferencesManager;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::sync::Arc;
use tracing::warn;

pub struct PreferencesWindow {
    window: adw::PreferencesWindow,
    manager: Arc<PreferencesManager>,
}

impl PreferencesWindow {
    pub fn new(parent: &adw::ApplicationWindow, manager: Arc<PreferencesManager>) -> Self {
        let window = adw::PreferencesWindow::builder()
            .title("Preferences")
            .transient_for(parent)
            .modal(true)
            .default_width(420)
            .default_height(360)
            .build();

        let general_page = adw::PreferencesPage::new();

        let refresh_group = adw::PreferencesGroup::builder().title("Refresh").build();

        let refresh_row = adw::ActionRow::builder()
            .title("Auto Refresh Interval")
            .subtitle("Minutes between automatic refreshes (0 disables)")
            .build();

        let refresh_spin = gtk::SpinButton::with_range(0.0, 120.0, 1.0);
        refresh_spin.set_width_chars(4);
        refresh_spin.set_hexpand(false);
        refresh_spin.set_vexpand(false);
        refresh_spin.set_halign(gtk::Align::End);
        refresh_spin.set_valign(gtk::Align::Center);
        refresh_row.add_suffix(&refresh_spin);
        refresh_row.set_activatable_widget(Some(&refresh_spin));
        refresh_group.add(&refresh_row);

        let notifications_group = adw::PreferencesGroup::builder()
            .title("Notifications")
            .build();

        let notify_row = adw::ActionRow::builder()
            .title("Desktop Notifications")
            .subtitle("Show a notification when a workflow run finishes")
            .build();
        let notify_switch = gtk::Switch::new();
        notify_switch.set_hexpand(false);
        notify_switch.set_vexpand(false);
        notify_switch.set_halign(gtk::Align::End);
        notify_switch.set_valign(gtk::Align::Center);
        notify_row.add_suffix(&notify_switch);
        notify_row.set_activatable_widget(Some(&notify_switch));
        notifications_group.add(&notify_row);

        let sounds_row = adw::ActionRow::builder()
            .title("Play Sound")
            .subtitle("Play an alert when a workflow fails")
            .build();
        let sounds_switch = gtk::Switch::new();
        sounds_switch.set_hexpand(false);
        sounds_switch.set_vexpand(false);
        sounds_switch.set_halign(gtk::Align::End);
        sounds_switch.set_valign(gtk::Align::Center);
        sounds_row.add_suffix(&sounds_switch);
        sounds_row.set_activatable_widget(Some(&sounds_switch));
        notifications_group.add(&sounds_row);

        general_page.add(&refresh_group);
        general_page.add(&notifications_group);
        window.add(&general_page);

        let manager_clone = manager.clone();
        let spin_clone = refresh_spin.clone();
        let notify_clone = notify_switch.clone();
        let sounds_clone = sounds_switch.clone();
        glib::MainContext::default().spawn_local(async move {
            let prefs = manager_clone.get().await;
            spin_clone.set_value((prefs.refresh_interval as f64) / 60.0);
            notify_clone.set_active(prefs.enable_notifications);
            sounds_clone.set_active(prefs.enable_sounds);
        });

        let manager_for_spin = manager.clone();
        refresh_spin.connect_value_changed(move |spin| {
            let minutes = spin.value().max(0.0);
            let manager = manager_for_spin.clone();
            glib::MainContext::default().spawn_local(async move {
                if let Err(err) = manager
                    .set_refresh_interval((minutes.round() as u64) * 60)
                    .await
                {
                    warn!("Failed to save refresh interval: {}", err);
                }
            });
        });

        let manager_for_notify = manager.clone();
        notify_switch.connect_state_set(move |_, state| {
            let manager = manager_for_notify.clone();
            glib::MainContext::default().spawn_local(async move {
                if let Err(err) = manager.set_notifications_enabled(state).await {
                    warn!("Failed to update notifications preference: {}", err);
                }
            });
            Propagation::Proceed
        });

        let manager_for_sounds = manager.clone();
        sounds_switch.connect_state_set(move |_, state| {
            let manager = manager_for_sounds.clone();
            glib::MainContext::default().spawn_local(async move {
                if let Err(err) = manager.set_sounds_enabled(state).await {
                    warn!("Failed to update sound preference: {}", err);
                }
            });
            Propagation::Proceed
        });

        Self { window, manager }
    }

    pub fn present(&self) {
        self.window.present();
    }

    #[allow(dead_code)]
    pub fn manager(&self) -> Arc<PreferencesManager> {
        self.manager.clone()
    }
}
