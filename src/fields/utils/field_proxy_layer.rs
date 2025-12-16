//! Re-usable layers for [`FieldType::FieldProxyLayers`]
//!

use std::borrow::Cow;
use std::marker::PhantomData;

use rorm_db::sql::aggregation::SelectAggregator;

use crate::conditions::{Binary, BinaryOperator, Column, In, InOperator, Value};
use crate::crud::selector::AggregatedColumn;
use crate::fields::proxy;
use crate::fields::proxy::FieldProxyLayerStack;
use crate::fields::proxy::{FieldProxyImpl, LayerStackBase};
use crate::fields::traits::{Array, FieldType};
use crate::{declare_proxy_layer, sealed};

/// Layer stack containing `SimpleEq` and `SimpleOrd` for some type `T`
pub type SimpleEqOrd<T, L = LayerStackBase> = SimpleEq<T, SimpleOrd<T, L>>;

/// Layer stack containing `SimpleEq` and `SimpleOrd` for some type `Option<T>`
pub type OptionSimpleEqOrd<T, L = LayerStackBase> = SimpleEqOrd<Option<T>, L>;

/// Layer stack containing `SimpleEq`, `SimpleOrd` and `SimpleMinMax` for some type `T`
pub type SimpleEqOrdMinMax<T, L = LayerStackBase> = SimpleEqOrd<T, SimpleMinMax<T, L>>;

/// Layer stack containing `SimpleEq`, `SimpleOrd` and `SimpleMinMax` for some type `Option<T>`
pub type OptionSimpleEqOrdMinMax<T, L = LayerStackBase> =
    SimpleEqOrd<Option<T>, SimpleMinMax<T, L>>;

declare_proxy_layer!(
    /// Field proxy layer which implements basic equality comparisons
    /// using the obvious SQL operators.
    ///
    /// The generic parameter `T` is the type of values to compare to.
    SimpleEq<T, _>
);

impl<T, I> SimpleEq<T, I>
where
    T: FieldType<Columns = Array<1>>,
    I: FieldProxyImpl,
{
    /// Constructs SQL condition comparing the field to a `value` using `==`
    ///
    /// ```
    /// let condition = SomeModel.some_field.equals(1.0);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn equals<'a>(&self, value: T) -> Binary<Column<I>, Value<'a>> {
        simple_binary(BinaryOperator::Equals, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `!=`
    ///
    /// ```
    /// let condition = SomeModel.some_field.not_equals(1.0);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn not_equals<'a>(&self, value: T) -> Binary<Column<I>, Value<'a>> {
        simple_binary(BinaryOperator::NotEquals, value)
    }

    /// Constructs SQL condition checking the field to be one of `values` using `IN`
    ///
    /// ```
    /// let condition = SomeModel.some_field.r#in([1.0, 2.0, 3.0]);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn r#in<'a>(self, values: impl IntoIterator<Item = T>) -> In<Column<I>, Value<'a>> {
        In {
            operator: InOperator::In,
            fst_arg: Column(proxy::new()),
            snd_arg: values
                .into_iter()
                .map(|value| {
                    let [value] = value.into_values();
                    value
                })
                .collect(),
        }
    }

    /// Constructs SQL condition checking the field to be one of `values` using `NOT IN`
    ///
    /// ```
    /// let condition = SomeModel.some_field.not_in([1.0, 2.0, 3.0]);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn not_in<'a>(self, values: impl IntoIterator<Item = T>) -> In<Column<I>, Value<'a>> {
        In {
            operator: InOperator::NotIn,
            fst_arg: Column(proxy::new()),
            snd_arg: values
                .into_iter()
                .map(|value| {
                    let [value] = value.into_values();
                    value
                })
                .collect(),
        }
    }
}

declare_proxy_layer!(
    /// Field proxy layer which implements basic ordering comparisons
    /// using the obvious SQL operators.
    ///
    /// The generic parameter `T` is the type of values to compare to.
    SimpleOrd<T, _>
);

impl<T, I> SimpleOrd<T, I>
where
    T: FieldType<Columns = Array<1>>,
    I: FieldProxyImpl,
{
    /// Constructs SQL condition comparing the field to a `value` using `>`
    ///
    /// ```
    /// let condition = SomeModel.some_field.greater_than(1.0);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn greater_than<'a>(&self, value: T) -> Binary<Column<I>, Value<'a>> {
        simple_binary(BinaryOperator::Greater, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `>=`
    ///
    /// ```
    /// let condition = SomeModel.some_field.greater_equals(1.0);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn greater_equals<'a>(&self, value: T) -> Binary<Column<I>, Value<'a>> {
        simple_binary(BinaryOperator::GreaterOrEquals, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `<`
    ///
    /// ```
    /// let condition = SomeModel.some_field.less_than(1.0);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn less_than<'a>(&self, value: T) -> Binary<Column<I>, Value<'a>> {
        simple_binary(BinaryOperator::Less, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `<=`
    ///
    /// ```
    /// let condition = SomeModel.some_field.less_equals(1.0);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn less_equals<'a>(&self, value: T) -> Binary<Column<I>, Value<'a>> {
        simple_binary(BinaryOperator::LessOrEquals, value)
    }
}

/// Helper function to reduce repetition
fn simple_binary<'a, T: FieldType<Columns = Array<1>>, I: FieldProxyImpl>(
    operator: BinaryOperator,
    value: T,
) -> Binary<Column<I>, Value<'a>> {
    let [snd_arg] = value.into_values();
    Binary {
        operator,
        fst_arg: Column(proxy::new()),
        snd_arg,
    }
}

