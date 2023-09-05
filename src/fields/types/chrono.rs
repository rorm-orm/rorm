use std::marker::PhantomData;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use rorm_db::sql::value::NullType;
use rorm_db::Error::DecodeError;
use rorm_db::{Error, Row};
use rorm_declaration::imr;

use crate::conditions::Value;
use crate::crud::decoder::{Decoder, DirectDecoder};
use crate::fields::traits::FieldType;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::{kind, AbstractField, ContainerField, FieldProxy, RawField};
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::{db_type, Source};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;
use crate::model::ConstNew;
use crate::{
    const_concat, impl_AsDbType, impl_FieldEq, impl_FieldOrd, new_converting_decoder, sealed,
};

impl_AsDbType!(NaiveTime, db_type::Time, Value::ChronoNaiveTime);
impl_AsDbType!(NaiveDate, db_type::Date, Value::ChronoNaiveDate);
impl_AsDbType!(NaiveDateTime, db_type::DateTime, Value::ChronoNaiveDateTime);
impl_AsDbType!(DateTime<Utc>, db_type::DateTime, Value::ChronoDateTime);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, NaiveTime> for NaiveTime         { Value::ChronoNaiveTime });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, NaiveDate> for NaiveDate         { Value::ChronoNaiveDate });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, NaiveDateTime> for NaiveDateTime { Value::ChronoNaiveDateTime });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, DateTime<Utc>> for DateTime<Utc> { Value::ChronoDateTime });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<NaiveTime>> for Option<NaiveTime>         { |option: Self| option.map(Value::ChronoNaiveTime).unwrap_or(Value::Null(NullType::ChronoNaiveTime)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<NaiveDate>> for Option<NaiveDate>         { |option: Self| option.map(Value::ChronoNaiveDate).unwrap_or(Value::Null(NullType::ChronoNaiveDate)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<NaiveDateTime>> for Option<NaiveDateTime> { |option: Self| option.map(Value::ChronoNaiveDateTime).unwrap_or(Value::Null(NullType::ChronoNaiveDateTime)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<DateTime<Utc>>> for Option<DateTime<Utc>> { |option: Self| option.map(Value::ChronoDateTime).unwrap_or(Value::Null(NullType::ChronoDateTime)) });
impl_FieldOrd!(NaiveTime, NaiveTime, Value::ChronoNaiveTime);
impl_FieldOrd!(NaiveDate, NaiveDate, Value::ChronoNaiveDate);
impl_FieldOrd!(NaiveDateTime, NaiveDateTime, Value::ChronoNaiveDateTime);
impl_FieldOrd!(DateTime<Utc>, DateTime<Utc>, Value::ChronoDateTime);
#[rustfmt::skip]
impl_FieldOrd!(Option<NaiveTime>, Option<NaiveTime>, |option: Self| option.map(Value::ChronoNaiveTime).unwrap_or(Value::Null(NullType::ChronoNaiveTime)));
#[rustfmt::skip]
impl_FieldOrd!(Option<NaiveDate>, Option<NaiveDate>, |option: Self| option.map(Value::ChronoNaiveDate).unwrap_or(Value::Null(NullType::ChronoNaiveDate)));
#[rustfmt::skip]
impl_FieldOrd!(Option<NaiveDateTime>, Option<NaiveDateTime>, |option: Self| option.map(Value::ChronoNaiveDateTime).unwrap_or(Value::Null(NullType::ChronoNaiveDateTime)));
#[rustfmt::skip]
impl_FieldOrd!(Option<DateTime<Utc>>, Option<DateTime<Utc>>, |option: Self| option.map(Value::ChronoDateTime).unwrap_or(Value::Null(NullType::ChronoDateTime)));

/// Implementation detail of [`DateTime<FixedOffset>`]
const _: () = {
    new_converting_decoder!(
        /// [`FieldDecoder`] for [`FixedOffset`]
        FixedOffsetDecoder,
        |value: i32| -> FixedOffset {
            FixedOffset::east_opt(value)
                .ok_or_else(|| DecodeError(format!("Couldn't decode fixed offset: {value}")))
        }
    );
    impl FieldType for FixedOffset {
        type Kind = kind::AsDbType;

        type Columns<T> = [T; 1];

        fn into_values(self) -> Self::Columns<Value<'static>> {
            [Value::I32(self.local_minus_utc())]
        }

        fn as_values(&self) -> Self::Columns<Value<'_>> {
            [Value::I32(self.local_minus_utc())]
        }

        type Decoder = FixedOffsetDecoder;
    }
    impl AsDbType for FixedOffset {
        type Primitive = i32;
        type DbType = db_type::Int32;

        fn from_primitive(primitive: Self::Primitive) -> Self {
            FixedOffset::east_opt(primitive).unwrap() // TODO handle this
        }
    }
    new_converting_decoder!(
        /// [`FieldDecoder`] for [`Option<FixedOffset>`](FixedOffset)
        OptionFixedOffsetDecoder,
        |value: Option<i32>| -> Option<FixedOffset> {
            value
                .map(|value| {
                    FixedOffset::east_opt(value).ok_or_else(|| {
                        DecodeError(format!("Couldn't decode fixed offset: {value}"))
                    })
                })
                .transpose()
        }
    );
    impl_AsDbType!(Option<FixedOffset>, OptionFixedOffsetDecoder);

    impl FieldType for DateTime<FixedOffset> {
        type Kind = kind::DateTime;

        type Columns<T> = [T; 2];

        fn into_values(self) -> Self::Columns<Value<'static>> {
            let [offset] = self.offset().into_values();
            [offset, Value::ChronoNaiveDateTime(self.naive_utc())]
        }

        fn as_values(&self) -> Self::Columns<Value<'_>> {
            let [offset] = self.offset().as_values();
            [offset, Value::ChronoNaiveDateTime(self.naive_utc())]
        }

        type Decoder = DateTimeDecoder;
    }

    /// [`FieldDecoder`] for [`DateTime<FixedOffset>`]
    pub struct DateTimeDecoder {
        offset: FixedOffsetDecoder,
        utc: DirectDecoder<NaiveDateTime>,
    }
    impl Decoder for DateTimeDecoder {
        type Result = DateTime<FixedOffset>;

        fn by_name(&self, row: &Row) -> Result<Self::Result, Error> {
            let offset = self.offset.by_name(row)?;
            let utc = self.utc.by_name(row)?;
            Ok(offset.from_utc_datetime(&utc))
        }

        fn by_index(&self, row: &Row) -> Result<Self::Result, Error> {
            let offset = self.offset.by_index(row)?;
            let utc = self.utc.by_index(row)?;
            Ok(offset.from_utc_datetime(&utc))
        }
    }
    impl FieldDecoder for DateTimeDecoder {
        fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
        where
            F: RawField<Type = Self::Result>,
            P: Path,
        {
            Self {
                offset: FixedOffsetDecoder::new(ctx, FieldProxy::<__DateTime_offset<F>, P>::new()),
                utc: DirectDecoder::new(ctx, FieldProxy::<__DateTime_utc<F>, P>::new()),
            }
        }
    }
    impl<F> AbstractField<kind::DateTime> for F
    where
        F: RawField<Kind = kind::DateTime, Type = DateTime<FixedOffset>>,
    {
        sealed!(impl);

        fn push_imr(self, imr: &mut Vec<imr::Field>) {
            __DateTime_offset::<F>::new().push_imr(imr);
            __DateTime_utc::<F>::new().push_imr(imr);
        }

        const COLUMNS: &'static [&'static str] =
            &[__DateTime_offset::<F>::NAME, __DateTime_utc::<F>::NAME];
    }
    impl<F, P> ContainerField<P, kind::DateTime> for F
    where
        P: Path,
        F: RawField<Kind = kind::DateTime, Type = DateTime<FixedOffset>>,
    {
        type Target = __DateTime_Fields<F, P>;
    }

    /// [`DateTime<FixedOffset>`]'s internal fields
    #[allow(non_camel_case_types)]
    pub struct __DateTime_Fields<F, P> {
        /// [`DateTime<FixedOffset>`]'s internal offset field
        pub offset: FieldProxy<__DateTime_offset<F>, P>,

        /// [`DateTime<FixedOffset>`]'s internal offset field
        pub utc: FieldProxy<__DateTime_utc<F>, P>,
    }
    impl<F, P: 'static> ConstNew for __DateTime_Fields<F, P>
    where
        F: RawField<Kind = kind::DateTime, Type = DateTime<FixedOffset>>,
    {
        const NEW: Self = __DateTime_Fields {
            offset: FieldProxy::new(),
            utc: FieldProxy::new(),
        };
        const REF: &'static Self = &Self::NEW;
    }

    /// [`DateTime<FixedOffset>`]'s internal offset field
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct __DateTime_offset<F>(PhantomData<F>);
    impl<F> RawField for __DateTime_offset<F>
    where
        F: RawField<Type = DateTime<FixedOffset>>,
    {
        type Kind = kind::AsDbType;
        type Type = FixedOffset;
        type Model = F::Model;
        const INDEX: usize = F::INDEX + 1;
        const NAME: &'static str = const_concat!(&[F::NAME, "_offset"]);
        const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
        const SOURCE: Option<Source> = F::SOURCE;
        fn new() -> Self {
            Self(PhantomData)
        }
    }

    /// [`DateTime<FixedOffset>`]'s internal offset field
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct __DateTime_utc<F>(PhantomData<F>);
    impl<F> RawField for __DateTime_utc<F>
    where
        F: RawField<Type = DateTime<FixedOffset>>,
    {
        type Kind = kind::AsDbType;
        type Type = NaiveDateTime;
        type Model = F::Model;
        const INDEX: usize = F::INDEX + 2;
        const NAME: &'static str = const_concat!(&[F::NAME, "_utc"]);
        const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
        const SOURCE: Option<Source> = F::SOURCE;
        fn new() -> Self {
            Self(PhantomData)
        }
    }
};
