// crates/server/src/shutdown.rs

use tokio::sync::{oneshot, broadcast};

/// Shutdown manager for graceful shutdown
pub struct ShutdownManager {
    tx: broadcast::Sender<()>,
}

impl ShutdownManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self { tx }
    }

    pub fn subscribe(&self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        let mut rx_broadcast = self.tx.subscribe();
        tokio::spawn(async move {
            let _ = rx_broadcast.recv().await;
            let _ = tx.send(());
        });
        rx
    }

    pub fn wait(&self) -> oneshot::Receiver<()> {
        self.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(());
    }
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}
