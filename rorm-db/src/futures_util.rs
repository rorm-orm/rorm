//! Small utilities around the [`futures_core`] crate

use std::future::Future;
use std::pin::Pin;

use futures_core::Stream;

/// Basic type alias for a dynamic future
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Basic type alias for a dynamic stream
pub(crate) type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;
