use std::marker::PhantomData;

use rorm_db::conditional::{self, Condition};
use rorm_db::value::Value;
use rorm_declaration::hmr;
use rorm_declaration::imr;

/// This trait maps rust types to database types
pub trait AsDbType {
    /// A type which can be retrieved from the db and then converted into Self.
    type Primitive;

    /// The database type as defined in the Intermediate Model Representation
    type DbType: hmr::db_type::DbType;

    /// The default annotations' concrete [`Annotations<...>`] type
    ///
    /// [`Annotations<...>`]: hmr::annotations::Annotations
    type Annotations;

    /// the default annotations
    const ANNOTATIONS: Self::Annotations;

    /// Convert the associated primitive type into `Self`.
    ///
    /// This function allows "non-primitive" types like [GenericId] or any [DbEnum] to implement
    /// their decoding without access to the underlying db details (namely `sqlx::Decode`)
    fn from_primitive(primitive: Self::Primitive) -> Self;

    /// Convert a reference to `Self` into the primitive [`Value`] used by our db implementation.
    fn as_primitive(&self) -> Value;

    /// Whether this type supports null.
    const IS_NULLABLE: bool = false;
}

macro_rules! impl_as_db_type {
    ($type:ty, $db_type:ident, $value_variant:ident $(using $method:ident)?) => {
        impl AsDbType for $type {
            type Primitive = Self;

            type DbType = hmr::db_type::$db_type;

            type Annotations = hmr::annotations::Annotations<(), (), (), (), (), (), (), (), (), ()>;
            const ANNOTATIONS: Self::Annotations = hmr::annotations::Annotations::new();

            #[inline(always)]
            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive
            }

            impl_as_db_type!(impl_as_primitive, $type, $db_type, $value_variant $(using $method)?);
        }
    };
    (impl_as_primitive, $type:ty, $db_type:ident, $value_variant:ident) => {
        #[inline(always)]
        fn as_primitive(&self) -> Value {
            Value::$value_variant(*self)
        }
    };
    (impl_as_primitive, $type:ty, $db_type:ident, $value_variant:ident using $method:ident) => {
        #[inline(always)]
        fn as_primitive(&self) -> Value {
            Value::$value_variant(self.$method())
        }
    };
}
impl_as_db_type!(chrono::NaiveTime, Time, NaiveTime);
impl_as_db_type!(chrono::NaiveDateTime, DateTime, NaiveDateTime);
impl_as_db_type!(chrono::NaiveDate, Date, NaiveDate);
impl_as_db_type!(i16, Int16, I16);
impl_as_db_type!(i32, Int32, I32);
impl_as_db_type!(i64, Int64, I64);
impl_as_db_type!(f32, Float, F32);
impl_as_db_type!(f64, Double, F64);
impl_as_db_type!(bool, Boolean, Bool);
impl_as_db_type!(Vec<u8>, VarBinary, Binary using as_slice);
impl_as_db_type!(String, VarChar, String using as_str);
impl<T: AsDbType> AsDbType for Option<T> {
    type Primitive = Self;
    type DbType = T::DbType;

    type Annotations = T::Annotations;
    const ANNOTATIONS: Self::Annotations = T::ANNOTATIONS;

    #[inline(always)]
    fn from_primitive(primitive: Self::Primitive) -> Self {
        primitive
    }

    fn as_primitive(&self) -> Value {
        match self {
            Some(value) => value.as_primitive(),
            None => Value::Null,
        }
    }

    const IS_NULLABLE: bool = true;
}

/// Map a rust enum, whose variant don't hold any data, into a database enum
///
/// ```rust
/// #[derive(rorm::DbEnum)]
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
impl<E: DbEnum> AsDbType for E {
    type Primitive = String;
    type DbType = hmr::db_type::Choices;

    type Annotations = hmr::annotations::Annotations<
        (),
        (),
        (),
        hmr::annotations::Choices,
        (),
        (),
        (),
        (),
        (),
        (),
    >;
    const ANNOTATIONS: Self::Annotations =
        hmr::annotations::Annotations::new().choices(hmr::annotations::Choices(E::CHOICES));

