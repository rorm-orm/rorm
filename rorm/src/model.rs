use crate::conditions::{Binary, BinaryOperator, Column, Value};
use crate::internal::field::Field;
use rorm_db::row::FromRow;
use rorm_declaration::imr;

/// Map a rust enum, whose variant don't hold any data, into a database enum
///
/// ```rust
/// #[derive(Copy, Clone, rorm::DbEnum)]
/// pub enum Gender {
///     Male,
///     Female,
///     Other,
/// }
/// ```
pub trait DbEnum {
    /// Convert a string into its corresponding variant.
    ///
    /// # Panics
    /// Panics, if no variant matches. Since the string should only come from the db,
    /// a non matching string would indicate an invalid db state.
    fn from_str(string: &str) -> Self;

    /// Convert a variant into its corresponding string.
    fn to_str(&self) -> &'static str;

    /// A slice containing all variants as strings.
    const CHOICES: &'static [&'static str];
}

/// Trait implemented on Patches i.e. a subset of a model's fields.
///
/// Implemented by [`derive(Patch)`] as well as [`derive(Model)`].
pub trait Patch: FromRow {
    /// The model this patch is for
    type Model: Model;

    /// List of columns i.e. fields this patch contains
    const COLUMNS: &'static [&'static str];

    /// List of fields' indexes this patch contains
    ///
    /// Used in [`contains_index`]
    const INDEXES: &'static [usize];

    /// Get a field's db value by its index
    fn get(&self, index: usize) -> Option<Value>;

    /// Build a [Condition](crate::conditions::Condition) which only matches on this instance.
    ///
    /// This method defaults to using the primary key.
    /// If the patch does not store the models primary key, this method will return `None`.
    fn as_condition(&self) -> Option<PatchAsCondition<Self>> {
        self.get(<<Self::Model as Model>::Primary as Field>::INDEX)
            .map(|value| Binary {
                operator: BinaryOperator::Equals,
                fst_arg: Column::new(),
                snd_arg: value,
            })
    }
}
/// The [Condition](crate::conditions::Condition) type returned by [Patch::as_condition]
pub type PatchAsCondition<'a, P> =
    Binary<Column<<<P as Patch>::Model as Model>::Primary, ()>, Value<'a>>;

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

/// Create an iterator from a patch which yield its fields as db values
///
/// This method can't be part of the [`Patch`] trait, since `impl Trait` is not allowed in traits.
pub fn iter_columns<P: Patch>(patch: &P) -> impl Iterator<Item = Value> {
    P::INDEXES.iter().filter_map(|&index| patch.get(index))
}

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`](rorm_macro::Model).
pub trait Model: Patch<Model = Self> {
    /// The primary key
    type Primary: Field;

    /// A struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    ///
    /// The struct is constructed once in the [`Model::FIELDS`] constant.
    type Fields<Path>: ConstNew;

    /// A constant struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    // Actually FIELDS is an alias for F instead of the other way around.
    // This changes was made in the hope it would improve IDE support.
    const FIELDS: Self::Fields<()> = Self::Fields::NEW;

    /// Shorthand version of [`FIELDS`]
    ///
    /// [`FIELDS`]: Model::FIELDS
    const F: Self::Fields<()> = Self::Fields::NEW;

    /// The model's table name
    const TABLE: &'static str;

    /// Returns the model's intermediate representation
    ///
    /// As library user you probably won't need this. You might want to look at [`write_models`].
    ///
    /// [`write_models`]: crate::write_models
    fn get_imr() -> imr::Model;
}

/// exposes a `NEW` constant, which act like [Default::default] but constant.
///
/// It's workaround for not having const methods in traits
pub trait ConstNew {
    /// A new or default instance
    const NEW: Self;
}

/// Stores a link to another model in a field.
///
/// In database language, this is a many to one relation.
pub enum ForeignModel<M: Model> {
    /// The other model's primary key which can be used to query it later.
    Key(<M::Primary as Field>::Type),
    /// The other model's queried instance.
    Instance(Box<M>),
}
impl<M: Model> Clone for ForeignModel<M>
where
    M: Clone,
    <M::Primary as Field>::Type: Clone,
{
    fn clone(&self) -> Self {
        match self {
            ForeignModel::Key(primary) => ForeignModel::Key(primary.clone()),
            ForeignModel::Instance(model) => ForeignModel::Instance(model.clone()),
        }
    }
}
