use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::BoxStream;
use futures::Stream;
use rorm_sql::conditional;
use sqlx::any::AnyRow;
use sqlx::{AnyPool, Error};

#[ouroboros::self_referencing]
pub struct QueryStream<'post_query> {
    pub(crate) query_str: String,
    pub(crate) bind_params: Vec<conditional::ConditionValue<'post_query>>,
    #[borrows(query_str, bind_params)]
    #[not_covariant]
    pub(crate) stream: BoxStream<'this, Result<AnyRow, Error>>,
}

impl<'post_query> QueryStream<'post_query> {
    pub(crate) fn build(
        stmt: String,
        bind_params: Vec<conditional::ConditionValue<'post_query>>,
        executor: &AnyPool,
    ) -> QueryStream<'post_query> {
        QueryStream::new(stmt, bind_params, |x, y| {
            let mut tmp = sqlx::query(x);

            for x in y {
                tmp = match x {
                    conditional::ConditionValue::String(x) => tmp.bind(x),
                    conditional::ConditionValue::I64(x) => tmp.bind(x),
                    conditional::ConditionValue::I32(x) => tmp.bind(x),
                    conditional::ConditionValue::I16(x) => tmp.bind(x),
                }
            }

            return tmp.fetch(executor);
        })
    }
}

impl Stream for QueryStream<'_> {
    type Item = Result<AnyRow, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.with_stream_mut(|x| x.as_mut().poll_next(cx))
    }
}
