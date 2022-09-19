use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::Error;
use crate::row::Row;
use crate::utils;
use futures::stream::BoxStream;
use futures::Stream;
use rorm_sql::value;
use sqlx::any::AnyRow;
use sqlx::AnyPool;

#[ouroboros::self_referencing]
pub struct QueryStream<'post_query> {
    pub(crate) query_str: String,
    pub(crate) bind_params: Vec<value::Value<'post_query>>,
    #[borrows(query_str, bind_params)]
    #[not_covariant]
    pub(crate) stream: BoxStream<'this, Result<AnyRow, sqlx::Error>>,
}

impl<'post_query> QueryStream<'post_query> {
    pub(crate) fn build(
        stmt: String,
        bind_params: Vec<value::Value<'post_query>>,
        executor: &AnyPool,
    ) -> QueryStream<'post_query> {
        QueryStream::new(stmt, bind_params, |stmt, bind_params| {
            let mut tmp = sqlx::query(stmt);

            for x in bind_params {
                tmp = utils::bind_param(tmp, *x);
            }

            return tmp.fetch(executor);
        })
    }
}

impl Stream for QueryStream<'_> {
    type Item = Result<Row, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.with_stream_mut(|x| {
            x.as_mut()
                .poll_next(cx)
                .map(|option| option.map(|result| result.map(Row::from).map_err(Error::SqlxError)))
        })
    }
}
