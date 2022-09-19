use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::imr;

/// This trait maps rust types to database types
pub trait AsDbType {
    /// A type understood which can be retrieved from the db and then converted into Self.
    type PRIMITIVE;
    /// Convert the associated primitive type into `Self`.
    ///
    /// This function allows "non-primitive" types like [GenericId] or any [DbEnum] to implement
    /// their decoding without access to the underlying db details (namely `sqlx::Decode`)
    fn from_primitive(primitive: Self::PRIMITIVE) -> Self;

    /// The database type as defined in the Intermediate Model Representation
    const DB_TYPE: imr::DbType;

    /// Whether this type supports null.
    const IS_NULLABLE: bool = false;

    /// Extend a Vec with migrator annotations which are implied by the type.
    fn implicit_annotations(_annotations: &mut Vec<imr::Annotation>) {}
}

macro_rules! impl_as_db_type {
    ($type:ty, $variant:ident) => {
        impl AsDbType for $type {
            type PRIMITIVE = Self;
            #[inline(always)]
            fn from_primitive(primitive: Self::PRIMITIVE) -> Self {
                primitive
            }

            const DB_TYPE: imr::DbType = imr::DbType::$variant;
        }
    };
}
impl_as_db_type!(Vec<u8>, VarBinary);
impl_as_db_type!(i8, Int8);
impl_as_db_type!(i16, Int16);
impl_as_db_type!(i32, Int32);
impl_as_db_type!(i64, Int64);
impl_as_db_type!(isize, Int64);
impl_as_db_type!(u8, UInt8);
impl_as_db_type!(u16, UInt16);
impl_as_db_type!(u32, UInt32);
impl_as_db_type!(u64, UInt64);
impl_as_db_type!(usize, UInt64);
impl_as_db_type!(f32, Float);
impl_as_db_type!(f64, Double);
impl_as_db_type!(bool, Boolean);
impl_as_db_type!(String, VarChar);
impl<T: AsDbType> AsDbType for Option<T> {
    type PRIMITIVE = Self;
    #[inline(always)]
    fn from_primitive(primitive: Self::PRIMITIVE) -> Self {
        primitive
    }

    fn implicit_annotations(annotations: &mut Vec<imr::Annotation>) {
        T::implicit_annotations(annotations)
    }

    const DB_TYPE: imr::DbType = T::DB_TYPE;

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

    /// Construct a vector containing all variants as strings.
    ///
    /// This will be called in order to construct the Intermediate Model Representation.
    fn as_choices() -> Vec<String>;
}
impl<E: DbEnum> AsDbType for E {
    type PRIMITIVE = String;
    fn from_primitive(primitive: Self::PRIMITIVE) -> Self {
        E::from_str(&primitive)
    }

    fn implicit_annotations(annotations: &mut Vec<imr::Annotation>) {
        annotations.push(imr::Annotation::Choices(E::as_choices()));
    }

    const DB_TYPE: imr::DbType = imr::DbType::Choices;
}

/// Trait implemented on Patches i.e. a subset of a model's fields.
///
/// Implemented by [`derive(Patch)`] as well as [`derive(Model)`].
pub trait Patch {
    /// The model this patch is for
    type MODEL;

    /// List of columns i.e. fields this patch contains
    const COLUMNS: &'static [&'static str];
}

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`].
///
/// [`derive(Model)`]: crate::Model
pub trait Model {
    /// [`FIELDS`]'s datatype
    ///
    /// [`FIELDS`]: Model::FIELDS
    type Fields;

    /// A struct holding the model's fields data
    const FIELDS: Self::Fields;

    /// Shorthand version of [`FIELDS`]
    ///
    /// [`FIELDS`]: Model::FIELDS
    const F: Self::Fields = Self::FIELDS;

    /// Returns the table name of the model
    fn table_name() -> &'static str;

    /// Returns the model's intermediate representation
    ///
    /// As library user you probably won't need this. You might want to look at [`write_models`].
    ///
    /// [`write_models`]: crate::write_models
    fn get_imr() -> imr::Model;
}

/// The type to add to most models as primary key:
/// ```ignore
/// use rorm::{Model, ID};
///
/// #[derive(Model)]
/// struct SomeModel {
///     id: ID,
///     ..
/// }
pub type ID = GenericId<i64>;

/// Generic Wrapper which implies the primary key and autoincrement annotation
#[derive(Copy, Clone, Debug)]
pub struct GenericId<I: AsDbType>(pub I);

impl<I: AsDbType> AsDbType for GenericId<I> {
    type PRIMITIVE = I;
    fn from_primitive(primitive: Self::PRIMITIVE) -> Self {
        GenericId(primitive)
    }

    const DB_TYPE: imr::DbType = I::DB_TYPE;

