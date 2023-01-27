use std::marker::PhantomData;

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use rorm_db::row::RowIndex;
use rorm_db::{Error, Row};
use rorm_declaration::imr;

use crate::conditions::Value;
use crate::const_concat;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{kind, AbstractField, FieldType, RawField};
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::{db_type, Source};

impl FieldType for FixedOffset {
    type Kind = kind::AsDbType;
}
impl AsDbType for FixedOffset {
    type Primitive = i32;
    type DbType = db_type::Int32;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        FixedOffset::east_opt(primitive).unwrap() // TODO handle this
    }

    fn as_primitive(&self) -> Value {
        Value::I32(self.local_minus_utc())
    }
}

impl FieldType for DateTime<FixedOffset> {
    type Kind = kind::DateTime;
}
impl<F> AbstractField<kind::DateTime> for F
where
    F: RawField<Kind = kind::DateTime, Type = DateTime<FixedOffset>>,
{
    fn push_imr(imr: &mut Vec<imr::Field>) {
        __DateTime_offset::<F>::push_imr(imr);
        __DateTime_utc::<F>::push_imr(imr);
    }

    fn get_from_row(_row: &Row, _index: impl RowIndex) -> Result<Self::Type, Error> {
        Err(Error::DecodeError("Not Implemented".to_string()))
    }
    fn get_by_name(row: &Row) -> Result<Self::Type, Error> {
        let offset = __DateTime_offset::<F>::get_by_name(row)?;
        let utc = __DateTime_utc::<F>::get_by_name(row)?;
        Ok(offset.from_utc_datetime(&utc))
    }

    fn push_value<'a>(value: &'a Self::Type, values: &mut Vec<Value<'a>>) {
        values.push(value.offset().as_primitive());
        values.push(Value::NaiveDateTime(value.naive_utc()));
    }

    const COLUMNS: &'static [&'static str] =
        &[__DateTime_offset::<F>::NAME, __DateTime_utc::<F>::NAME];
}

#[allow(non_camel_case_types)]
pub struct __DateTime_offset<F>(PhantomData<F>);
impl<F> RawField for __DateTime_offset<F>
where
    F: RawField<Kind = kind::DateTime, Type = DateTime<FixedOffset>>,
{
    type Kind = kind::AsDbType;
    type Type = FixedOffset;
    type ExplicitDbType = ();
    type Model = F::Model;
    const INDEX: usize = F::INDEX + 1;
    const NAME: &'static str = const_concat!(&[F::NAME, "_offset"]);
    const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
    const SOURCE: Option<Source> = F::SOURCE;
}

#[allow(non_camel_case_types)]
pub struct __DateTime_utc<F>(PhantomData<F>);
impl<F> RawField for __DateTime_utc<F>
where
    F: RawField<Kind = kind::DateTime, Type = DateTime<FixedOffset>>,
{
    type Kind = kind::AsDbType;
    type Type = NaiveDateTime;
    type ExplicitDbType = ();
    type Model = F::Model;
    const INDEX: usize = F::INDEX + 2;
    const NAME: &'static str = const_concat!(&[F::NAME, "_utc"]);
    const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
    const SOURCE: Option<Source> = F::SOURCE;
}
