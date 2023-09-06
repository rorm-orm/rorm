//! Implementation detail of [`ForeignModelByField`]

use std::marker::PhantomData;

use rorm_db::{Error, Row};
use rorm_declaration::imr;

use crate::conditions::Value;
use crate::crud::decoder::Decoder;
use crate::fields::types::ForeignModelByField;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::modifier::{
    AnnotationsModifier, SingleColumnCheck, SingleColumnFromName,
};
use crate::internal::field::{FieldProxy, FieldType, RawField, SingleColumnField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::internal::hmr::{AsImr, Source};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;
use crate::model::{GetField, Model};
use crate::{impl_FieldEq, sealed};

impl<FF> FieldType for ForeignModelByField<FF>
where
    Self: ForeignModelTrait,
    FF: SingleColumnField,
    FF::Type: AsDbType,
    FF::Model: GetField<FF>, // always true
{
    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [FF::type_into_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.get_field(),
        })]
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [FF::type_as_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.borrow_field(),
        })]
    }

    fn get_imr<F: RawField<Type = Self>>() -> Self::Columns<imr::Field> {
        [imr::Field {
            name: F::NAME.to_string(),
            db_type: <Self as ForeignModelTrait>::DbType::IMR,
            annotations: F::EFFECTIVE_ANNOTATIONS
                .unwrap_or_else(Annotations::empty)
                .as_imr(),
            source_defined_at: F::SOURCE.map(|s| s.as_imr()),
        }]
    }

    type Decoder = ForeignModelByFieldDecoder<FF>;

    type AnnotationsModifier<F: RawField<Type = Self>> = ForeignAnnotations<Self>;

    type CheckModifier<F: RawField<Type = Self>> =
        SingleColumnCheck<<Self as ForeignModelTrait>::DbType>;

    type ColumnsFromName<F: RawField<Type = Self>> = SingleColumnFromName;
}

impl<FF> FieldType for Option<ForeignModelByField<FF>>
where
    Self: ForeignModelTrait,
    FF: SingleColumnField,
    FF::Type: AsDbType,
    FF::Model: GetField<FF>, // always true
    Option<FF::Type>: AsDbType,
{
    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        self.map(ForeignModelByField::into_values)
            .unwrap_or([Value::Null(
                <<Option<FF::Type> as AsDbType>::DbType>::NULL_TYPE,
            )])
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        self.as_ref()
            .map(ForeignModelByField::as_values)
            .unwrap_or([Value::Null(
                <<Option<FF::Type> as AsDbType>::DbType>::NULL_TYPE,
            )])
    }

    fn get_imr<F: RawField<Type = Self>>() -> Self::Columns<imr::Field> {
        [imr::Field {
            name: F::NAME.to_string(),
            db_type: <Self as ForeignModelTrait>::DbType::IMR,
            annotations: F::EFFECTIVE_ANNOTATIONS
                .unwrap_or_else(Annotations::empty)
                .as_imr(),
            source_defined_at: F::SOURCE.map(|s| s.as_imr()),
        }]
    }

    type Decoder = OptionForeignModelByFieldDecoder<FF>;

    type AnnotationsModifier<F: RawField<Type = Self>> = ForeignAnnotations<Self>;

    type CheckModifier<F: RawField<Type = Self>> =
        SingleColumnCheck<<Self as ForeignModelTrait>::DbType>;

    type ColumnsFromName<F: RawField<Type = Self>> = SingleColumnFromName;
}

#[doc(hidden)]
pub trait ForeignModelTrait {
    sealed!(trait);

    type DbType: DbType;
    type RelatedField: SingleColumnField;
    const IS_OPTION: bool;
    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type>;
}

impl<FF> ForeignModelTrait for ForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: AsDbType,
    FF::Model: GetField<FF>, // always true
{
    sealed!(impl);

    type DbType = <FF::Type as AsDbType>::DbType;
    type RelatedField = FF;
    const IS_OPTION: bool = false;
    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type> {
        Some(match self {
            ForeignModelByField::Key(key) => key,
            ForeignModelByField::Instance(instance) => instance.borrow_field(),
        })
    }
}

impl<FF: SingleColumnField> ForeignModelTrait for Option<ForeignModelByField<FF>>
where
    FF: SingleColumnField,
    FF::Type: AsDbType,
    FF::Model: GetField<FF>, // always true
    Option<FF::Type>: AsDbType,
{
    sealed!(impl);

    type DbType = <FF::Type as AsDbType>::DbType;
    type RelatedField = FF;
    const IS_OPTION: bool = true;

    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type> {
        self.as_ref().map(|value| match value {
            ForeignModelByField::Key(key) => key,
            ForeignModelByField::Instance(instance) => instance.borrow_field(),
        })
    }
}

