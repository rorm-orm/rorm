use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;

use rorm_db::row::RowError;
use rorm_db::sql::value::NullType;
use rorm_db::Row;
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::conditions::Value;
use crate::crud::decoder::Decoder;
use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::fields::traits::into_value::IntoValue;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::simple::SimpleFieldILike;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldLike};
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::types::max_str_impl::{LenImpl, NumBytes};
use crate::fields::utils::check::shared_linter_check;
use crate::fields::utils::const_fn::Contains;
use crate::fields::utils::get_annotations::merge_annotations;
use crate::fields::utils::get_names::single_column_name;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::{Annotations, MaxLength};
use crate::internal::query_context::QueryContext;

/// String which is restricted to a maximum length
///
/// When storing strings in a database you have to specify a `#[rorm(max_length = ...)]`
/// which is enforced by the database upon insertion or update.
/// This can result in a rather opaque [`rorm::Error`](crate::Error) when you
/// fail to check your strings before passing to the database.
/// This type forces you to perform this check before the database by having a fallible constructor.
///
/// The "length" of a string is not really a well-defined thing
/// and different databases might have different opinions.
/// So this type uses a generic `Impl: LenImpl` to select our databases definition of "length".
/// However, note that this will reduce our code's portability and is therefor not the recommended default.
///
/// This type is also generic over the string implementation to also support `&str` and `Cow<'_, str>`.
#[derive(Copy, Clone, Debug)]
pub struct MaxStr<const MAX_LEN: usize = 255, Impl = NumBytes, Str = String> {
    string: Str,
    len_impl: Impl,
}

impl<const MAX_LEN: usize, Impl, Str> Default for MaxStr<MAX_LEN, Impl, Str>
where
    Self: Sized,
    Str: Deref<Target = str> + Default,
    Impl: LenImpl + Default,
{
    /// Returns the “default value” for a type. [Read more](Default::default)
    ///
    /// # Panics
    /// If `Str::default` produces a value which is longer than `MAX_LEN`.
    fn default() -> Self {
        Self::new(Default::default())
            .unwrap_or_else(|_| panic!("A `Default` for a `Deref<Target = str>` should be empty"))
    }
}

impl<const MAX_LEN: usize, Impl, Str> MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
    Impl: LenImpl,
{
    /// Wraps a string returning `Err` if it is too long.
    pub fn new(string: Str) -> Result<Self, MaxLenError<Str>>
    where
        Impl: Default,
    {
        Self::with_impl(string, Impl::default())
    }

    /// Wraps a string using a custom [`LenImpl`] returning `None` if the string is too long.
    pub fn with_impl(string: Str, len_impl: Impl) -> Result<Self, MaxLenError<Str>> {
        let got = len_impl.len(&string);
        if got > MAX_LEN {
            Err(MaxLenError {
                string,
                max: MAX_LEN,
                got,
            })
        } else {
            Ok(Self { string, len_impl })
        }
    }

    /// Returns the length of the wrapped `Str`.
    ///
    /// This method replaces `str::len` which is exposed through `Deref<Target = str>`
    /// to return the length relevant to the limit.
    pub fn len(&self) -> usize {
        self.len_impl.len(&self.string)
    }

    /// Returns `true` if `self` has a [`len`](Self::len) of `0`
    ///
    /// Beware: The length is not necessarily in bytes but whatever the length implementation does.
    pub fn is_empty(&self) -> bool {
        self.len_impl.len(&self.string) == 0
    }

    /// Borrow the wrapped string while remembering its `MAX_LEN`.
    ///
    /// This method is the in inverse of [`MaxStr::to_owned`].
    pub fn as_ref(&self) -> MaxStr<MAX_LEN, Impl, &str>
    where
        Impl: Clone,
    {
        MaxStr {
            string: &self.string,
            len_impl: self.len_impl.clone(),
        }
    }
}
impl<const MAX_LEN: usize, Impl, Str> MaxStr<MAX_LEN, Impl, Str> {
    /// Get the actual string, discarding the length guarantee
    pub fn into_inner(self) -> Str {
        self.string
    }
}
impl<const MAX_LEN: usize> MaxStr<MAX_LEN, NumBytes, &'static str> {
    /// Converts a static string to a `MaxStr`.
    ///
    /// This function checks the static string to not exceed the `MAX_LEN`.
    ///
    /// # Panics
    /// When the static string is too long.
    pub const fn from_static(string: &'static str) -> Self {
        if string.len() > MAX_LEN {
            // The compiler will not evaluate the constant (and panic and compile-time)
            // if it can prove that the `if` branch is never taken.
            const {
                panic!("Static string is too long");
            }
        }
        Self {
            string,
            len_impl: NumBytes,
        }
    }
}

