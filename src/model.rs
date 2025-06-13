//! This module holds traits and structs for working with models

use rorm_declaration::imr;

use crate::conditions::{Binary, BinaryOperator, Column, Value};
use crate::crud::selector::Selector;
use crate::fields::proxy;
use crate::fields::utils::column_name::ColumnName;
use crate::internal::field::{Field, SingleColumnField};
use crate::internal::hmr::{AsImr, Source};
use crate::internal::relation_path::Path;
use crate::internal::ConstRef;

/// Trait implemented on Patches i.e. a subset of a model's fields.
///
/// Implemented by [`derive(Patch)`] as well as [`derive(Model)`].
pub trait Patch: Sized + 'static {
    /// The model this patch is for
    type Model: Model;

    /// Enum implementing the "value space" representation of the patch
    ///
    /// This is more of an implementation detail of the derive macro.
    type ValueSpaceImpl: Selector<Result = Self, Model = Self::Model> + Default;

    /// Create a `Vec` containing the patch's columns
    fn columns() -> Vec<ColumnName> {
        let mut columns = Vec::new();
        Self::push_columns(&mut columns);
        columns
    }

    /// Push the patch's columns onto a `Vec`
    fn push_columns(columns: &mut Vec<ColumnName>);

    /// Create a [`Vec`] moving the patch's condition values
    fn values(self) -> Vec<Value<'static>> {
        let mut values = Vec::new();
        self.push_values(&mut values);
        values
    }

    /// Push the patch's condition values onto a [`Vec`]
    fn push_values(self, values: &mut Vec<Value>);

    /// Create a [`Vec`] borrowing the patch's condition values
    fn references(&self) -> Vec<Value<'_>> {
        let mut values = Vec::new();
        self.push_references(&mut values);
        values
    }

    /// Push the patch's condition values onto a [`Vec`]
    fn push_references<'a>(&'a self, values: &mut Vec<Value<'a>>);
}

/// The [Condition](crate::conditions::Condition) type returned by [Identifiable::as_condition]
pub type PatchAsCondition<'a, P> =
    Binary<Column<(<<P as Patch>::Model as Model>::Primary, <P as Patch>::Model)>, Value<'a>>;

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`](rorm_macro::Model).
pub trait Model: Patch<Model = Self> {
    /// The primary key
    type Primary: Field<Model = Self> + SingleColumnField;

    /// A struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    ///
    /// The struct is constructed once in the [`Model::FIELDS`] constant.
    type Fields<P: Path>: ConstRef;

    /// A constant struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    const FIELDS: Self::Fields<Self>;

    /// Shorthand version of [`FIELDS`]
    ///
    /// [`FIELDS`]: Model::FIELDS
    #[deprecated(note = "Use `Model.field` instead of `Model::F.field`")]
    const F: Self::Fields<Self>;

    /// The model's table name
    const TABLE: &'static str;

    /// Location of the model in the source code
    const SOURCE: Source;

    /// Push the model's fields' imr representation onto a vec
    fn push_fields_imr(fields: &mut Vec<imr::Field>);

    /// Returns the model's intermediate representation
    ///
    /// As library user you probably won't need this. You might want to look at [`write_models`].
    ///
    /// [`write_models`]: crate::write_models
    fn get_imr() -> imr::Model {
        let mut fields = Vec::new();
        Self::push_fields_imr(&mut fields);
        imr::Model {
            name: Self::TABLE.to_string(),
            fields,
            source_defined_at: Some(Self::SOURCE.as_imr()),
        }
    }
}

/// Expose a models' fields on the type level using indexes
pub trait FieldByIndex<const INDEX: usize>: Model {
    /// The model's field at `INDEX`
    type Field: Field<Model = Self>;
}

/// Generic access to a patch's fields
///
/// This enables generic code to check if a patch contains a certain field
/// (for example the model's primary key, see [Identifiable])
/// and gain access to it.
pub trait GetField<F: Field>: Patch {
    /// Take the field by ownership
    fn get_field(self) -> F::Type;

    /// Borrow the field
    fn borrow_field(&self) -> &F::Type;

    /// Borrow the field mutably
    fn borrow_field_mut(&mut self) -> &mut F::Type;
}

/// Update a model's field based on the model's primary key
///
/// This trait is similar to [`GetField::borrow_field_mut`].
/// But [`GetField::borrow_field_mut`] only allows access to one field at a time,
/// because the method hides the fact, that the mutual borrow only applies to a single field.
/// This trait provides a solution to this problem, for a common scenario:
/// The need for an additional immutable borrow to the primary key.
pub trait UpdateField<F: Field<Model = Self>>: Model {
    /// Update a model's field based on the model's primary key
    fn update_field<'m, T>(
        &'m mut self,
        update: impl FnOnce(&'m <<Self as Model>::Primary as Field>::Type, &'m mut F::Type) -> T,
    ) -> T;
}

/// A patch which contains its model's primary key.
pub trait Identifiable: Patch {
    /// Get a reference to the primary key
    fn get_primary_key(&self) -> &<<Self::Model as Model>::Primary as Field>::Type;

    /// Build a [Condition](crate::conditions::Condition)
    /// which only applies to this instance by comparing the primary key.
    fn as_condition(&self) -> PatchAsCondition<'_, Self> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column(proxy::new()),
            snd_arg: <Self::Model as Model>::Primary::type_as_value(self.get_primary_key()),
        }
    }
}

impl<M: Model, P: Patch<Model = M> + GetField<M::Primary>> Identifiable for P {
    fn get_primary_key(&self) -> &<M::Primary as Field>::Type {
        <Self as GetField<M::Primary>>::borrow_field(self)
    }
}
