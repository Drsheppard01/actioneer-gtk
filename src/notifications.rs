use notify_rust::{Notification, Timeout, Urgency};
use tokio::task;
use tracing::{debug, error, info};

/// Notification manager for Linux using XDG Desktop Notifications
/// Similar to NotificationManager.swift in macOS version
#[derive(Clone)]
#[allow(dead_code)] // Will be used when integrated with UI
pub struct NotificationManager {
    app_name: String,
}

#[allow(dead_code)] // Will be used when integrated with UI
impl NotificationManager {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    /// Send a notification for completed workflow
    pub async fn notify_workflow_completed(
        &self,
        workflow_name: &str,
        run_title: &str,
        _status: &str,
        conclusion: Option<&str>,
    ) -> anyhow::Result<()> {
        let summary = format!("Workflow Completed: {}", workflow_name);
        let body = format!("{} - {}", run_title, self.conclusion_text(conclusion));

        // Determine urgency based on conclusion
        let urgency = if conclusion == Some("failure") { 2 } else { 1 }; // 0=low, 1=normal, 2=critical

        info!(
            workflow = workflow_name,
            run = run_title,
            conclusion = conclusion.unwrap_or("unknown"),
            "Dispatching workflow completion notification"
        );

        self.send_notification(&summary, &body, urgency).await?;

        debug!("Notification sent successfully");

        Ok(())
    }

    /// Send a generic notification
    async fn send_notification(
        &self,
        summary: &str,
        body: &str,
        urgency: u8,
    ) -> anyhow::Result<()> {
        let summary = summary.to_string();
        let body = body.to_string();
        let app_name = self.app_name.clone();

        task::spawn_blocking(move || {
            let urgency_level = match urgency {
                0 => Urgency::Low,
                2 => Urgency::Critical,
                _ => Urgency::Normal,
            };

            debug!(
                "Sending notification: {} - {} (urgency: {:?})",
                summary, body, urgency_level
            );

            let mut notification = Notification::new();
            notification
                .summary(&summary)
                .body(&body)
                .appname(&app_name)
                .urgency(urgency_level)
                .timeout(Timeout::Milliseconds(8000));

            if let Err(err) = notification.show() {
                error!("Failed to show desktop notification: {}", err);
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to dispatch notification: {}", e))
    }

    fn conclusion_text(&self, conclusion: Option<&str>) -> String {
        match conclusion {
            Some("success") => "Success ✓".to_string(),
            Some("failure") => "Failed ✗".to_string(),
            Some("cancelled") => "Cancelled".to_string(),
            Some("skipped") => "Skipped".to_string(),
            Some("timed_out") => "Timed Out".to_string(),
            Some("action_required") => "Action Required".to_string(),
            Some("neutral") => "Neutral".to_string(),
            Some(other) => other
                .replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
            None => "Completed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conclusion_text() {
        let manager = NotificationManager::new("test");

        assert_eq!(manager.conclusion_text(Some("success")), "Success ✓");
        assert_eq!(manager.conclusion_text(Some("failure")), "Failed ✗");
        assert_eq!(manager.conclusion_text(Some("cancelled")), "Cancelled");
        assert_eq!(manager.conclusion_text(None), "Completed");
    }
}