impl<const MAX_LEN: usize, Impl: Clone> MaxStr<MAX_LEN, Impl, &'_ str> {
    /// Allocates the wrapped string while remembering its `MAX_LEN`.
    ///
    /// This method is the in inverse of [`MaxStr::as_ref`].
    pub fn to_owned<Str>(&self) -> MaxStr<MAX_LEN, Impl, Str>
    where
        Str: for<'a> From<&'a str>,
    {
        MaxStr {
            string: self.string.into(),
            len_impl: self.len_impl.clone(),
        }
    }
}

/// Error returned by [`MaxStr`]'s constructors when the input string is too long
#[derive(Debug)]
pub struct MaxLenError<Str = String> {
    /// The rejected string
    pub string: Str,
    /// The maximum length which was exceeded
    pub max: usize,
    /// The `string`'s length (according to the [`LenImpl`] which was used)
    pub got: usize,
}

impl<Str: Deref<Target = str>> fmt::Display for MaxLenError<Str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "string is longer than {max}", max = self.max)
    }
}

impl<Str: fmt::Debug + Deref<Target = str>> std::error::Error for MaxLenError<Str> {}

impl<const MAX_LEN: usize, Impl, Str> Deref for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl<const MAX_LEN: usize, Impl, Str> Borrow<str> for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn borrow(&self) -> &str {
        &self.string
    }
}

impl<const MAX_LEN: usize, Impl, Str> Eq for MaxStr<MAX_LEN, Impl, Str> where
    Str: Deref<Target = str>
{
}

impl<const MAX_LEN: usize, Impl, Str> PartialEq for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn eq(&self, other: &Self) -> bool {
        *self.string == *other.string
    }
}

impl<const MAX_LEN: usize, Impl, Str> Ord for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let this = &*self.string;
        let other = &*other.string;
        this.cmp(other)
    }
}

impl<const MAX_LEN: usize, Impl, Str> PartialOrd for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const MAX_LEN: usize, Impl, Str> Hash for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let this = &*self.string;
        this.hash(state)
    }
}

impl<const MAX_LEN: usize, Impl, Str> fmt::Display for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &*self.string;
        this.fmt(f)
    }
}

impl<const MAX_LEN: usize, Impl, Str> Serialize for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.string.serialize(serializer)
    }
}

impl<'de, const MAX_LEN: usize, Impl, Str> Deserialize<'de> for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str> + Deserialize<'de>,
    Impl: LenImpl + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(Str::deserialize(deserializer)?).map_err(|error| {
            <D::Error as serde::de::Error>::invalid_value(
                Unexpected::Str(&error.string),
                &format!("string with a maximum length of {MAX_LEN}").as_str(),
            )
        })
    }
}

impl<const MAX_LEN: usize, Impl> FieldType for MaxStr<MAX_LEN, Impl, String>
where
    Impl: LenImpl + Default + 'static,
{
    type Columns = Array<1>;

    const NULL: FieldColumns<Self, NullType> = [NullType::String];

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::String(Cow::Owned(self.string))]
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::String(Cow::Borrowed(&self.string))]
    }

    type Decoder = MaxStrDecoder<MAX_LEN, Impl>;
    type GetAnnotations = merge_annotations<ImplicitMaxLength<MAX_LEN>>;
    type Check = shared_linter_check<1>;
    type GetNames = single_column_name;
}

pub struct MaxStrDecoder<const MAX_LEN: usize, Impl: LenImpl> {
    column: String,
    index: usize,
    generics: PhantomData<Impl>,
}

impl<const MAX_LEN: usize, Impl> Decoder for MaxStrDecoder<MAX_LEN, Impl>
where
    Impl: LenImpl + Default,
{
    type Result = MaxStr<MAX_LEN, Impl, String>;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        MaxStr::<MAX_LEN, Impl, String>::new(row.get(self.column.as_str())?).map_err(|error| {
            RowError::Decode {
                index: self.column.as_str().into(),
                source: error.into(),
            }
        })
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        MaxStr::<MAX_LEN, Impl, String>::new(row.get(self.index)?).map_err(|error| {
            RowError::Decode {
                index: self.index.into(),
                source: error.into(),
            }
        })
    }
}

impl<const MAX_LEN: usize, Impl> FieldDecoder for MaxStrDecoder<MAX_LEN, Impl>
where
    Impl: LenImpl + Default,
{
    fn new<I>(ctx: &mut QueryContext, _: FieldProxy<I>) -> Self
    where
        I: FieldProxyImpl<Field: Field<Type = Self::Result>>,
    {
        let (index, column) = ctx.select_field::<I::Field, I::Path>();
        Self {
            column,
            index,
            generics: PhantomData,
        }
    }
}