/// [`AnnotationsModifier`] which:
/// - sets `nullable`
/// - copies `max_length` from the foreign key
/// - sets `foreign`
pub struct ForeignAnnotations<T: ForeignModelTrait>(pub PhantomData<T>);
impl<T: ForeignModelTrait, F: RawField<Type = T>> AnnotationsModifier<F> for ForeignAnnotations<T> {
    const MODIFIED: Option<Annotations> = {
        let mut annos = F::EXPLICIT_ANNOTATIONS;
        annos.nullable = T::IS_OPTION;
        if annos.max_length.is_none() {
            if let Some(target_annos) = <RF<F> as RawField>::EFFECTIVE_ANNOTATIONS {
                annos.max_length = target_annos.max_length;
            }
        }
        annos.foreign = Some(hmr::annotations::ForeignKey {
            table_name: <RF<F> as RawField>::Model::TABLE,
            column_name: <RF<F> as RawField>::NAME,
        });
        Some(annos)
    };
}

/// Marker trait without actual bounds for fields of type foreign model
pub trait ForeignModelField: RawField {
    sealed!(trait);
}

pub(crate) type RF<F> = <<F as RawField>::Type as ForeignModelTrait>::RelatedField;
impl<F> ForeignModelField for F
where
    F: SingleColumnField,
    F::Type: ForeignModelTrait,
    <<F::Type as ForeignModelTrait>::RelatedField as RawField>::Model:
        GetField<<F::Type as ForeignModelTrait>::RelatedField>, // always true
{
    sealed!(impl);
}

/// [`FieldDecoder`] for [`ForeignModelByField<FF>`]
pub struct ForeignModelByFieldDecoder<FF: SingleColumnField>(<FF::Type as FieldType>::Decoder);
impl<FF: SingleColumnField> Decoder for ForeignModelByFieldDecoder<FF> {
    type Result = ForeignModelByField<FF>;

    fn by_name(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0.by_name(row).map(ForeignModelByField::Key)
    }

    fn by_index(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0.by_index(row).map(ForeignModelByField::Key)
    }
}
impl<FF: SingleColumnField> FieldDecoder for ForeignModelByFieldDecoder<FF> {
    fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path,
    {
        Self(FieldDecoder::new(
            ctx,
            FieldProxy::<FakeFieldType<FF::Type, F>, P>::new(),
        ))
    }
}

/// [`FieldDecoder`] for [`Option<ForeignModelByField<FF>>`](ForeignModelByField)
pub struct OptionForeignModelByFieldDecoder<FF: SingleColumnField>(
    <Option<FF::Type> as FieldType>::Decoder,
)
where
    Option<FF::Type>: FieldType;
impl<FF: SingleColumnField> Decoder for OptionForeignModelByFieldDecoder<FF>
where
    Option<FF::Type>: FieldType,
{
    type Result = Option<ForeignModelByField<FF>>;

    fn by_name(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0
            .by_name(row)
            .map(|option| option.map(ForeignModelByField::Key))
    }

    fn by_index(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0
            .by_index(row)
            .map(|option| option.map(ForeignModelByField::Key))
    }
}
impl<FF: SingleColumnField> FieldDecoder for OptionForeignModelByFieldDecoder<FF>
where
    Option<FF::Type>: FieldType,
{
    fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path,
    {
        Self(FieldDecoder::new(
            ctx,
            FieldProxy::<FakeFieldType<Option<FF::Type>, F>, P>::new(),
        ))
    }
}

/// Take a field `F` and create a new "fake" field with the different [`RawField::Type`](RawField::Type) `T`
#[allow(non_camel_case_types)]
struct FakeFieldType<T, F>(PhantomData<(T, F)>);
impl<T, F> Clone for FakeFieldType<T, F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, F> Copy for FakeFieldType<T, F> {}
impl<T, F> RawField for FakeFieldType<T, F>
where
    T: FieldType + 'static,
    F: RawField,
{
    type Type = T;
    type Model = F::Model;
    const INDEX: usize = F::INDEX;
    const NAME: &'static str = F::NAME;
    const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
    const SOURCE: Option<Source> = F::SOURCE;
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, FF::Type, FieldEq_ForeignModelByField_Owned> for ForeignModelByField<FF>
    where
        FF: SingleColumnField,
        FF::Type: AsDbType,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_into_value }
);
impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, FF::Type, FieldEq_ForeignModelByField_Owned> for Option<ForeignModelByField<FF>>
    where
        FF: SingleColumnField,
        FF::Type: AsDbType,
        Option<FF::Type>: AsDbType,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_into_value }
);

impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, &'rhs FF::Type, FieldEq_ForeignModelByField_Borrowed> for ForeignModelByField<FF>
    where
        FF: SingleColumnField,
        FF::Type: AsDbType,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_as_value }
);
impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, &'rhs FF::Type, FieldEq_ForeignModelByField_Borrowed> for Option<ForeignModelByField<FF>>
    where
        FF: SingleColumnField,
        FF::Type: AsDbType,
        Option<FF::Type>: AsDbType,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_as_value }
);

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_ForeignModelByField_Owned;
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_ForeignModelByField_Borrowed;