declare_proxy_layer!(
    /// Field proxy layer which implements `SUM` and `AVG` SQL aggregation.
    ///
    /// The generic parameter `T` is the type resulting from the sum.
    SimpleSumAvg<T, _>
);

impl<T, I> SimpleSumAvg<T, I>
where
    T: FieldType<Columns = Array<1>>,
    I: FieldProxyImpl,
{
    /// Constructs SQL aggregation for the field using the `AVG` operator.
    ///
    /// ```
    /// let condition = SomeModel.some_field.avg();
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn avg<'a>(&self) -> AggregatedColumn<I, Option<f64>> {
        AggregatedColumn {
            sql: SelectAggregator::Avg,
            alias: "avg",
            field: proxy::new(),
            result: PhantomData,
        }
    }

    /// Constructs SQL aggregation for the field using the `SUM` operator.
    ///
    /// ```
    /// let condition = SomeModel.some_field.sum();
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn sum<'a>(&self) -> AggregatedColumn<I, Option<T>> {
        AggregatedColumn {
            sql: SelectAggregator::Sum,
            alias: "sum",
            field: proxy::new(),
            result: PhantomData,
        }
    }
}

declare_proxy_layer!(
    /// Field proxy layer which implements `MIN` and `MAX` SQL aggregation.
    ///
    /// The generic parameter `T` is the type resulting from both.
    /// (`T` will be wrapped in `Option<_>` because `NULL` will be returned for empty queries)
    SimpleMinMax<T, _>
);

impl<T, I> SimpleMinMax<T, I>
where
    T: FieldType<Columns = Array<1>>,
    I: FieldProxyImpl,
{
    /// Constructs SQL aggregation for the field using the `MIN` operator.
    ///
    /// ```
    /// let condition = SomeModel.some_field.min();
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn min<'a>(&self) -> AggregatedColumn<I, Option<T>> {
        AggregatedColumn {
            sql: SelectAggregator::Min,
            alias: "min",
            field: proxy::new(),
            result: PhantomData,
        }
    }

    /// Constructs SQL aggregation for the field using the `MAX` operator.
    ///
    /// ```
    /// let condition = SomeModel.some_field.max();
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: f32,
    /// }
    /// ```
    pub fn max<'a>(&self) -> AggregatedColumn<I, Option<T>> {
        AggregatedColumn {
            sql: SelectAggregator::Max,
            alias: "max",
            field: proxy::new(),
            result: PhantomData,
        }
    }
}