    fn from_primitive(primitive: Self::Primitive) -> Self {
        E::from_str(&primitive)
    }

    fn as_primitive(&self) -> Value {
        Value::String(self.to_str())
    }
}

/// Trait implemented on Patches i.e. a subset of a model's fields.
///
/// Implemented by [`derive(Patch)`] as well as [`derive(Model)`].
pub trait Patch {
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

    /// Build a [Condition] which only matches on this instance.
    ///
    /// This method defaults to using the primary key.
    /// If the patch does not store the models primary key, this method will return `None`.
    fn as_condition(&self) -> Option<Condition> {
        self.get(Self::Model::PRIMARY.1).map(|value| {
            Condition::BinaryCondition(conditional::BinaryCondition::Equals(Box::new([
                Condition::Value(Value::Ident(Self::Model::PRIMARY.0)),
                Condition::Value(value),
            ])))
        })
    }
}

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
    P::INDEXES.iter().map(|&index| patch.get(index)).flatten()
}

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`].
///
/// [`derive(Model)`]: crate::Model
pub trait Model: Patch<Model = Self> {
    /// The primary key's name and index
    const PRIMARY: (&'static str, usize);

    /// A struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`]).
    ///
    /// The struct is constructed once in the [`Model::FIELDS`] constant.
    type Fields;

    /// A constant struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`]).
    // Actually FIELDS is an alias for F instead of the other way around.
    // This changes was made in the hope it would improve IDE support.
    const FIELDS: Self::Fields = Self::F;

    /// Shorthand version of [`FIELDS`]
    ///
    /// [`FIELDS`]: Model::FIELDS
    const F: Self::Fields;

    /// Returns the table name of the model
    fn table_name() -> &'static str;

    /// Returns the model's intermediate representation
    ///
    /// As library user you probably won't need this. You might want to look at [`write_models`].
    ///
    /// [`write_models`]: crate::write_models
    fn get_imr() -> imr::Model;
}

/// All relevant information about a model's field
#[derive(Copy, Clone)]
pub struct Field<T, D: hmr::db_type::DbType, A> {
    /// This field's position in the model.
    pub index: usize,

    /// Name of this field
    pub name: &'static str,

    /// List of annotations this field has set
    pub annotations: A,

    /// Optional definition of the location of field in the source code
    pub source: Option<Source>,

    #[doc(hidden)]
    pub _phantom: PhantomData<(T, D)>,
}

impl<T: AsDbType, D: hmr::db_type::DbType, A> Field<T, D, A> {
    /// Reexport [`AsDbType::from_primitive`]
    ///
    /// This method makes macros' syntax slightly cleaner
    #[inline(always)]
    pub fn convert_primitive(&self, primitive: T::Primitive) -> T {
        T::from_primitive(primitive)
    }
}

impl<
        T: AsDbType,
        D: hmr::db_type::DbType,
        A: hmr::annotations::AsImr<Imr = Vec<imr::Annotation>>,
    > From<&'_ Field<T, D, A>> for imr::Field
{
    fn from(field: &'_ Field<T, D, A>) -> Self {
        let mut annotations = field.annotations.as_imr();
        if !T::IS_NULLABLE {
            annotations.push(imr::Annotation::NotNull);
        }
        imr::Field {
            name: field.name.to_string(),
            db_type: D::IMR,
            annotations,
            source_defined_at: field.source.map(Into::into),
        }
    }
}

/// Location in the source code a [Model] or [Field] originates from
/// Used for better error messages in the migration tool
#[derive(Copy, Clone)]
pub struct Source {
    /// Filename of the source code of the [Model] or [Field]
    pub file: &'static str,
    /// Line of the [Model] or [Field]
    pub line: usize,
    /// Column of the [Model] or [Field]
    pub column: usize,
}

impl From<Source> for imr::Source {
    fn from(source: Source) -> Self {
        imr::Source {
            file: source.file.to_string(),
            line: source.line,
            column: source.column,
        }
    }
}
