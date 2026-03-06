use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{Sink, Stream, StreamExt};

mod broadcast_pipe_connector;
mod combined_producer;
mod deferred_producer;
mod round_robin_pipe_connector;

pub use broadcast_pipe_connector::{BroadcastPipeConnector, BroadcastPipeConnectorSink, BroadcastPipeConnectorStream};

/// [`Producer`] is used as pipeline entry point.
#[async_trait]
pub trait Producer<Item> {
    /// Fetches next [`Item`].
    async fn next(&mut self) -> Option<eyre::Result<Item>>;
}

#[async_trait]
impl<I, T> Producer<I> for T
where
    T: Stream<Item = eyre::Result<I>> + Unpin + Send,
{
    async fn next(&mut self) -> Option<eyre::Result<I>> {
        StreamExt::next(self).await
    }
}

pub type BoxedProducer<I> = Box<dyn Producer<I> + Unpin + Send>;

#[async_trait]
pub trait IntoStream<T> {
    async fn into_stream(self: Box<Self>) -> BoxStream<'static, eyre::Result<T>>;
}

#[async_trait]
impl<T, I> IntoStream<I> for T
where
    T: Producer<I> + Send + ?Sized + 'static,
    I: Send + 'static,
{
    async fn into_stream(self: Box<Self>) -> BoxStream<'static, eyre::Result<I>> {
        futures::stream::unfold(
            self,
            |mut state| async move { state.next().await.map(|item| (item, state)) },
        )
        .boxed()
    }
}

#[async_trait]
pub trait Transformer<Input, Output> {
    async fn transform(&self, i: Input) -> eyre::Result<Output>;
}

#[allow(dead_code)]
#[async_trait]
pub trait Filter<Item> {
    async fn filter(&self, i: Item) -> eyre::Result<Option<Item>>;
}

#[async_trait]
pub trait TransformFilter<Input, Output> {
    async fn filter_transform(&self, i: Input) -> eyre::Result<Option<Output>>;
}

#[allow(dead_code)]
#[async_trait]
pub trait Consumer<Item>: Sink<Item> {
    /// Fetches next [`Item`].
    async fn consume(&self, item: Item) -> eyre::Result<()>;
}