declare_proxy_layer!(
    ///
    StringLayers<_>
);
impl<I> StringLayers<I>
where
    I: FieldProxyImpl,
{
    /// Constructs SQL condition comparing the field to a `value` using `==`
    ///
    /// ```
    /// let condition = SomeModel.some_field.equals("foo");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn equals<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::Equals, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `!=`
    ///
    /// ```
    /// let condition = SomeModel.some_field.not_equals("foo");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn not_equals<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::NotEquals, value)
    }

    /// Constructs SQL condition checking the field to be one of `values` using `IN`
    ///
    /// ```
    /// let condition = SomeModel.some_field.r#in(["foo", "bar", "baz"]);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn r#in<'a>(
        self,
        values: impl IntoIterator<Item = impl Str<'a>>,
    ) -> In<Column<I>, Value<'a>> {
        In {
            operator: InOperator::In,
            fst_arg: Column(proxy::new()),
            snd_arg: values.into_iter().map(Str::into_value).collect(),
        }
    }

    /// Constructs SQL condition checking the field to be one of `values` using `NOT IN`
    ///
    /// ```
    /// let condition = SomeModel.some_field.not_in(["foo", "bar", "baz"]);
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn not_in<'a>(
        self,
        values: impl IntoIterator<Item = impl Str<'a>>,
    ) -> In<Column<I>, Value<'a>> {
        In {
            operator: InOperator::NotIn,
            fst_arg: Column(proxy::new()),
            snd_arg: values.into_iter().map(Str::into_value).collect(),
        }
    }

    /// Constructs SQL condition comparing the field to a `value` using `>`
    ///
    /// ```
    /// let condition = SomeModel.some_field.greater_than("foo");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn greater_than<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::Greater, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `>=`
    ///
    /// ```
    /// let condition = SomeModel.some_field.greater_equals("foo");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn greater_equals<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::GreaterOrEquals, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `<`
    ///
    /// ```
    /// let condition = SomeModel.some_field.less_than("foo");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn less_than<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::Less, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `<=`
    ///
    /// ```
    /// let condition = SomeModel.some_field.less_equals("foo");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn less_equals<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::LessOrEquals, value)
    }

    /// Constructs SQL aggregation for the field using the `MIN` operator.
    ///
    /// ```
    /// let condition = SomeModel.some_field.min();
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn min(&self) -> AggregatedColumn<I, Option<String>> {
        AggregatedColumn {
            sql: SelectAggregator::Min,
            alias: "min",
            field: proxy::new(),
            result: PhantomData,
        }
    }

    /// Constructs SQL aggregation for the field using the `MAX` operator.
    ///
    /// ```
    /// let condition = SomeModel.some_field.max();
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn max(&self) -> AggregatedColumn<I, Option<String>> {
        AggregatedColumn {
            sql: SelectAggregator::Max,
            alias: "max",
            field: proxy::new(),
            result: PhantomData,
        }
    }

    /// Constructs SQL condition comparing the field to a `value` using `LIKE`
    ///
    /// ```
    /// let condition = SomeModel.some_field.like("f%o");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn like<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::Like, value)
    }

    /// Constructs SQL condition comparing the field to a `value` using `NOT LIKE`
    ///
    /// ```
    /// let condition = SomeModel.some_field.not_like("f%o");
    ///
    /// #[derive(Model)]
    /// pub struct SomeModel {
    ///     #[rorm(id)]
    ///     pub id: i64,
    ///
    ///     pub some_field: String,
    /// }
    /// ```
    pub fn not_like<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::NotLike, value)
    }

    /// Uses `LIKE` to check whether the field contains the string `rhs`
    pub fn contains<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like(format!("%{}%", escape_like(value)))
    }

    /// Uses `LIKE` to check whether the field starts with the string `rhs`
    pub fn starts_with<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like(format!("{}%", escape_like(value)))
    }

    /// Uses `LIKE` to check whether the field ends with the string `rhs`
    pub fn ends_with<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like(format!("%{}", escape_like(value)))
    }
}

#[cfg(feature = "postgres-only")]
impl<I> StringLayers<I>
where
    I: FieldProxyImpl,
{
    /// Compare the field to another value using `ILIKE`
    pub fn like_ignore_case<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::ILike, value)
    }

    /// Compare the field to another value using `NOT ILIKE`
    pub fn not_like_ignore_case<'a>(&self, value: impl Str<'a>) -> Binary<Column<I>, Value<'a>> {
        string_binary(BinaryOperator::NotILike, value)
    }

    /// Uses `ILIKE` to check whether the field contains the string `rhs` while ignoring case
    pub fn contains_ignore_case<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like_ignore_case(format!("%{}%", escape_like(value)))
    }

    /// Uses `ILIKE` to check whether the field starts with the string `rhs` while ignoring case
    pub fn starts_with_ignore_case<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like_ignore_case(format!("{}%", escape_like(value)))
    }

    /// Uses `ILIKE` to check whether the field ends with the string `rhs` while ignoring case
    pub fn ends_with_ignore_case<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like_ignore_case(format!("%{}", escape_like(value)))
    }

    /// Uses `ILIKE` to check whether the field is equal to the string `rhs` while ignoring case
    pub fn equals_ignore_case<'a>(&self, value: &str) -> Binary<Column<I>, Value<'a>> {
        self.like_ignore_case(escape_like(value))
    }
}

/// Helper function to reduce repetition
fn string_binary<'a, I: FieldProxyImpl>(
    operator: BinaryOperator,
    value: impl Str<'a>,
) -> Binary<Column<I>, Value<'a>> {
    Binary {
        operator,
        fst_arg: Column(proxy::new()),
        snd_arg: value.into_value(),
    }
}

/// Escape the special character from an argument to `LIKE`
fn escape_like(string: &str) -> String {
    string
        .replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

/// A `String`, `&str` or `Cow<str>`
pub trait Str<'str> {
    sealed!(trait);
    #[doc(hidden)]
    fn into_value(self) -> Value<'str>;
}
impl<'str> Str<'str> for String {
    sealed!(impl);
    fn into_value(self) -> Value<'str> {
        Value::String(Cow::Owned(self))
    }
}
impl<'str> Str<'str> for &'str str {
    sealed!(impl);
    fn into_value(self) -> Value<'str> {
        Value::String(Cow::Borrowed(self))
    }
}
impl<'str> Str<'str> for &'str String {
    sealed!(impl);
    fn into_value(self) -> Value<'str> {
        Value::String(Cow::Borrowed(self))
    }
}
impl<'str> Str<'str> for Cow<'str, str> {
    sealed!(impl);
    fn into_value(self) -> Value<'str> {
        Value::String(self)
    }
}
