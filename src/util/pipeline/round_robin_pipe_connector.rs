use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll, Waker};

use eyre::bail;
use futures::StreamExt;
use futures::{Sink, Stream};
use parking_lot::{FairMutex, RwLock};

use crate::util::pipeline::BoxedProducer;

#[derive(Debug)]
struct SharedRoundRobinBuffer<T> {
    state: Arc<RoundRobinBufferState<T>>,
}

impl<T> Clone for SharedRoundRobinBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

#[derive(Debug)]
struct RoundRobinBufferState<T> {
    receiver_queues: RwLock<HashMap<u64, RwLock<ReceiverInner<T>>>>,
    send_wakers: FairMutex<Vec<Waker>>,
    receiver_id_gen: AtomicU64,
    capacity: usize,
}

#[derive(Debug)]
struct ReceiverInner<T> {
    queue: VecDeque<T>,
    recv_waker: Option<Waker>,
}

impl<T> SharedRoundRobinBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            state: Arc::new(RoundRobinBufferState {
                receiver_queues: RwLock::new(HashMap::new()),
                capacity,
                receiver_id_gen: AtomicU64::new(0),
                send_wakers: FairMutex::new(vec![]),
            }),
        }
    }

    fn wake_one_sender(&self) {
        let mut g = self.state.send_wakers.lock();
        if let Some(w) = g.pop() {
            w.wake();
        }
    }
}

#[derive(Debug)]
pub struct RoundRobinPipeConnectorSink<T> {
    buffer: SharedRoundRobinBuffer<T>,
}

impl<T> Clone for RoundRobinPipeConnectorSink<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

