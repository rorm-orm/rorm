use std::borrow::Cow;

use rorm_declaration::imr;
use url::Url;

use crate::conditions::Value;
use crate::fields::traits::FieldType;
use crate::internal::field::as_db_type::{get_single_imr, AsDbType};
use crate::internal::field::modifier::{MergeAnnotations, SingleColumnCheck, SingleColumnFromName};
use crate::internal::field::Field;
use crate::internal::hmr;
use crate::{impl_FieldEq, new_converting_decoder, Error};

impl_FieldEq!(impl<'rhs> FieldEq<'rhs, &'rhs Url> for Url {|url: &'rhs Url| Value::String(Cow::Borrowed(url.as_str()))});
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Url> for Url {|url: Url| Value::String(Cow::Owned(url.into()))});

impl FieldType for Url {
    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [Value::String(Cow::Owned(self.into()))]
    }

    #[inline(always)]
    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [Value::String(Cow::Borrowed(self.as_str()))]
    }

    fn get_imr<F: Field<Type = Self>>() -> Self::Columns<imr::Field> {
        get_single_imr::<F>(imr::DbType::VarChar)
    }

    type Decoder = UrlDecoder;

    type AnnotationsModifier<F: Field<Type = Self>> = MergeAnnotations<Self>;

    type CheckModifier<F: Field<Type = Self>> = SingleColumnCheck<hmr::db_type::VarChar>;

    type ColumnsFromName<F: Field<Type = Self>> = SingleColumnFromName;
}
impl AsDbType for Url {
    type Primitive = String;

    type DbType = hmr::db_type::VarChar;
}
new_converting_decoder!(
    pub UrlDecoder,
    |value: String| -> Url {
        Url::parse(&value).map_err(|err| Error::DecodeError(format!("Couldn't parse url: {err}")))
    }
);

impl FieldType for Option<Url> {
    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        self.map(<Url>::into_values).unwrap_or([Value::Null(
            <<Url as AsDbType>::DbType as hmr::db_type::DbType>::NULL_TYPE,
        )])
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        self.as_ref().map(<Url>::as_values).unwrap_or([Value::Null(
            <<Url as AsDbType>::DbType as hmr::db_type::DbType>::NULL_TYPE,
        )])
    }

    fn get_imr<F: Field<Type = Self>>() -> Self::Columns<imr::Field> {
        get_single_imr::<F>(imr::DbType::VarChar)
    }

    type Decoder = OptionUrlDecoder;

    type AnnotationsModifier<F: Field<Type = Self>> = MergeAnnotations<Self>;

    type CheckModifier<F: Field<Type = Self>> = SingleColumnCheck<<Url as AsDbType>::DbType>;

    type ColumnsFromName<F: Field<Type = Self>> = SingleColumnFromName;
}
impl AsDbType for Option<Url> {
    type Primitive = Option<<Url as AsDbType>::Primitive>;
    type DbType = <Url as AsDbType>::DbType;

    const IMPLICIT: Option<hmr::annotations::Annotations> = {
        let mut annos = if let Some(annos) = <Url as AsDbType>::IMPLICIT {
            annos
        } else {
            hmr::annotations::Annotations::empty()
        };
        annos.nullable = true;
        Some(annos)
    };
}
new_converting_decoder!(
    pub OptionUrlDecoder,
    |value: Option<String>| -> Option<Url> {
        value.map(|string| Url::parse(&string)).transpose().map_err(|err| Error::DecodeError(format!("Couldn't parse url: {err}")))
    }
);
