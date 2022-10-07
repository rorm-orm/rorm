//! This module holds the high level model representation
//!
//! It adds:
//! - [`db_type`]: a type level version of [`imr::DbType`] to be used in generic type bound checks
//! - [`annotation`]: a type level version of [`imr::Annotation`] to be used in generic type bound checks
//!
//! These features are split into different submodules to avoid name conflicts.

/// A type level version of [`imr::DbType`] to be used in generic type bound checks
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
pub mod annotations {
    use crate::imr;
    use std::marker::PhantomData;

    /// Trait to store a concrete optional annotation as generic type parameter.
    /// If you just need any annotation, use the [`Any`] type as `T`.
    pub trait Annotation<T: 'static + Copy>: 'static + Copy {
        /// Convert the annotation into its imr representation.
        ///
        /// `()` implements this trait and always returns `None`
        /// Any other implementation should return `Some`
        fn as_imr(&self) -> Option<imr::Annotation>;
    }

    // Unset annotation parameter
    impl<T: 'static + Copy> Annotation<T> for () {
        fn as_imr(&self) -> Option<imr::Annotation> {
            None
        }
    }

    /// An annotation which is implied by a field's datatype
    ///
    /// This type is not necessary, but helps orientation in the hugh [`Annotations`] struct.
    #[derive(Copy, Clone)]
    pub struct Implicit<T>(T);
    impl<A: Annotation<T>, T: 'static + Copy> Annotation<T> for Implicit<A> {
        fn as_imr(&self) -> Option<imr::Annotation> {
            self.0.as_imr()
        }
    }

    /// An annotation which is forbidden to be set on a [`Annotations`] struct.
    #[derive(Copy, Clone)]
    pub struct Forbidden<T>(PhantomData<T>);
    impl<T> Forbidden<T> {
        pub const fn new() -> Self {
            Forbidden(PhantomData)
        }
    }
    impl<A: Annotation<T>, T: 'static + Copy> Annotation<T> for Forbidden<A> {
        fn as_imr(&self) -> Option<imr::Annotation> {
            None
        }
    }

    macro_rules! impl_annotations {
        ($($(#[doc = $doc:literal])* $field:ident $anno:ident $(($data:ty))?,)* [$($generic:ident),*]) => {
            $(
                $(#[doc = $doc])*
                #[derive(Copy, Clone)]
                pub struct $anno$((pub $data))?;
                impl Annotation<$anno> for $anno {
                    fn as_imr(&self) -> Option<imr::Annotation> {
                        Some(imr::Annotation::$anno$(({
                            let data: &$data = &self.0;
                            data.as_imr()
                        }))?)
                    }
                }
            )*

            #[derive(Copy, Clone)]
            pub struct Annotations<
                $($generic: Annotation<$anno>),*
            >{
                $(pub $field: $generic,)*
            }

            impl Annotations<(), (), (), (), (), (), (), (), (), ()> {
                pub const fn new() -> Self {
                    Annotations {
                        $($field: (),)*
                    }
                }
            }

            impl<$($generic: Annotation<$anno>),*> AsImr for Annotations<$($generic),*> {
                type Imr = Vec<imr::Annotation>;

                fn as_imr(&self) -> Self::Imr {
                    let mut annotations = Vec::new();
                    $(
                        if let Some(anno) = self.$field.as_imr() {
                            annotations.push(anno);
                        }
                    )*
                    annotations
                }
            }
        };
    }

    impl_annotations!(
        /// Only for [DbType::Timestamp], [DbType::Datetime], [DbType::Time] and [DbType::Date].
        /// Will set the current time of the database when a row is created.
        auto_create_time AutoCreateTime,
        /// Only for [DbType::Timestamp], [DbType::Datetime], [DbType::Time] and [DbType::Date].
        /// Will set the current time of the database when a row is updated.
        auto_update_time AutoUpdateTime,
        /// AUTO_INCREMENT constraint
        auto_increment AutoIncrement,
        /// A list of choices to set
        choices Choices(&'static [&'static str]),
        /// DEFAULT constraint
        default DefaultValue(DefaultValueData),
        /// Create an index. The optional [IndexValue] can be used, to build more complex indexes.
        index Index(Option<IndexData>),
        /// Only for VARCHAR. Specifies the maximum length of the column's content.
        max_length MaxLength(i32),
        /// NOT NULL constraint
        not_null NotNull,
        /// The annotated column will be used as primary key
        primary_key PrimaryKey,
        /// UNIQUE constraint
        unique Unique,

        // Generic parameters inside `Annotations<...>`
        [A, B, C, D, E, F, G, H, I, J]
    );

    /// This trait is used to "compute" [`Annotations<...>`]'s next concrete type after a step in the builder pattern.
    ///
    /// It would be reasonable to put the actual "step" method into this trait: `some_annos.add(SomeAnno)`
    /// Sadly rust's traits don't support const methods (yet?).
    /// So each "step" method needs its own name and exists completely detached to this trait: `some_annos.some_anno(SomeAnno)`
    ///
    /// ```
    /// use rorm_declaration::hmr::annotations::{Annotations, Step, Unique};
    /// let annos: Annotations<(), (), (), (), (), (), (), (), (), ()> = Annotations::new();
    /// let new_annos: <Annotations<(), (), (), (), (), (), (), (), (), ()> as Step<Unique>>::Output = annos.unique(Unique);
    /// ```
    ///
    /// See [`Add`] for a shorthand
    pub trait Step<T> {
        type Output;
    }

    /// Shorthand for working with [`Step`]
    ///
    /// ```
    /// use rorm_declaration::hmr::annotations::{Annotations, Add, Unique};
    /// let annos: Annotations<(), (), (), (), (), (), (), (), (), ()> = Annotations::new();
    /// let new_annos: Add<Unique, Annotations<(), (), (), (), (), (), (), (), (), ()>> = annos.unique(Unique);
    /// ```
    pub type Add<X, Y> = <Y as Step<X>>::Output;

    /// Represents a complex index
    #[derive(Copy, Clone)]
    pub struct IndexData {
        /// Name of the index. Can be used multiple times in a [Model] to create an
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

    pub trait AsImr {
        type Imr;

        fn as_imr(&self) -> Self::Imr;
    }

    impl AsImr for Option<IndexData> {
        type Imr = Option<imr::IndexValue>;

        fn as_imr(&self) -> Self::Imr {
            self.as_ref().map(|data| imr::IndexValue {
                name: data.name.to_string(),
                priority: data.priority,
            })
        }
    }

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

    impl AsImr for i32 {
        type Imr = i32;
        fn as_imr(&self) -> Self::Imr {
            *self
        }
    }

    impl AsImr for &'static [&'static str] {
        type Imr = Vec<String>;
        fn as_imr(&self) -> Self::Imr {
            self.iter().map(ToString::to_string).collect()
        }
    }

    macro_rules! impl_builder_step {
        (
            $($generic_pre:ident  : $anno_pre:ident  @ $field_pre:ident ,)*
             ($generic:ident      : $anno:ident      @ $field:ident     ),
            $($generic_post:ident : $anno_post:ident @ $field_post:ident,)*
        ) => {
            // If annotation is not set yet, set it.
            impl<$($generic_pre: Annotation<$anno_pre>,)* $($generic_post: Annotation<$anno_post>,)*>
            Annotations<$($generic_pre,)*  (), $($generic_post,)*> {
                #[doc = concat!("Add a ", stringify!($anno), " annotation")]
                pub const fn $field(&self, $field: $anno) -> Annotations<$($generic_pre,)*  $anno, $($generic_post,)*> {
                    let Annotations {$($field_pre,)* $($field_post,)* ..} = *self;
                    Annotations {$($field_pre,)* $field, $($field_post,)*}
                }
            }

            impl<$($generic_pre: Annotation<$anno_pre>,)* $($generic_post: Annotation<$anno_post>,)*>
            Step<$anno> for Annotations<$($generic_pre,)*  (), $($generic_post,)*> {
                type Output = Annotations<$($generic_pre,)*  $anno, $($generic_post,)*>;
            }

            /*
            // If annotation has been set already, panic.
            impl<$($generic_pre: Annotation<$anno_pre>,)* $($generic_post: Annotation<$anno_post>,)*>
            Annotations<$($generic_pre,)*  $anno, $($generic_post,)*> {
                #[doc = concat!("Add a ", stringify!($anno), " annotation")]
                pub const fn $field(&self, $field: $anno) -> Annotations<$($generic_pre,)*  $anno, $($generic_post,)*> {
                    panic!(concat!("Can't set annotation \"", stringify!($field), "\". It has already been set."));
                }
            }

            // If annotation was set implicit, panic.
            impl<$($generic_pre: Annotation<$anno_pre>,)* $($generic_post: Annotation<$anno_post>,)*>
            Annotations<$($generic_pre,)*  Implicit<$anno>, $($generic_post,)*> {
                #[doc = concat!("Add a ", stringify!($anno), " annotation")]
                pub const fn $field(&self, $field: $anno) -> Annotations<$($generic_pre,)*  $anno, $($generic_post,)*> {
                    panic!(concat!("Can't set annotation \"", stringify!($field), "\". It has already been set implicitly."));
                }
            }

            // If annotation is forbidden, panic.
            impl<$($generic_pre: Annotation<$anno_pre>,)* $($generic_post: Annotation<$anno_post>,)*>
            Annotations<$($generic_pre,)*  Forbidden<$anno>, $($generic_post,)*> {
                #[doc = concat!("Add a ", stringify!($anno), " annotation")]
                pub const fn $field(&self, $field: $anno) -> Annotations<$($generic_pre,)*  $anno, $($generic_post,)*> {
                    panic!(concat!("Can't set annotation \"", stringify!($field), "\". It is not allowed on this field."));
                }
            }
            */
        };
    }
    mod builder {
        use super::*;
        impl_builder_step!(
            (A: AutoCreateTime @ auto_create_time),
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            (B: AutoUpdateTime @ auto_update_time),
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            (C: AutoIncrement @ auto_increment),
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            (D: Choices @ choices),
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            (E: DefaultValue @ default),
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            (F: Index @ index),
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            (G: MaxLength @ max_length),
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            (H: NotNull @ not_null),
            I: PrimaryKey @ primary_key,
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            (I: PrimaryKey @ primary_key),
            J: Unique @ unique,
        );
        impl_builder_step!(
            A: AutoCreateTime @ auto_create_time,
            B: AutoUpdateTime @ auto_update_time,
            C: AutoIncrement @ auto_increment,
            D: Choices @ choices,
            E: DefaultValue @ default,
            F: Index @ index,
            G: MaxLength @ max_length,
            H: NotNull @ not_null,
            I: PrimaryKey @ primary_key,
            (J: Unique @ unique),
        );
    }
}
