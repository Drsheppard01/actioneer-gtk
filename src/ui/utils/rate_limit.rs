/// Utility functions for rate limit display
use crate::api::models::RateLimitInfo;
use chrono::{DateTime, Utc};
use gtk4::{self as gtk};

/// Update rate limit label with current info
pub fn update_rate_limit_label(label: &gtk::Label, info: Option<RateLimitInfo>) {
    if let Some(info) = info {
        let reset_time = DateTime::from_timestamp(info.reset, 0).unwrap_or_else(Utc::now);

        let now = Utc::now();
        let duration = reset_time.signed_duration_since(now);

        let time_str = if duration.num_seconds() < 0 {
            "now".to_string()
        } else if duration.num_minutes() < 1 {
            format!("{}s", duration.num_seconds())
        } else if duration.num_hours() < 1 {
            format!("{}m", duration.num_minutes())
        } else {
            format!("{}h", duration.num_hours())
        };

        label.set_text(&format!(
            "Rate limit: {}/{} (resets in {})",
            info.remaining, info.limit, time_str
        ));
    } else {
        label.set_text("Rate limit: –");
    }
}
