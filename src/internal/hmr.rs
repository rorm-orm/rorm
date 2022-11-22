//! This module holds the high level model representation
//!
//! It adds:
//! - [`db_type`]: a type level version of [`imr::DbType`] to be used in generic type bound checks
//! - [`annotations`]: a type level version of [`imr::Annotation`] to be used in generic type bound checks
//!
//! These features are split into different submodules to avoid name conflicts.
//!
//! [`imr::DbType`]: crate::imr::DbType
//! [`imr::Annotation`]: crate::imr::Annotation

use crate::imr;

/// A type level version of [`imr::DbType`] to be used in generic type bound checks
///
/// [`imr::DbType`]: crate::imr::DbType
pub mod db_type {
    use crate::imr;

    /// Trait to associate the type-level db types with their runtime db types
    pub trait DbType: 'static {
        /// Equivalent runtime db type
        const IMR: imr::DbType;
    }

    macro_rules! impl_db_types {
        ($(#[doc = $doc:literal] $type:ident,)*) => {
            $(
                #[doc = $doc]
                pub struct $type;
                impl DbType for $type {
                    const IMR: imr::DbType = imr::DbType::$type;
                }
            )*
        };
    }

    impl_db_types!(
        /// Type level version of [`imr::DbType::VarChar`]
        VarChar,
        /// Type level version of [`imr::DbType::VarBinary`]
        VarBinary,
        /// Type level version of [`imr::DbType::Int8`]
        Int8,
        /// Type level version of [`imr::DbType::Int16`]
        Int16,
        /// Type level version of [`imr::DbType::Int32`]
        Int32,
        /// Type level version of [`imr::DbType::Int64`]
        Int64,
        /// Type level version of [`imr::DbType::Float`]
        Float,
        /// Type level version of [`imr::DbType::Double`]
        Double,
        /// Type level version of [`imr::DbType::Boolean`]
        Boolean,
        /// Type level version of [`imr::DbType::Date`]
        Date,
        /// Type level version of [`imr::DbType::DateTime`]
        DateTime,
        /// Type level version of [`imr::DbType::Timestamp`]
        Timestamp,
        /// Type level version of [`imr::DbType::Time`]
        Time,
        /// Type level version of [`imr::DbType::Choices`]
        Choices,
    );

    /// A type-level [Option], ether some [DbType] or none i.e. `()`
    pub trait OptionDbType {
        /// [Option::unwrap_or]
        ///
        /// `Self`, if it is "some" i.e. not `()` and `Default` otherwise
        type UnwrapOr<Default: DbType>: DbType;
    }
    impl<T: DbType> OptionDbType for T {
        type UnwrapOr<Default: DbType> = Self;
    }
    impl OptionDbType for () {
        type UnwrapOr<Default: DbType> = Default;
    }
}

/// A type level version of [`imr::Annotation`] to be used in generic type bound checks
///
/// [`imr::Annotation`]: crate::imr::Annotation
pub mod annotations {
    use crate::imr;

    macro_rules! impl_annotations {
        ($($(#[doc = $doc:literal])* $anno:ident $(($data:ty))?,)*) => {
            $(
                $(#[doc = $doc])*
                pub struct $anno$((
                    /// The annotation's data
                    pub $data
                ))?;

                impl AsImr for $anno {
                    type Imr = imr::Annotation;

                    fn as_imr(&self) -> imr::Annotation {
                        imr::Annotation::$anno$(({
                            let data: &$data = &self.0;
                            data.as_imr()
                        }))?
                    }
                }
            )*
        };
    }

    impl_annotations!(
        /// Will set the current time of the database when a row is created.
        AutoCreateTime,
        /// Will set the current time of the database when a row is updated.
        AutoUpdateTime,
        /// AUTO_INCREMENT constraint
        AutoIncrement,
        /// A list of choices to set
        Choices(&'static [&'static str]),
        /// DEFAULT constraint
        DefaultValue(DefaultValueData),
        /// Create an index. The optional [IndexData] can be used, to build more complex indexes.
        Index(Option<IndexData>),
        /// Only for VARCHAR. Specifies the maximum length of the column's content.
        MaxLength(i32),
        /// The annotated column will be used as primary key
        PrimaryKey,
        /// UNIQUE constraint
        Unique,
    );

    /// Action to take on a foreign key in case of on delete
    pub type OnDelete = imr::ReferentialAction;

    /// Action take on a foreign key in case of an update
    pub type OnUpdate = imr::ReferentialAction;

    /// Represents a complex index
    pub struct IndexData {
        /// Name of the index. Can be used multiple times in a model to create an
        /// index with multiple columns.
        pub name: &'static str,

        /// The order to put the columns in while generating an index.
        /// Only useful if multiple columns with the same name are present.
        pub priority: Option<i32>,
    }

    /// A column's default value which is any non object / array json value
    pub enum DefaultValueData {
        /// Use hexadecimal to represent binary data
        String(&'static str),
        /// i64 is used as it can represent any integer defined in DbType
        Integer(i64),
        /// Ordered float is used as f64 does not Eq and Order which are needed for Hash
        Float(f64),
        /// Just a bool. Nothing interesting here.
        Boolean(bool),
    }

    /// Trait for converting a hmr type into a imr one
    pub trait AsImr {
        /// Imr type to convert to
        type Imr;

        /// Convert to imr type
        fn as_imr(&self) -> Self::Imr;
    }

    /// [`Index`]'s data
    impl AsImr for Option<IndexData> {
        type Imr = Option<imr::IndexValue>;

        fn as_imr(&self) -> Self::Imr {
            self.as_ref().map(|data| imr::IndexValue {
                name: data.name.to_string(),
                priority: data.priority,
            })
        }
    }

    /// [`DefaultValue`]'s data
    impl AsImr for DefaultValueData {
        type Imr = imr::DefaultValue;

        fn as_imr(&self) -> Self::Imr {
            match self {
                DefaultValueData::String(string) => imr::DefaultValue::String(string.to_string()),
                DefaultValueData::Integer(integer) => imr::DefaultValue::Integer(*integer),
                DefaultValueData::Float(float) => imr::DefaultValue::Float((*float).into()),
                DefaultValueData::Boolean(boolean) => imr::DefaultValue::Boolean(*boolean),
            }
        }
    }

    /// [`MaxLength`]'s data
    impl AsImr for i32 {
        type Imr = i32;
        fn as_imr(&self) -> Self::Imr {
            *self
        }
    }

    /// [`Choices`]' data
    impl AsImr for &'static [&'static str] {
        type Imr = Vec<String>;
        fn as_imr(&self) -> Self::Imr {
            self.iter().map(ToString::to_string).collect()
        }
    }
}

/// Location in the source code a model or field originates from
/// Used for better error messages in the migration tool
#[derive(Copy, Clone)]
pub struct Source {
    /// Filename of the source code of the model or field
    pub file: &'static str,
    /// Line of the model or field
    pub line: usize,
    /// Column of the model or field
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
