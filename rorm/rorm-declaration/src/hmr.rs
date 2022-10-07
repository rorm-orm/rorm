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
        /// Type level version of [`imr::DbType::UInt8`]
        UInt8,
        /// Type level version of [`imr::DbType::UInt16`]
        UInt16,
        /// Type level version of [`imr::DbType::UInt32`]
        UInt32,
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
        /// Type level version of [`imr::DbType::Set`]
        Set,
    );
}

/// A type level version of [`imr::Annotation`] to be used in generic type bound checks
///
/// [`imr::Annotation`]: crate::imr::Annotation
pub mod annotations {
    use crate::imr;
    use std::marker::PhantomData;

    /// Trait to store a concrete optional annotation as generic type parameter.
    pub trait Annotation<T: 'static + Copy>: 'static + Copy {
        /// Convert the annotation into its imr representation.
        ///
        /// [`NotSet`] and [`Forbidden`] return `None`.
        /// [`Implicit`] and any annotation itself return `Some`.
        fn as_imr(&self) -> Option<imr::Annotation>;
    }

    /// An annotation which has not been set
    #[derive(Copy, Clone)]
    pub struct NotSet<T>(PhantomData<T>);
    impl<T> NotSet<T> {
        /// Alternative to constructor which avoids importing [`PhantomData`]
        pub const fn new() -> Self {
            NotSet(PhantomData)
        }
    }
    impl<T: Annotation<T>> Annotation<T> for NotSet<T> {
        fn as_imr(&self) -> Option<imr::Annotation> {
            None
        }
    }

    /// An annotation which is implied by a field's datatype
    #[derive(Copy, Clone)]
    pub struct Implicit<T>(T);
    impl<T> Implicit<T> {
        /// Constructor to keep similar API to [`Forbidden`] and [`NotSet`]
        pub const fn new(anno: T) -> Self {
            Implicit(anno)
        }
    }
    impl<T: Annotation<T>> Annotation<T> for Implicit<T> {
        fn as_imr(&self) -> Option<imr::Annotation> {
            self.0.as_imr()
        }
    }

    /// An annotation which is forbidden to be set.
    #[derive(Copy, Clone)]
    pub struct Forbidden<T>(PhantomData<T>);
    impl<T> Forbidden<T> {
        /// Alternative to constructor which avoids importing [`PhantomData`]
        pub const fn new() -> Self {
            Forbidden(PhantomData)
        }
    }
    impl<T: Annotation<T>> Annotation<T> for Forbidden<T> {
        fn as_imr(&self) -> Option<imr::Annotation> {
            None
        }
    }

    macro_rules! impl_annotations {
        ($($(#[doc = $doc:literal])* $field:ident $anno:ident $(($data:ty))?,)*) => {
            $(
                $(#[doc = $doc])*
                #[derive(Copy, Clone)]
                pub struct $anno$((
                    /// The annotation's data
                    pub $data
                ))?;

                impl Annotation<$anno> for $anno {
                    fn as_imr(&self) -> Option<imr::Annotation> {
                        Some(imr::Annotation::$anno$(({
                            let data: &$data = &self.0;
                            data.as_imr()
                        }))?)
                    }
                }
            )*
        };
    }

    impl_annotations!(
        /// Will set the current time of the database when a row is created.
        auto_create_time AutoCreateTime,
        /// Will set the current time of the database when a row is updated.
        auto_update_time AutoUpdateTime,
        /// AUTO_INCREMENT constraint
        auto_increment AutoIncrement,
        /// A list of choices to set
        choices Choices(&'static [&'static str]),
        /// DEFAULT constraint
        default DefaultValue(DefaultValueData),
        /// Create an index. The optional [IndexData] can be used, to build more complex indexes.
        index Index(Option<IndexData>),
        /// Only for VARCHAR. Specifies the maximum length of the column's content.
        max_length MaxLength(i32),
        /// NOT NULL constraint
        not_null NotNull,
        /// The annotated column will be used as primary key
        primary_key PrimaryKey,
        /// UNIQUE constraint
        unique Unique,
    );

    /// This trait is used to "compute" [`Annotations<...>`]'s next concrete type after a step in the builder pattern.
    ///
    /// It would be reasonable to put the actual "step" method into this trait: `some_annos.add(SomeAnno)`
    /// Sadly rust's traits don't support const methods (yet?).
    /// So each "step" method needs its own name and exists completely detached to this trait: `some_annos.some_anno(SomeAnno)`
    pub trait Step<T> {
        /// The resulting type after this step
        type Output;
    }

    /// Represents a complex index
    #[derive(Copy, Clone)]
    pub struct IndexData {
        /// Name of the index. Can be used multiple times in a model to create an
        /// index with multiple columns.
        pub name: &'static str,

        /// The order to put the columns in while generating an index.
        /// Only useful if multiple columns with the same name are present.
        pub priority: Option<i32>,
    }

    /// A column's default value which is any non object / array json value
    #[derive(Copy, Clone)]
    pub enum DefaultValueData {
        /// Use hexadecimal to represent binary data
        String(&'static str),
        /// i128 is used as it can represent any integer defined in DbType
        Integer(i128),
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
