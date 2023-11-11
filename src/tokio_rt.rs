use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::Notify;
/// The Service State handles watching for a shutdown signal.
///
/// This is the Tokio Implementation.
///
/// Ordering is Relaxed because the only time it is changed is when the service is shutting down.
#[derive(Debug)]
pub struct ServiceState {
    pub notify: Notify,
    pub running: AtomicBool,
}
impl ServiceState {
    /// Creates a new Service State.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            notify: Notify::new(),
            running: AtomicBool::new(true),
        })
    }
    /// Starts a task that watches for a shutdown signal.
    pub fn watch_for_shutdown(this: Arc<Self>) {
        tokio::spawn(async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!("Failed to watch for shutdown: {}", e);
            }
            this.running.store(false, Ordering::Relaxed);
            this.notify.notify_waiters();
        });
    }
    /// Checks if the service is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    /// Shuts down the service.
    pub fn shutdown(&self) {
        self.notify.notify_waiters();

        self.running.store(false, Ordering::Relaxed);
    }
}
