use std::marker::PhantomData;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{StreamExt, stream};

use crate::util::pipeline::{IntoStream, Producer};

pub struct CombinedProducer<P, I>
where
    P: Producer<I> + IntoStream<I> + ?Sized,
{
    stream: BoxStream<'static, eyre::Result<I>>,
    phantom_data: PhantomData<P>,
}

impl<P, I> CombinedProducer<P, I>
where
    P: Producer<I> + IntoStream<I> + Unpin + Send + ?Sized + 'static,
    I: 'static,
{
    pub async fn new(producers: impl IntoIterator<Item = Box<P>>) -> Self {
        let mut streams = vec![];
        for p in producers {
            streams.push(p.into_stream().await);
        }
        Self {
            stream: stream::select_all(streams).boxed(),
            phantom_data: PhantomData,
        }
    }
}

#[async_trait]
impl<P, I> Producer<I> for CombinedProducer<P, I>
where
    P: Producer<I> + IntoStream<I> + ?Sized + Unpin + Send + 'static,
{
    async fn next(&mut self) -> Option<eyre::Result<I>> {
        StreamExt::next(&mut self.stream).await
    }
}
