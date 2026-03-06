use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use once_cell::sync::OnceCell;
use tokio::sync::Notify;

use crate::util::pipeline::{BoxedProducer, Producer};

#[derive(derive_more::Debug)]
pub struct DeferredProducer<T> {
    #[debug(skip)]
    producer_future: Option<BoxFuture<'static, eyre::Result<BoxedProducer<T>>>>,
    notifier: Arc<Notify>,
    producer: OnceCell<BoxedProducer<T>>,
}

impl<T> DeferredProducer<T> {
    pub fn new(producer_future: BoxFuture<'static, eyre::Result<BoxedProducer<T>>>, notifier: Arc<Notify>) -> Self {
        Self {
            producer_future: Some(producer_future),
            notifier,
            producer: OnceCell::new(),
        }
    }
}

#[async_trait]
impl<T> Producer<T> for DeferredProducer<T>
where
    T: Send,
{
    async fn next(&mut self) -> Option<eyre::Result<T>> {
        if self.producer.get().is_none() {
            // Wait until external initialization is signaled
            self.notifier.notified().await;
            let fut = self
                .producer_future
                .take()
                .expect("producer_future should be present until first initialization");
            let producer = fut.await;
            match producer {
                Ok(p) => self.producer.get_or_init(|| p),
                Err(e) => {
                    tracing::error!("Producer creation error: {}", e);
                    return None;
                }
            };
        }
        let p = self
            .producer
            .get_mut()
            .expect("should be initialized as was checked before");
        p.next().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Notify;
    use tokio::time::{Duration, sleep, timeout};

    use super::DeferredProducer;
    use crate::util::pipeline::Producer;

    fn boxed_producer_from_vec(items: Vec<eyre::Result<i32>>) -> crate::util::pipeline::BoxedProducer<i32> {
        let stream = futures::stream::iter(items);
        Box::new(stream)
    }

    #[tokio::test]
    async fn deferred_initializes_on_notify_and_yields_items() {
        let notifier = Arc::new(Notify::new());
        let boxed = boxed_producer_from_vec(vec![Ok(1), Ok(2), Ok(3)]);
        let fut = Box::pin(async move { Ok(boxed) });

        let mut dp = DeferredProducer::new(fut, notifier.clone());

        // Without notify, next should not complete within timeout
        assert!(timeout(Duration::from_millis(50), dp.next()).await.is_err());

        // Notify initialization, then items should flow
        notifier.notify_one();
        assert_eq!(dp.next().await.unwrap().unwrap(), 1);
        assert_eq!(dp.next().await.unwrap().unwrap(), 2);
        assert_eq!(dp.next().await.unwrap().unwrap(), 3);
        assert!(dp.next().await.is_none());
    }

    #[tokio::test]
    async fn deferred_future_error_results_in_none() {
        let notifier = Arc::new(Notify::new());
        let fut = Box::pin(async move { eyre::bail!("init failed") });
        let mut dp = DeferredProducer::<i32>::new(fut, notifier.clone());

        notifier.notify_one();
        // If initialization fails, next should return None (end of stream)
        assert!(dp.next().await.is_none());
    }

    #[tokio::test]
    async fn underlying_error_is_propagated() {
        let notifier = Arc::new(Notify::new());
        let boxed = boxed_producer_from_vec(vec![Err(eyre::eyre!("boom")), Ok(10)]);
        let fut = Box::pin(async move { Ok(boxed) });

        let mut dp = DeferredProducer::new(fut, notifier.clone());
        notifier.notify_one();

        let first = dp.next().await.unwrap();
        assert!(first.is_err());

        let second = dp.next().await.unwrap();
        assert_eq!(second.unwrap(), 10);
    }

    #[tokio::test]
    async fn notify_can_happen_after_some_delay() {
        let notifier = Arc::new(Notify::new());
        let boxed = boxed_producer_from_vec(vec![Ok(7)]);
        let fut = Box::pin(async move {
            // Simulate async creation delay
            sleep(Duration::from_millis(10)).await;
            Ok(boxed)
        });

        let mut dp = DeferredProducer::new(fut, notifier.clone());

        // Kick off a delayed notify
        let n = notifier.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(20)).await;
            n.notify_one();
        });

        // First item becomes available only after notify and future completes
        let v = dp.next().await.unwrap().unwrap();
        assert_eq!(v, 7);
    }
}
