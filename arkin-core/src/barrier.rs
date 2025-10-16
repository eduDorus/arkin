use std::{
    sync::{atomic::AtomicU8, Arc},
    time::Duration,
};

use tokio::sync::Notify;

pub struct SyncBarrier {
    ingestor_barrier: Arc<Notify>,
    pubsub_barrier: Arc<Notify>,
    parties: usize,
    counter: AtomicU8,
    window_duration: Duration,
}

impl SyncBarrier {
    pub fn new(parties: usize, window_duration: Duration) -> Self {
        Self {
            ingestor_barrier: Arc::new(Notify::new()),
            pubsub_barrier: Arc::new(Notify::new()),
            parties,
            counter: AtomicU8::new(0),
            window_duration,
        }
    }

    pub async fn ingestor_confirm_and_wait(&self) {
        self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.counter.load(std::sync::atomic::Ordering::SeqCst) as usize >= self.parties {
            self.pubsub_barrier.notify_waiters();
        }
        self.ingestor_barrier.notified().await;
    }

    pub async fn pubsub_confirm_and_wait(&self) {
        self.pubsub_barrier.notified().await;
    }

    pub async fn release_ingestors(&self) {
        self.counter.store(0, std::sync::atomic::Ordering::SeqCst);
        self.ingestor_barrier.notify_waiters();
    }

    pub fn window_duration(&self) -> Duration {
        self.window_duration
    }
}
