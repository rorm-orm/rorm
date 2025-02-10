//! Small utilities around the [`futures_core`] crate

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

/// Basic type alias for a dynamic future
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Basic type alias for a dynamic stream
pub(crate) type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

/// An empty [`Stream`]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct EmptyStream<T>(PhantomData<T>);

impl<T> Stream for EmptyStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
