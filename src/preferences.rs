use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Will be used when integrated with UI
pub struct Preferences {
    /// Auto-refresh interval in seconds (0 = disabled)
    pub refresh_interval: u64,

    /// Last selected repository ID
    pub last_selected_repo_id: Option<i64>,

    /// Window width
    pub window_width: i32,

    /// Window height  
    pub window_height: i32,

    /// Show notifications
    pub enable_notifications: bool,

    /// Play sound on workflow completion
    pub enable_sounds: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            refresh_interval: 0, // Disabled by default
            last_selected_repo_id: None,
            window_width: 1000,
            window_height: 700,
            enable_notifications: true,
            enable_sounds: true,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)] // Will be used when integrated with UI
pub struct PreferencesManager {
    prefs: Arc<RwLock<Preferences>>,
    config_path: PathBuf,
}

#[allow(dead_code)] // Will be used when integrated with UI
impl PreferencesManager {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("actioneer");

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("preferences.json");
        let prefs = if config_path.exists() {
            let data = fs::read_to_string(&config_path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Preferences::default()
        };

        Ok(Self {
            prefs: Arc::new(RwLock::new(prefs)),
            config_path,
        })
    }

    pub async fn get(&self) -> Preferences {
        self.prefs.read().await.clone()
    }

    pub async fn update<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut Preferences),
    {
        let mut prefs = self.prefs.write().await;
        f(&mut prefs);
        self.save(&prefs)?;
        Ok(())
    }

    pub async fn set_refresh_interval(&self, seconds: u64) -> anyhow::Result<()> {
        self.update(|p| p.refresh_interval = seconds).await
    }

    pub async fn set_last_selected_repo(&self, repo_id: Option<i64>) -> anyhow::Result<()> {
        self.update(|p| p.last_selected_repo_id = repo_id).await
    }

    pub async fn set_window_size(&self, width: i32, height: i32) -> anyhow::Result<()> {
        self.update(|p| {
            p.window_width = width;
            p.window_height = height;
        })
        .await
    }

    pub async fn set_notifications_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        self.update(|p| p.enable_notifications = enabled).await
    }

    pub async fn set_sounds_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        self.update(|p| p.enable_sounds = enabled).await
    }

    fn save(&self, prefs: &Preferences) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(prefs)?;
        fs::write(&self.config_path, data)?;
        Ok(())
    }
}

impl Default for PreferencesManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PreferencesManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preferences_default() {
        let prefs = Preferences::default();
        assert_eq!(prefs.refresh_interval, 0);
        assert_eq!(prefs.window_width, 1000);
        assert!(prefs.enable_notifications);
    }

    #[tokio::test]
    async fn test_preferences_update() {
        let manager = PreferencesManager::new().unwrap();

        manager.set_refresh_interval(30).await.unwrap();

        let prefs = manager.get().await;
        assert_eq!(prefs.refresh_interval, 30);
    }
}
