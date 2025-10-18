/// Background task: Observe favorites changes
use crate::favorites::FavoritesManager;
use gtk4::glib;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

/// Observe favorites manager and update local state
#[allow(dead_code)] // Will be used when fully integrated
pub fn observe_favorites<F>(
    manager: Arc<FavoritesManager>,
    favorites_state: Arc<Mutex<HashSet<i64>>>,
    on_change: F,
) where
    F: Fn() + 'static,
{
    let receiver = manager.subscribe();

    glib::MainContext::default().spawn_local(async move {
        let mut receiver_local = receiver;

        {
            let initial = receiver_local.borrow().clone();
            let mut favorites = favorites_state.lock();
            *favorites = initial;
        }
        on_change();

        loop {
            // Wrap receiver.changed() in tokio spawn since it needs tokio context
            let changed_result = crate::runtime_handle()
                .spawn(async move {
                    let result = receiver_local.changed().await;
                    (receiver_local, result)
                })
                .await
                .unwrap();

            receiver_local = changed_result.0;
            if changed_result.1.is_err() {
                break;
            }

            let latest = receiver_local.borrow().clone();
            {
                let mut favorites = favorites_state.lock();
                *favorites = latest.clone();
            }
            on_change();
        }
    });
}
