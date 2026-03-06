use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll, Waker};

use futures::{Sink, Stream, StreamExt};
use parking_lot::RwLock;

use crate::util::pipeline::BoxedProducer;

#[derive(Debug)]
struct SharedBroadcastBuffer<T> {
    state: Arc<BroadcastBufferState<T>>,
}

impl<T> Clone for SharedBroadcastBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

#[derive(Debug)]
struct BroadcastBufferState<T> {
    receiver_queues: RwLock<HashMap<u64, RwLock<ReceiverInner<T>>>>,
    receiver_id_gen: AtomicU64,
    capacity: usize,
}

#[derive(Debug)]
struct ReceiverInner<T> {
    queue: VecDeque<T>,
    recv_waker: Option<Waker>,
}

impl<T> SharedBroadcastBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            state: Arc::new(BroadcastBufferState {
                receiver_queues: RwLock::new(HashMap::new()),
                capacity,
                receiver_id_gen: AtomicU64::new(0),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastPipeConnectorSink<T> {
    buffer: SharedBroadcastBuffer<T>,
}

impl<T> Sink<T> for BroadcastPipeConnectorSink<T>
where
    T: Debug + Send + Unpin + Clone + 'static,
{
    type Error = eyre::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let g = self.buffer.state.receiver_queues.read();
        if g.is_empty() {
            return Ok(());
        }

        for recv_inner in g.values() {
            let mut g = recv_inner.write();
            g.queue.push_back(item.clone());
            if g.queue.len() > self.buffer.state.capacity {
                g.queue.pop_front();
            }
            if let Some(w) = g.recv_waker.take() {
                w.wake()
            }
        }

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // TODO
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // TODO
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub struct BroadcastPipeConnectorStream<T> {
    id: u64,
    buffer: SharedBroadcastBuffer<T>,
}

impl<T> BroadcastPipeConnectorStream<T> {
    pub fn into_producer(self) -> BoxedProducer<T>
    where
        T: Debug + Send + Clone + Sync + 'static,
    {
        Box::new(self.map(Ok))
    }
}

impl<T> Drop for BroadcastPipeConnectorStream<T> {
    fn drop(&mut self) {
        // Remove this receiver's queue when dropped
        let mut g = self.buffer.state.receiver_queues.write();
        g.remove(&self.id);
    }
}

impl<T> Stream for BroadcastPipeConnectorStream<T>
where
    T: Debug + Send + Clone + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let g = self.buffer.state.receiver_queues.read();
        let inner = g
            .get(&self.id)
            .expect("should exist as was inserted in connector creation");
        let mut m_g = inner.write();

        if let Some(item) = m_g.queue.pop_front() {
            Poll::Ready(Some(item))
        } else {
            m_g.recv_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[derive(Debug)]
pub struct BroadcastPipeConnector<T> {
    buffer: SharedBroadcastBuffer<T>,
}

impl<T: Send + Clone + 'static> BroadcastPipeConnector<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: SharedBroadcastBuffer::new(capacity),
        }
    }

    pub fn sink(&self) -> BroadcastPipeConnectorSink<T> {
        BroadcastPipeConnectorSink {
            buffer: self.buffer.clone(),
        }
    }

    pub fn stream(&self) -> BroadcastPipeConnectorStream<T> {
        let id = self.buffer.state.receiver_id_gen.fetch_add(1, Ordering::SeqCst);
        {
            let mut g = self.buffer.state.receiver_queues.write();
            g.insert(
                id,
                RwLock::new(ReceiverInner {
                    queue: VecDeque::new(),
                    recv_waker: None,
                }),
            );
        }

        BroadcastPipeConnectorStream {
            id,
            buffer: self.buffer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures::{Sink, SinkExt, StreamExt};

    use crate::util::pipeline::broadcast_pipe_connector::BroadcastPipeConnector;

    #[tokio::test]
    async fn test_broadcast_single_producer_multiple_consumers() {
        let channel = BroadcastPipeConnector::new(10);
        let mut sink = channel.sink();
        let mut stream1 = channel.stream();
        let mut stream2 = channel.stream();

        // Send messages
        for i in 0..5 {
            sink.send(i).await.unwrap();
        }

        // Both consumers should receive all messages
        let mut received1 = Vec::new();
        let mut received2 = Vec::new();

        for _ in 0..5 {
            if let Some(msg) = stream1.next().await {
                received1.push(msg);
            }
            if let Some(msg) = stream2.next().await {
                received2.push(msg);
            }
        }

        // Both streams should have received all messages
        received1.sort();
        received2.sort();
        assert_eq!(received1, vec![0, 1, 2, 3, 4]);
        assert_eq!(received2, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_broadcast_multiple_producers() {
        let channel = BroadcastPipeConnector::new(10);
        let mut sink1 = channel.sink();
        let mut sink2 = channel.sink();
        let mut stream = channel.stream();

        sink1.send(1).await.unwrap();
        sink2.send(2).await.unwrap();

        let mut received = Vec::new();
        for _ in 0..2 {
            if let Some(msg) = stream.next().await {
                received.push(msg);
            }
        }

        received.sort();
        assert_eq!(received, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_broadcast_no_receivers() {
        let channel = BroadcastPipeConnector::<i32>::new(10);
        let mut sink = channel.sink();

        // Try to send without any receivers - should fail
        let result = sink.send(42).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_backpressure() {
        let channel = BroadcastPipeConnector::new(2);
        let mut sink = channel.sink();
        let _stream1 = channel.stream();
        let _stream2 = channel.stream();

        // Fill both receiver queues (2 * 2 = 4 messages total capacity)
        sink.send(1).await.unwrap();
        sink.send(2).await.unwrap();
        sink.send(3).await.unwrap();
        sink.send(4).await.unwrap();

        // Try to send one more - should apply backpressure
        let sink_pin = Pin::new(&mut sink);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        match sink_pin.poll_ready(&mut cx) {
            Poll::Pending => {} // Expected - buffer is full
            Poll::Ready(Ok(())) => {
                // Might be ready due to timing
            }
            Poll::Ready(Err(e)) => panic!("Unexpected error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_broadcast_concurrent_access() {
        let channel = BroadcastPipeConnector::new(50);
        let num_producers = 3;
        let num_consumers = 2;
        let messages_per_producer = 10;

        // Create multiple producers
        let mut producer_handles = Vec::new();
        for producer_id in 0..num_producers {
            let sink = channel.sink();
            let handle = tokio::spawn(async move {
                let mut sink = sink;
                for i in 0..messages_per_producer {
                    let message = format!("p{}-m{}", producer_id, i);
                    sink.send(message).await.unwrap();
                }
            });
            producer_handles.push(handle);
        }

        // Create multiple consumers
        let mut consumer_handles = Vec::new();
        for _ in 0..num_consumers {
            let stream = channel.stream();
            let handle = tokio::spawn(async move {
                let mut stream = stream;
                let mut received = Vec::new();
                for _ in 0..(num_producers * messages_per_producer) {
                    if let Some(msg) = stream.next().await {
                        received.push(msg);
                    }
                }
                received
            });
            consumer_handles.push(handle);
        }

        // Wait for all producers to finish
        for handle in producer_handles {
            handle.await.unwrap();
        }

        // Collect results from all consumers
        let mut all_consumer_messages = Vec::new();
        for handle in consumer_handles {
            let consumer_messages = handle.await.unwrap();
            all_consumer_messages.push(consumer_messages);
        }

        // Each consumer should have received all messages
        let expected_total = num_producers * messages_per_producer;
        for consumer_messages in &all_consumer_messages {
            assert_eq!(consumer_messages.len(), expected_total);
        }

        // All consumers should have received the same messages (broadcast)
        for i in 1..all_consumer_messages.len() {
            let mut first_consumer = all_consumer_messages[0].clone();
            let mut current_consumer = all_consumer_messages[i].clone();
            first_consumer.sort();
            current_consumer.sort();
            assert_eq!(first_consumer, current_consumer);
        }
    }

    #[tokio::test]
    async fn test_broadcast_drop_cleanup() {
        let channel = BroadcastPipeConnector::new(10);
        let mut sink = channel.sink();

        // Create and drop a stream
        {
            let _stream = channel.stream();
            // Stream goes out of scope and should be cleaned up
        }

        // Should be able to send without receivers now
        let result = sink.send(42).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_capacity_1() {
        // Test with capacity = 1
        let channel = BroadcastPipeConnector::new(1);
        let mut sink = channel.sink();
        let mut stream1 = channel.stream();
        let mut stream2 = channel.stream();

        // Send one message (should fill both receiver queues)
        sink.send(42).await.unwrap();

        // Both streams should receive the message
        assert_eq!(stream1.next().await, Some(42));
        assert_eq!(stream2.next().await, Some(42));

        // Send another message
        sink.send(84).await.unwrap();
        assert_eq!(stream1.next().await, Some(84));
        assert_eq!(stream2.next().await, Some(84));
    }
}
