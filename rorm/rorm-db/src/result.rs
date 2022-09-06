use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::BoxStream;
use futures::{Stream, TryStream, TryStreamExt};
use sqlx::any::AnyRow;
use sqlx::Row;
use sqlx::{AnyPool, Error};

#[ouroboros::self_referencing]
pub struct QueryStream {
    pub(crate) query_str: String,
    #[borrows(query_str)]
    #[not_covariant]
    pub(crate) stream: BoxStream<'this, Result<AnyRow, Error>>,
}

impl QueryStream {
    pub(crate) fn build(stmt: String, executor: &AnyPool) -> QueryStream {
        QueryStream::new(stmt, |x| sqlx::query(x).fetch(executor))
    }
}

impl Stream for QueryStream {
    type Item = Result<AnyRow, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.with_stream_mut(|x| x.as_mut().poll_next(cx))
    }
}