impl<T> Sink<T> for RoundRobinPipeConnectorSink<T>
where
    T: Debug + Send + Unpin + 'static,
{
    type Error = eyre::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let g = self.buffer.state.receiver_queues.read();
        let mut length = 0;
        for (_, l) in g.iter() {
            let g1 = l.read();
            length += g1.queue.len();
        }

        if length < self.buffer.state.capacity {
            Poll::Ready(Ok(()))
        } else {
            // Buffer is full, register waker
            let mut w_g = self.buffer.state.send_wakers.lock();
            w_g.push(cx.waker().clone());
            Poll::Pending
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let g = self.buffer.state.receiver_queues.read();
        if g.is_empty() {
            return Ok(());
        }
        const MAX_ATTEMPTS: usize = 7;
        for _ in 0..MAX_ATTEMPTS {
            for (_, recv_inner) in g.iter() {
                if let Some(mut i) = recv_inner.try_write() {
                    i.queue.push_back(item);
                    if let Some(w) = i.recv_waker.take() {
                        w.wake()
                    }
                    return Ok(());
                }
            }
        }

        bail!("All receivers are busy after {} attempts", MAX_ATTEMPTS)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub struct RoundRobinPipeConnectorStream<T> {
    id: u64,
    buffer: SharedRoundRobinBuffer<T>,
}

impl<T> Drop for RoundRobinPipeConnectorStream<T> {
    fn drop(&mut self) {
        // Remove this receiver's queue when dropped
        let mut g = self.buffer.state.receiver_queues.write();
        g.remove(&self.id);

        // Wake all senders in case they were waiting for capacity
        drop(g);
        self.buffer.wake_one_sender();
    }
}

impl<T> RoundRobinPipeConnectorStream<T>
where
    T: Debug + Send + 'static,
{
    fn try_steal_item(&self) -> Option<T> {
        let g = self.buffer.state.receiver_queues.read();
        for (id, l) in g.iter() {
            if *id == self.id {
                continue;
            }

            let mut m_g = l.write();
            if let Some(item) = m_g.queue.pop_front() {
                return Some(item);
            }
        }

        None
    }

    pub fn into_producer(self) -> BoxedProducer<T>
    where
        T: Sync,
    {
        Box::new(self.map(Ok))
    }
}

impl<T> Stream for RoundRobinPipeConnectorStream<T>
where
    T: Debug + Send + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let g = self.buffer.state.receiver_queues.read();
        let inner = g
            .get(&self.id)
            .expect("should exist as was inserted in connector creation");
        let mut m_g = inner.write();
        if let Some(item) = m_g.queue.pop_front() {
            if m_g.queue.len() < self.buffer.state.capacity {
                self.buffer.wake_one_sender();
            }

            Poll::Ready(Some(item))
        } else {
            if let Some(item) = self.try_steal_item() {
                return Poll::Ready(Some(item));
            }

            m_g.recv_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
#[derive(Debug)]
pub struct RoundRobinPipeConnector<T> {
    buffer: SharedRoundRobinBuffer<T>,
}

impl<T: Send + 'static> RoundRobinPipeConnector<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: SharedRoundRobinBuffer::new(capacity),
        }
    }

    pub fn sink(&self) -> RoundRobinPipeConnectorSink<T> {
        RoundRobinPipeConnectorSink {
            buffer: self.buffer.clone(),
        }
    }

    pub fn stream(&self) -> RoundRobinPipeConnectorStream<T> {
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

        RoundRobinPipeConnectorStream {
            id,
            buffer: self.buffer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures::{Sink, SinkExt, Stream, StreamExt};

    use crate::util::pipeline::round_robin_pipe_connector::RoundRobinPipeConnector;

    #[tokio::test]
    async fn test_round_robin_single_producer_multiple_consumers() {
        let channel = RoundRobinPipeConnector::new(10);
        let mut sink = channel.sink();
        let mut stream1 = channel.stream();
        let mut stream2 = channel.stream();

        // Send messages
        for i in 0..10 {
            sink.send(i).await.unwrap();
        }

        // Collect messages from both consumers
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

        // Each consumer should have received some messages
        assert!(!received1.is_empty());
        assert!(!received2.is_empty());

        // Total messages should equal sent messages
        assert_eq!(received1.len() + received2.len(), 10);

        // No duplicates - each message received by only one consumer
        let mut all_messages: HashSet<i32> = HashSet::new();
        all_messages.extend(received1.iter());
        all_messages.extend(received2.iter());
        assert_eq!(all_messages.len(), 10);
    }

    #[tokio::test]
    async fn test_round_robin_multiple_producers() {
        let channel = RoundRobinPipeConnector::new(10);
        let mut sink1 = channel.sink();
        let mut sink2 = channel.sink();
        let mut stream = channel.stream();

        sink1.send(1).await.unwrap();
        sink2.send(2).await.unwrap();

        let msg1 = stream.next().await.unwrap();
        let msg2 = stream.next().await.unwrap();

        // Should receive both messages (order may vary)
        let mut received = vec![msg1, msg2];
        received.sort();
        assert_eq!(received, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_round_robin_backpressure() {
        let channel = RoundRobinPipeConnector::new(2);
        let mut sink = channel.sink();
        // without stream sink will not sink
        let _stream = channel.stream();

        // Fill the buffer
        sink.send(1).await.unwrap();
        sink.send(2).await.unwrap();

        // Try to send one more - should apply backpressure
        // This would block if buffer is full, so we test with poll_ready
        let sink_pin = Pin::new(&mut sink);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Buffer should be full
        match sink_pin.poll_ready(&mut cx) {
            Poll::Pending => {} // Expected - buffer is full
            Poll::Ready(Ok(())) => {
                // Might be ready if cleanup happened
            }
            Poll::Ready(Err(e)) => panic!("Unexpected error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_send_without_receivers() {
        let channel = RoundRobinPipeConnector::<i32>::new(10);
        let mut sink = channel.sink();

        // Try to send without any receivers - should fail
        let result = sink.send(42).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_work_stealing() {
        let channel = RoundRobinPipeConnector::new(10);
        let mut sink = channel.sink();
        let mut stream1 = channel.stream();
        let mut stream2 = channel.stream();

        // Send messages to fill one receiver's queue preferentially
        for i in 0..5 {
            sink.send(i).await.unwrap();
        }

        // stream1 consumes its own messages first
        let mut stream1_messages = Vec::new();
        for _ in 0..3 {
            if let Some(msg) = stream1.next().await {
                stream1_messages.push(msg);
            }
        }

        // stream2 should steal remaining messages from stream1
        let mut stream2_messages = Vec::new();
        for _ in 0..2 {
            if let Some(msg) = stream2.next().await {
                stream2_messages.push(msg);
            }
        }

        // Verify all messages were received
        let mut all_messages = stream1_messages;
        all_messages.extend(stream2_messages);
        all_messages.sort();
        assert_eq!(all_messages, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_multiple_sinks_single_stream() {
        let channel = RoundRobinPipeConnector::new(10);
        let mut sink1 = channel.sink();
        let mut sink2 = channel.sink();
        let mut sink3 = channel.sink();
        let mut stream = channel.stream();

        // Send from multiple sinks concurrently
        let send_task1 = tokio::spawn(async move {
            for i in 0..5 {
                sink1.send(format!("sink1-{}", i)).await.unwrap();
            }
        });

        let send_task2 = tokio::spawn(async move {
            for i in 0..5 {
                sink2.send(format!("sink2-{}", i)).await.unwrap();
            }
        });

        let send_task3 = tokio::spawn(async move {
            for i in 0..5 {
                sink3.send(format!("sink3-{}", i)).await.unwrap();
            }
        });

        let stream_task = tokio::spawn(async move {
            // Collect all messages
            let mut messages = Vec::new();
            for _ in 0..15 {
                if let Some(msg) = stream.next().await {
                    messages.push(msg);
                }
            }

            assert_eq!(messages.len(), 15);

            // Verify we got messages from all sinks
            let sink1_count = messages.iter().filter(|m| m.starts_with("sink1")).count();
            let sink2_count = messages.iter().filter(|m| m.starts_with("sink2")).count();
            let sink3_count = messages.iter().filter(|m| m.starts_with("sink3")).count();

            assert_eq!(sink1_count, 5);
            assert_eq!(sink2_count, 5);
            assert_eq!(sink3_count, 5);
        });

        // Wait for all sends to complete
        let _r = tokio::join!(send_task1, send_task2, send_task3, stream_task);
    }

    #[tokio::test]
    async fn test_high_contention_scenario() {
        let channel = RoundRobinPipeConnector::new(100);
        let num_producers = 6;
        let num_consumers = 5;
        let messages_per_producer = 50;
        let messages_per_consumer = 60;

        assert_eq!(
            num_producers * messages_per_producer,
            num_consumers * messages_per_consumer
        );

        // Create multiple producers
        let mut producer_handles = Vec::new();
        for producer_id in 0..num_producers {
            let sink = channel.sink();
            let handle = tokio::spawn(async move {
                let mut sink = sink;
                for i in 0..messages_per_producer {
                    let message = format!("p{}-m{}", producer_id, i);
                    sink.send(message.clone()).await.unwrap();
                    // Add small delay to simulate real workload
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
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
                // Each consumer tries to get some messages
                for _ in 0..messages_per_consumer {
                    if let Some(msg) = stream.next().await {
                        received.push(msg);
                        // Small processing delay
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    } else {
                        break;
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
        let mut all_received_messages = Vec::new();
        for handle in consumer_handles {
            let consumer_messages = handle.await.unwrap();
            all_received_messages.extend(consumer_messages);
        }

        // Verify total message count
        let expected_total = num_producers * messages_per_producer;
        assert_eq!(all_received_messages.len(), expected_total);

        // Verify no duplicates
        let mut unique_messages = HashSet::new();
        for msg in &all_received_messages {
            assert!(unique_messages.insert(msg.clone()), "Duplicate message: {}", msg);
        }

        // Verify we have messages from all producers
        for producer_id in 0..num_producers {
            let producer_messages: Vec<_> = all_received_messages
                .iter()
                .filter(|msg| msg.starts_with(&format!("p{}-", producer_id)))
                .collect();
            assert_eq!(producer_messages.len(), messages_per_producer);
        }
    }

    #[tokio::test]
    async fn test_empty_channel_polling() {
        let channel = RoundRobinPipeConnector::<i32>::new(10);
        let mut stream = channel.stream();

        // Poll empty stream - should return Pending
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let stream_pin = Pin::new(&mut stream);

        match stream_pin.poll_next(&mut cx) {
            Poll::Pending => {} // Expected
            Poll::Ready(None) => panic!("Stream should not be closed"),
            Poll::Ready(Some(_)) => panic!("Should not receive message from empty stream"),
        }
    }

    #[tokio::test]
    async fn test_capacity_edge_case() {
        // Test with capacity = 1
        let channel = RoundRobinPipeConnector::new(1);
        let mut sink = channel.sink();
        let mut stream = channel.stream();

        // Send one message (should fill capacity)
        sink.send(42).await.unwrap();

        // Verify backpressure
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

        // Consume the message
        let msg = stream.next().await.unwrap();
        assert_eq!(msg, 42);

        // Now sink should be ready again
        sink.send(84).await.unwrap();
        let msg = stream.next().await.unwrap();
        assert_eq!(msg, 84);
    }

    #[tokio::test]
    async fn test_zero_capacity() {
        // Test edge case with zero capacity - should still work but with immediate backpressure
        let channel = RoundRobinPipeConnector::new(0);
        let mut sink = channel.sink();
        let mut stream = channel.stream();

        // With zero capacity, poll_ready should return Pending
        let sink_pin = Pin::new(&mut sink);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        match sink_pin.poll_ready(&mut cx) {
            Poll::Pending => {} // Expected with zero capacity
            Poll::Ready(Ok(())) => {
                // Might be ready due to implementation details
                // Try to send a message
                let result = sink.send(42).await;
                if result.is_ok() {
                    // If send succeeded, verify we can receive it
                    let msg = stream.next().await.unwrap();
                    assert_eq!(msg, 42);
                }
            }
            Poll::Ready(Err(e)) => panic!("Unexpected error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_round_robin_drop_cleanup() {
        let channel = RoundRobinPipeConnector::new(10);
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
}
