use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use tokio::sync::Notify;

#[derive(Clone, Debug, Default)]
pub(crate) struct ActivationReviewNotifier {
    waiters: Arc<Mutex<ActivationReviewWaiters>>,
}

#[derive(Debug, Default)]
struct ActivationReviewWaiters {
    next_id: u64,
    by_review: HashMap<String, HashMap<u64, Weak<Notify>>>,
}

#[derive(Debug)]
pub(crate) struct ActivationReviewWaiter {
    id: u64,
    review_id: String,
    notify: Arc<Notify>,
    waiters: Weak<Mutex<ActivationReviewWaiters>>,
}

impl ActivationReviewNotifier {
    pub(crate) async fn register(&self, review_id: &str) -> ActivationReviewWaiter {
        let notify = Arc::new(Notify::new());
        let mut waiters = self
            .waiters
            .lock()
            .expect("activation waiter lock poisoned");
        let id = waiters.next_id;
        waiters.next_id = waiters
            .next_id
            .checked_add(1)
            .expect("activation waiter ID overflow");
        waiters
            .by_review
            .entry(review_id.to_owned())
            .or_default()
            .insert(id, Arc::downgrade(&notify));
        ActivationReviewWaiter {
            id,
            review_id: review_id.to_owned(),
            notify,
            waiters: Arc::downgrade(&self.waiters),
        }
    }

    pub(crate) async fn notify(&self, review_id: &str) {
        let waiters = self
            .waiters
            .lock()
            .expect("activation waiter lock poisoned")
            .by_review
            .remove(review_id);
        if let Some(waiters) = waiters {
            for waiter in waiters.into_values().filter_map(|waiter| waiter.upgrade()) {
                waiter.notify_one();
            }
        }
    }
}

impl Drop for ActivationReviewWaiter {
    fn drop(&mut self) {
        let Some(waiters) = self.waiters.upgrade() else {
            return;
        };
        let mut waiters = waiters.lock().expect("activation waiter lock poisoned");
        let remove_review = waiters
            .by_review
            .get_mut(&self.review_id)
            .is_some_and(|entries| {
                entries.remove(&self.id);
                entries.is_empty()
            });
        if remove_review {
            waiters.by_review.remove(&self.review_id);
        }
    }
}

impl ActivationReviewWaiter {
    pub(crate) async fn wait(self) {
        self.notify.notified().await;
    }
}

#[cfg(test)]
mod tests {
    use super::ActivationReviewNotifier;

    #[tokio::test]
    async fn notification_before_wait_is_retained() {
        let notifier = ActivationReviewNotifier::default();
        let waiter = notifier.register("review").await;

        notifier.notify("review").await;

        tokio::time::timeout(std::time::Duration::from_millis(100), waiter.wait())
            .await
            .expect("notification permit was lost");
    }

    #[tokio::test]
    async fn notification_wakes_every_registered_waiter() {
        let notifier = ActivationReviewNotifier::default();
        let first = notifier.register("review").await;
        let second = notifier.register("review").await;

        notifier.notify("review").await;

        tokio::time::timeout(std::time::Duration::from_millis(100), async {
            tokio::join!(first.wait(), second.wait());
        })
        .await
        .expect("registered waiter was not notified");
    }

    #[tokio::test]
    async fn dropping_waiter_removes_registration() {
        let notifier = ActivationReviewNotifier::default();
        drop(notifier.register("review").await);

        assert!(notifier
            .waiters
            .lock()
            .expect("activation waiter lock poisoned")
            .by_review
            .is_empty());
    }
}