    fn implicit_annotations(annotations: &mut Vec<imr::Annotation>) {
        I::implicit_annotations(annotations);
        annotations.push(imr::Annotation::PrimaryKey); // TODO check if already
        annotations.push(imr::Annotation::AutoIncrement);
    }
}

impl<I: AsDbType> From<I> for GenericId<I> {
    fn from(id: I) -> Self {
        GenericId(id)
    }
}

impl<I: AsDbType> Deref for GenericId<I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<I: AsDbType> DerefMut for GenericId<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// [`ModelDefinitions`]'s fields.
///
/// This is similar to [`imr::Field`]. See [`ModelDefinition`] for the why.
#[derive(Copy, Clone)]
pub struct Field<T: 'static> {
    /// Name of this field
    pub name: &'static str,

    /// [imr::DbType] of this field
    pub db_type: imr::DbType,

    /// List of annotations this field has set
    pub annotations: &'static [Annotation],

    /// Optional definition of the location of field in the source code
    pub source: Option<Source>,

    #[doc(hidden)]
    pub _phantom: PhantomData<&'static T>,
}

impl<T: AsDbType> From<&'_ Field<T>> for imr::Field {
    fn from(field: &'_ Field<T>) -> Self {
        let mut annotations: Vec<_> = field.annotations.iter().map(|&anno| anno.into()).collect();
        T::implicit_annotations(&mut annotations);
        if !T::IS_NULLABLE {
            annotations.push(imr::Annotation::NotNull);
        }
        imr::Field {
            name: field.name.to_string(),
            db_type: field.db_type,
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

/// The subset of annotations which need to be communicated with the migration tool
#[derive(Copy, Clone)]
pub enum Annotation {
    /// Only for [DbType::Timestamp], [DbType::Datetime], [DbType::Time], [DbType::Date] and
    /// [DbType::Uint64]. Will set the current time of the database when a row is created.
    AutoCreateTime,
    /// Only for [DbType::Timestamp], [DbType::Datetime], [DbType::Time], [DbType::Date] and
    /// [DbType::Uint64]. Will set the current time of the database when a row is updated.
    AutoUpdateTime,
    /// AUTO_INCREMENT constraint
    AutoIncrement,
    /// A list of choices to set
    Choices(&'static [&'static str]),
    /// DEFAULT constraint
    DefaultValue(DefaultValue),
    /// Create an index. The optional [IndexValue] can be used, to build more complex indexes.
    Index(Option<IndexValue>),
    /// Only for VARCHAR. Specifies the maximum length of the column's content.
    MaxLength(i32),
    /// NOT NULL constraint
    NotNull,
    /// The annotated column will be used as primary key
    PrimaryKey,
    /// UNIQUE constraint
    Unique,
}

impl From<Annotation> for imr::Annotation {
    fn from(anno: Annotation) -> Self {
        match anno {
            Annotation::AutoCreateTime => imr::Annotation::AutoCreateTime,
            Annotation::AutoUpdateTime => imr::Annotation::AutoUpdateTime,
            Annotation::AutoIncrement => imr::Annotation::AutoIncrement,
            Annotation::Choices(choices) => {
                imr::Annotation::Choices(choices.into_iter().map(ToString::to_string).collect())
            }
            Annotation::DefaultValue(value) => imr::Annotation::DefaultValue(match value {
                DefaultValue::String(string) => imr::DefaultValue::String(string.to_string()),
                DefaultValue::Boolean(boolean) => imr::DefaultValue::Boolean(boolean),
                DefaultValue::Float(float) => imr::DefaultValue::Float(float.into()),
                DefaultValue::Integer(integer) => imr::DefaultValue::Integer(integer),
            }),
            Annotation::Index(index) => {
                imr::Annotation::Index(index.map(|index| imr::IndexValue {
                    name: index.name.to_string(),
                    priority: index.priority,
                }))
            }
            Annotation::MaxLength(length) => imr::Annotation::MaxLength(length),
            Annotation::NotNull => imr::Annotation::NotNull,
            Annotation::PrimaryKey => imr::Annotation::PrimaryKey,
            Annotation::Unique => imr::Annotation::Unique,
        }
    }
}

/// Represents a complex index
#[derive(Copy, Clone)]
pub struct IndexValue {
    /// Name of the index. Can be used multiple times in a [Model] to create an
    /// index with multiple columns.
    pub name: &'static str,

    /// The order to put the columns in while generating an index.
    /// Only useful if multiple columns with the same name are present.
    pub priority: Option<i32>,
}

/// A column's default value which is any non object / array json value
#[derive(Copy, Clone)]
pub enum DefaultValue {
    /// Use hexadecimal to represent binary data
    String(&'static str),
    /// i128 is used as it can represent any integer defined in DbType
    Integer(i128),
    /// Ordered float is used as f64 does not Eq and Order which are needed for Hash
    Float(f64),
    /// Just a bool. Nothing interesting here.
    Boolean(bool),
}
