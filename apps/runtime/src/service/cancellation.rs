//! Receiver-only service cancellation.

use tokio::sync::watch;

/// A receiver-only cooperative cancellation handle for one Runtime service.
#[derive(Debug, Clone)]
pub struct Cancellation {
    receiver: watch::Receiver<bool>,
}

impl Cancellation {
    /// Returns whether the Runtime has requested cancellation.
    #[must_use]
    pub fn is_requested(&self) -> bool {
        *self.receiver.borrow()
    }

    /// Waits until cancellation is requested or the Runtime owner is dropped.
    pub async fn cancelled(&mut self) {
        if self.is_requested() {
            return;
        }

        while self.receiver.changed().await.is_ok() {
            if self.is_requested() {
                return;
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct CancellationSource {
    sender: watch::Sender<bool>,
}

impl CancellationSource {
    pub(super) fn new() -> (Self, Cancellation) {
        let (sender, receiver) = watch::channel(false);
        (Self { sender }, Cancellation { receiver })
    }

    pub(super) fn subscribe(&self) -> Cancellation {
        Cancellation {
            receiver: self.sender.subscribe(),
        }
    }

    pub(super) fn request(&self) {
        self.sender.send_replace(true);
    }
}

#[cfg(test)]
mod tests {
    use super::CancellationSource;

    #[tokio::test]
    async fn cancellation_is_receiver_only_and_idempotent() {
        let (source, mut cancellation) = CancellationSource::new();

        assert!(!cancellation.is_requested());
        source.request();
        source.request();
        cancellation.cancelled().await;

        assert!(cancellation.is_requested());
    }
}
