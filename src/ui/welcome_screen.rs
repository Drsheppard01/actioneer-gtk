use gtk4::prelude::*;
use gtk4::{self as gtk, glib};
use libadwaita as adw;

pub struct WelcomeScreen {
    widget: gtk::Box,
}

impl WelcomeScreen {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_valign(gtk::Align::Center);
        widget.set_halign(gtk::Align::Center);
        widget.set_vexpand(true);
        widget.set_hexpand(true);

        // App icon placeholder (you can replace with actual icon later)
        let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        icon_box.set_halign(gtk::Align::Center);
        icon_box.set_margin_bottom(24);

        let icon = gtk::Image::from_icon_name("system-run-symbolic");
        icon.set_pixel_size(128);
        icon_box.append(&icon);
        widget.append(&icon_box);

        // Welcome title
        let title = gtk::Label::new(Some("Welcome to Actioneer"));
        title.add_css_class("title-1");
        title.set_margin_bottom(12);
        widget.append(&title);

        // Subtitle
        let subtitle = gtk::Label::new(Some("Manage your GitHub Actions workflows with ease"));
        subtitle.add_css_class("dim-label");
        subtitle.set_margin_bottom(36);
        widget.append(&subtitle);

        // Features list
        let features_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        features_box.set_halign(gtk::Align::Center);
        features_box.set_margin_bottom(48);

        Self::add_feature(
            &features_box,
            "media-playback-start-symbolic",
            "Trigger workflows instantly",
            "success",
        );
        Self::add_feature(
            &features_box,
            "view-reveal-symbolic",
            "Monitor runs in real-time",
            "accent",
        );
        Self::add_feature(
            &features_box,
            "folder-documents-symbolic",
            "View detailed logs",
            "warning",
        );

        widget.append(&features_box);

        // Buttons
        let buttons_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        buttons_box.set_halign(gtk::Align::Center);
        buttons_box.set_width_request(300);

        let signin_button = gtk::Button::with_label("Sign in with GitHub");
        signin_button.add_css_class("suggested-action");
        signin_button.add_css_class("pill");
        signin_button.set_icon_name("avatar-default-symbolic");
        signin_button.set_widget_name("welcome-signin-button");
        buttons_box.append(&signin_button);

        let demo_button = gtk::Button::with_label("Try Demo Mode");
        demo_button.add_css_class("pill");
        demo_button.set_widget_name("welcome-demo-button");
        demo_button.set_sensitive(false); // Disabled for now as requested
        demo_button.set_tooltip_text(Some("Demo mode coming soon"));
        buttons_box.append(&demo_button);

        widget.append(&buttons_box);

        Self { widget }
    }

    fn add_feature(container: &gtk::Box, icon_name: &str, text: &str, css_class: &str) {
        let feature_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        feature_box.set_halign(gtk::Align::Start);

        let icon = gtk::Image::from_icon_name(icon_name);
        icon.add_css_class(css_class);
        icon.set_pixel_size(24);
        feature_box.append(&icon);

        let label = gtk::Label::new(Some(text));
        label.set_halign(gtk::Align::Start);
        feature_box.append(&label);

        container.append(&feature_box);
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn connect_signin<F: Fn() + 'static>(&self, callback: F) {
        if let Some(button) = self
            .widget
            .first_child()
            .and_then(|w| Self::find_widget_by_name(&w, "welcome-signin-button"))
        {
            if let Ok(btn) = button.downcast::<gtk::Button>() {
                btn.connect_clicked(move |_| callback());
            }
        }
    }

    fn find_widget_by_name(widget: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
        if widget.widget_name() == name {
            return Some(widget.clone());
        }

        let mut child = widget.first_child();
        while let Some(w) = child {
            if let Some(found) = Self::find_widget_by_name(&w, name) {
                return Some(found);
            }
            child = w.next_sibling();
        }
        None
    }
}
