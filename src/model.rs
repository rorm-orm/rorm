//! This module holds traits and structs for working with models

use rorm_db::row::FromRow;
use rorm_declaration::imr;

use crate::conditions::{Binary, BinaryOperator, Column, Value};
use crate::internal::field::{Field, RawField};
use crate::internal::relation_path::Path;

/// Trait implemented on Patches i.e. a subset of a model's fields.
///
/// Implemented by [`derive(Patch)`] as well as [`derive(Model)`].
pub trait Patch: FromRow + 'static {
    /// The model this patch is for
    type Model: Model;

    /// List of columns i.e. fields this patch contains
    const COLUMNS: &'static [&'static str];

    /// List of fields' indexes this patch contains
    ///
    /// Used in [`contains_index`]
    const INDEXES: &'static [usize];

    /// Create a [`Vec`] containing the patch's condition values
    ///
    /// These can be used to insert the patch.
    fn values(&self) -> Vec<Value> {
        let mut values = Vec::new();
        self.push_values(&mut values);
        values
    }

    /// Push the patch's condition values onto a [`Vec`]
    fn push_values<'a>(&'a self, values: &mut Vec<Value<'a>>);

    /// Get a reference to a field
    fn field<F>(&self) -> &F::Type
    where
        F: RawField,
        Self: GetField<F>,
    {
        <Self as GetField<F>>::get_field(self)
    }

    /// Get a mutable reference to a field
    fn field_mut<F>(&mut self) -> &mut F::Type
    where
        F: RawField,
        Self: GetField<F>,
    {
        <Self as GetField<F>>::get_field_mut(self)
    }
}

/// The [Condition](crate::conditions::Condition) type returned by [Identifiable::as_condition]
pub type PatchAsCondition<'a, P> =
    Binary<Column<<<P as Patch>::Model as Model>::Primary, <P as Patch>::Model>, Value<'a>>;

/// Check whether a [`Patch`] contains a certain field index.
///
/// This function in const and can therefore check the existence of fields at compile time.
pub const fn contains_index<P: Patch>(field: usize) -> bool {
    let mut indexes = P::INDEXES;
    while let [index, remaining @ ..] = indexes {
        indexes = remaining;
        if *index == field {
            return true;
        }
    }
    false
}

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`](rorm_macro::Model).
pub trait Model: Patch<Model = Self> {
    /// The primary key
    type Primary: Field<Model = Self>;

    /// A struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    ///
    /// The struct is constructed once in the [`Model::FIELDS`] constant.
    type Fields<P: Path>: ConstNew;

    /// A constant struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    const FIELDS: Self::Fields<Self>;

    /// Shorthand version of [`FIELDS`]
    ///
    /// [`FIELDS`]: Model::FIELDS
    const F: Self::Fields<Self>;

    /// The model's table name
    const TABLE: &'static str;

    /// Returns the model's intermediate representation
    ///
    /// As library user you probably won't need this. You might want to look at [`write_models`].
    ///
    /// [`write_models`]: crate::write_models
    fn get_imr() -> imr::Model;
}

/// Expose a models' fields on the type level using indexes
pub trait FieldByIndex<const INDEX: usize>: Model {
    /// The model's field at `INDEX`
    type Field: RawField<Model = Self>;
}

/// Generic access to a patch's fields
///
/// This enables generic code to check if a patch contains a certain field
/// (for example the model's primary key, see [Identifiable])
/// and gain access to it.
pub trait GetField<F: RawField>: Patch {
    /// Get reference to the field
    fn get_field(&self) -> &F::Type;

    /// Get mutable reference to the field
    fn get_field_mut(&mut self) -> &mut F::Type;
}

/// Update a model's field based on the model's primary key
///
/// This trait is similar to [GetField::get_field_mut].
/// But [GetField::get_field_mut] only allows access to one field at a time,
/// because the method hides the fact, that the mutual borrow only applies to a single field.
/// This trait provides a solution to this problem, for a common scenario:
/// The need for an additional immutable borrow to the primary key.
pub trait UpdateField<F: RawField<Model = Self>>: Model {
    /// Update a model's field based on the model's primary key
    fn update_field<'m, T>(
        &'m mut self,
        update: impl FnOnce(&'m <<Self as Model>::Primary as RawField>::Type, &'m mut F::Type) -> T,
    ) -> T;
}

/// A patch which contains its model's primary key.
pub trait Identifiable: Patch {
    /// Get a reference to the primary key
    fn get_primary_key(&self) -> &<<Self::Model as Model>::Primary as RawField>::Type;

    /// Build a [Condition](crate::conditions::Condition)
    /// which only applies to this instance by comparing the primary key.
    fn as_condition(&self) -> PatchAsCondition<Self> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column::new(),
            snd_arg: <<Self::Model as Model>::Primary as Field>::as_condition_value(
                self.get_primary_key(),
            ),
        }
    }
}
impl<M: Model, P: Patch<Model = M> + GetField<M::Primary>> Identifiable for P {
    fn get_primary_key(&self) -> &<M::Primary as RawField>::Type {
        <Self as GetField<M::Primary>>::get_field(self)
    }
}

/// exposes a `NEW` constant, which act like [Default::default] but constant.
///
/// It's workaround for not having const methods in traits
pub trait ConstNew {
    /// A new or default instance
    const NEW: Self;
}
