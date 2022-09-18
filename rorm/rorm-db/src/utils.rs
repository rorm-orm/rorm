use sqlx::database::HasArguments;
use sqlx::query::Query;

use rorm_sql::conditional;

type AnyQuery<'q> = Query<'q, sqlx::Any, <sqlx::Any as HasArguments<'q>>::Arguments>;

/**
This helper method is used to bind ConditionValues to the query.
*/
pub(crate) fn bind_param<'post_query, 'query>(
    query: AnyQuery<'query>,
    param: conditional::ConditionValue<'post_query>,
) -> AnyQuery<'query>
where
    'post_query: 'query,
{
    match param {
        conditional::ConditionValue::String(x) => query.bind(x),
        conditional::ConditionValue::I64(x) => query.bind(x),
        conditional::ConditionValue::I32(x) => query.bind(x),
        conditional::ConditionValue::I16(x) => query.bind(x),
        conditional::ConditionValue::Bool(x) => query.bind(x),
        conditional::ConditionValue::F32(x) => query.bind(x),
        conditional::ConditionValue::F64(x) => query.bind(x),
        conditional::ConditionValue::Null => {
            static NULL: Option<bool> = None;
            query.bind(NULL)
        }
    }
}
