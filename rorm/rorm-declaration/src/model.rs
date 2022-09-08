use std::ops::{Deref, DerefMut};

use crate::imr;

/// This trait maps rust types to database types
pub trait AsDbType {
    /// Extend a Vec with migrator annotations which are implied by the type.
    fn implicit_annotations(_annotations: &mut Vec<imr::Annotation>) {}

    /// The database type as defined in the Intermediate Model Representation
    const DB_TYPE: imr::DbType;

    /// Whether this type supports null.
    const IS_NULLABLE: bool = false;
}

macro_rules! impl_as_db_type {
    ($type:ty, $variant:ident) => {
        impl AsDbType for $type {
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
    fn implicit_annotations(annotations: &mut Vec<imr::Annotation>) {
        annotations.push(imr::Annotation::Choices(E::as_choices()));
    }

    const DB_TYPE: imr::DbType = imr::DbType::Choices;
}

/// A ModelDefinition provides methods to do something similar to reflection on model structs.
///
/// This trait is only implemented on empty types and used as dyn objects i.e. it is a higher
/// level representation for a function table.
/// It is automatically implemented for you by the [`derive(Model)`] attribute.
///
/// [`derive(Model)`]: crate::Model
// sync and send is required in order to store it as a static
pub trait GetModelDefinition: Sync + Send {
    /// Build rorm's model representation.
    fn as_rorm(&self) -> ModelDefinition;

    /// Build the Intermediate Model Representation
    fn as_imr(&self) -> imr::Model {
        self.as_rorm().into()
    }
}

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`].
///
/// [`derive(Model)`]: crate::Model
pub trait Model<FIELDS> {
    /// Returns the table name of the model
    fn table_name() -> &'static str;

    /// Returns a struct with all the fields' definitions
    fn fields() -> &'static FIELDS;
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
    fn implicit_annotations(annotations: &mut Vec<imr::Annotation>) {
        I::implicit_annotations(annotations);
        annotations.push(imr::Annotation::PrimaryKey); // TODO check if already
        annotations.push(imr::Annotation::AutoIncrement);
    }

    const DB_TYPE: imr::DbType = I::DB_TYPE;
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

/// rorm's model representation holding all data about a specific model.
///
/// This is very similar to the [Intermediate Model Representation](imr::Model). But it contains
/// more information and uses a slightly different format.
/// (For example using `&'static str` instead of `String`)
///
/// # WIP
/// This representations doesn't do much currently, but it is planned to be used in resolving relations.
pub struct ModelDefinition {
    /// Name of the table
    pub name: &'static str,

    /// Fields the Model has attached
    pub fields: Vec<Field>,

    /// Optional location of the source of this model
    pub source: Option<Source>,
}

impl From<ModelDefinition> for imr::Model {
    fn from(model: ModelDefinition) -> Self {
        imr::Model {
            name: model.name.to_string(),
            fields: model.fields.into_iter().map(From::from).collect(),
            source_defined_at: model.source.map(Into::into),
        }
    }
}

/// [`ModelDefinitions`]'s fields.
///
/// This is similar to [`imr::Field`]. See [`ModelDefinition`] for the why.
#[derive(Copy, Clone)]
pub struct Field {
    /// Name of this field
    pub name: &'static str,

    /// [imr::DbType] of this field
    pub db_type: imr::DbType,

    /// List of annotations this field has set
    pub annotations: &'static [Annotation],

    /// Pointer to static [AsDbType::implicit_annotations] implementation
    pub implicit_annotations: &'static (dyn Fn(&mut Vec<imr::Annotation>) + Sync),

    /// Whether this field is nullable or not
    pub nullable: bool,

    /// Optional definition of the location of field in the source code
    pub source: Option<Source>,
}

impl From<Field> for imr::Field {
    fn from(field: Field) -> Self {
        let mut annotations: Vec<_> = field.annotations.iter().map(|&anno| anno.into()).collect();
        (field.implicit_annotations)(&mut annotations);
        if !field.nullable {
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
