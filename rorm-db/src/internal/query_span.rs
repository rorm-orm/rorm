use sqlx::SqlStr;
use tracing::field::Empty;
use tracing::{info_span, Span};

pub struct QuerySpanBuilder {
    /// Counts the number for bind parameters (`values: Vec<_>`)
    ///
    /// The integer type is "small" because Postgres has a hard limit on this number.
    /// The integer is signed because the `tracing` + OpenTelemetry doesn't support unsigned:
    ///
    /// `tracing`'s `Visit` trait only has `record_i64` and `record_u64`.
    /// This forces every `Value` implementation for integers to choose one.
    /// Naturally, `uX` uses the `record_u64` method.
    /// OpenTelemetry only supports integer in the range of `i64`
    /// and explicitly documents to use strings for larger integer
    /// ([Source](https://opentelemetry.io/docs/specs/otel/common/attribute-type-mapping/#integer-values)).
    /// `tracing-opentelemetry` made the choice to not handle `uX` and always falls back to string.
    values_len: i32,
    span: Span,
}

impl QuerySpanBuilder {
    pub fn new(query: &SqlStr) -> Self {
        Self {
            values_len: 0,
            span: info_span!(
                target: "rorm::db::executor",
                "sql",
                sql = query.as_str(),
                values.len = Empty,
            ),
        }
    }

    pub fn track_bind(&mut self) {
        self.values_len = self.values_len.saturating_add(1);
    }

    pub fn finish(self) -> Span {
        self.span.record("values.len", self.values_len);
        self.span
    }
}