/// Type passed to [`merge_annotations`] to set the `max_length` annotation;
pub struct ImplicitMaxLength<const MAX_LEN: usize>;
impl<const MAX_LEN: usize> Contains<Annotations> for ImplicitMaxLength<MAX_LEN> {
    const ITEM: Annotations = {
        let mut annos = Annotations::empty();
        annos.max_length = Some(MaxLength(MAX_LEN as i32));
        annos
    };
}

impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldEq<'rhs, &'rhs str> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldIn<'rhs, &'rhs str> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldEq<'rhs, &'rhs String> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldIn<'rhs, &'rhs String> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldEq<'rhs, String> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldIn<'rhs, String> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldEq<'rhs, Cow<'rhs, str>>
    for MaxStr<MAX_LEN, Impl>
{
}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldIn<'rhs, Cow<'rhs, str>>
    for MaxStr<MAX_LEN, Impl>
{
}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldEq<'rhs, &'rhs Self> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldIn<'rhs, &'rhs MaxStr<MAX_LEN, Impl>>
    for MaxStr<MAX_LEN, Impl>
{
}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldEq<'rhs, Self> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldIn<'rhs, MaxStr<MAX_LEN, Impl>>
    for MaxStr<MAX_LEN, Impl>
{
}

impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldLike<'rhs, &'rhs str> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldLike<'rhs, &'rhs String>
    for MaxStr<MAX_LEN, Impl>
{
}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldLike<'rhs, String> for MaxStr<MAX_LEN, Impl> {}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldLike<'rhs, Cow<'rhs, str>>
    for MaxStr<MAX_LEN, Impl>
{
}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldLike<'rhs, &'rhs MaxStr<MAX_LEN, Impl>>
    for MaxStr<MAX_LEN, Impl>
{
}
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldLike<'rhs, MaxStr<MAX_LEN, Impl>>
    for MaxStr<MAX_LEN, Impl>
{
}

#[cfg(feature = "postgres-only")]
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldILike<'rhs, &'rhs str> for MaxStr<MAX_LEN, Impl> {}
#[cfg(feature = "postgres-only")]
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldILike<'rhs, &'rhs String>
    for MaxStr<MAX_LEN, Impl>
{
}
#[cfg(feature = "postgres-only")]
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldILike<'rhs, String> for MaxStr<MAX_LEN, Impl> {}
#[cfg(feature = "postgres-only")]
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldILike<'rhs, Cow<'rhs, str>>
    for MaxStr<MAX_LEN, Impl>
{
}
#[cfg(feature = "postgres-only")]
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldILike<'rhs, &'rhs MaxStr<MAX_LEN, Impl>>
    for MaxStr<MAX_LEN, Impl>
{
}
#[cfg(feature = "postgres-only")]
impl<'rhs, const MAX_LEN: usize, Impl> SimpleFieldILike<'rhs, MaxStr<MAX_LEN, Impl>>
    for MaxStr<MAX_LEN, Impl>
{
}

impl<'a, const MAX_LEN: usize, Impl, Str> IntoValue<'a> for MaxStr<MAX_LEN, Impl, Str>
where
    Str: Into<String>,
{
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Owned(self.into_inner().into()))
    }
}
impl<'a, const MAX_LEN: usize, Impl, Str> IntoValue<'a> for &'a MaxStr<MAX_LEN, Impl, Str>
where
    Str: Deref<Target = str>,
{
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Borrowed(&self.string))
    }
}

#[cfg(feature = "utoipa")]
mod utoipa_impl {
    use utoipa::openapi::{Object, RefOr, Schema, SchemaType};
    use utoipa::ToSchema;

    use crate::fields::types::max_str_impl::LenImpl;
    use crate::fields::types::MaxStr;

    impl<'s, const MAX_LEN: usize, Impl: LenImpl> ToSchema<'s> for MaxStr<MAX_LEN, Impl, String> {
        fn schema() -> (&'s str, RefOr<Schema>) {
            (
                "MaxStr",
                RefOr::T(Schema::Object(Object::with_type(SchemaType::String))),
            )
        }
    }
}

#[cfg(feature = "schemars")]
mod schemars_impl {
    use schemars::gen::SchemaGenerator;
    use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
    use schemars::JsonSchema;

    use crate::fields::types::MaxStr;

    impl<const MAX_LEN: usize> JsonSchema for MaxStr<MAX_LEN> {
        fn schema_name() -> String {
            format!("MaxStr_{MAX_LEN}")
        }

        fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
            let mut object = SchemaObject::default();
            object.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::String)));
            object.string().max_length = Some(MAX_LEN as u32);
            Schema::Object(object)
        }
    }
}
