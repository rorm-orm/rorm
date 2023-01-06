//! A type level version of [`imr::Annotation`](crate::imr::Annotation)
//! to be used in generic type bound checks and a struct to store them

use rorm_declaration::{imr, lints};

use super::AsImr;

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

/// Struct storing a [Field](crate::internal::field::Field)'s annotations.
pub struct Annotations {
    /// The `#[rorm(auto_create_time)]` annotation
    pub auto_create_time: Option<AutoCreateTime>,

    /// The `#[rorm(auto_update_time)]` annotation
    pub auto_update_time: Option<AutoUpdateTime>,

    /// The `#[rorm(auto_increment)]` annotation
    pub auto_increment: Option<AutoIncrement>,

    /// The `#[rorm(choices(..))]` annotation
    pub choices: Option<Choices>,

    /// The `#[rorm(default = ..)]` annotation
    pub default: Option<DefaultValue>,

    /// The `#[rorm(index(..))]` annotation
    pub index: Option<Index>,

    /// The `#[rorm(max_length = ..)]` annotation
    pub max_length: Option<MaxLength>,

    /// The `#[rorm(on_delete = ..)]` annotation
    pub on_delete: Option<OnDelete>,

    /// The `#[rorm(on_update = ..)]` annotation
    pub on_update: Option<OnUpdate>,

    /// The `#[rorm(primary_key)]` annotation
    pub primary_key: Option<PrimaryKey>,

    /// The `#[rorm(unique)]` annotation
    pub unique: Option<Unique>,

    /// Set implicitly if type is `Option<T>`
    pub nullable: bool,

    /// Set implicitly if type is `ForeignModel<M>`
    pub foreign: bool,
}

impl AsImr for Annotations {
    type Imr = Vec<imr::Annotation>;

    // Used for consistent syntax
    #[allow(clippy::redundant_pattern_matching)]
    fn as_imr(&self) -> Vec<imr::Annotation> {
        // Deconstruct to help ensure every annotation is handled.
        let Self {
            auto_create_time,
            auto_update_time,
            auto_increment,
            choices,
            default,
            index,
            max_length,
            foreign: _,   // Has to be set by field
            on_delete: _, //
            on_update: _, //
            primary_key,
            unique,
            nullable: _, // Set via not_null()
        } = self;
        let mut annotations = Vec::new();
        if let Some(_) = auto_create_time {
            annotations.push(imr::Annotation::AutoCreateTime);
        }
        if let Some(_) = auto_update_time {
            annotations.push(imr::Annotation::AutoCreateTime);
        }
        if let Some(_) = auto_increment {
            annotations.push(imr::Annotation::AutoIncrement);
        }
        if let Some(choices) = choices {
            annotations.push(choices.as_imr());
        }
        if let Some(default) = default {
            annotations.push(default.as_imr());
        }
        if let Some(index) = index {
            annotations.push(index.as_imr())
        }
        if let Some(max_length) = max_length {
            annotations.push(max_length.as_imr())
        }
        if let Some(_) = primary_key {
            annotations.push(imr::Annotation::PrimaryKey);
        }
        if let Some(_) = unique {
            annotations.push(imr::Annotation::Unique);
        }
        if self.not_null() {
            annotations.push(imr::Annotation::NotNull);
        }
        annotations
    }
}

impl Annotations {
    /// Construct an empty Annotations struct
    pub const fn empty() -> Self {
        Self {
            auto_create_time: None,
            auto_update_time: None,
            auto_increment: None,
            choices: None,
            default: None,
            index: None,
            max_length: None,
            on_delete: None,
            on_update: None,
            primary_key: None,
            unique: None,
            nullable: false,
            foreign: false,
        }
    }

    /// Is SQL's not null annotation set?
    pub const fn not_null(&self) -> bool {
        let implicit = self.primary_key.is_some();
        !self.nullable && !implicit
    }

    /// Convert to the representation used by the shared lints.
    pub const fn as_lint(&self) -> lints::Annotations {
        lints::Annotations {
            auto_create_time: self.auto_create_time.is_some(),
            auto_update_time: self.auto_update_time.is_some(),
            auto_increment: self.auto_increment.is_some(),
            choices: self.choices.is_some(),
            default: self.default.is_some(),
            index: self.index.is_some(),
            max_length: self.max_length.is_some(),
            not_null: self.not_null(),
            primary_key: self.primary_key.is_some(),
            unique: self.unique.is_some(),
            foreign_key: self.foreign,
        }
    }

    /// Merge with another annotations instance
    ///
    /// This method is used to merge a field's explicitly set annotations with its type's implicit ones.
    /// If a annotation is set on both structs, it name will be returned as error.
    pub const fn merge(mut self, other: Self) -> Result<Self, &'static str> {
        macro_rules! merge {
            ($self:expr, let Self {$($field:ident,)+} = $other:expr;) => {{
                let Self {
                    $($field,)+
                    nullable,
                    foreign,
                } = other;

                $(
                    if self.$field.is_none() {
                        self.$field = $field;
                    } else if $field.is_some() {
                        return Err(stringify!($field));
                    }
                )+

                if !self.nullable {
                    self.nullable = nullable;
                } else {
                    return Err("nullable");
                }

                if !self.foreign {
                    self.foreign = foreign;
                } else {
                    return Err("foreign");
                }
            }};
        }
        merge!(self, let Self {
            auto_create_time,
            auto_update_time,
            auto_increment,
            choices,
            default,
            index,
            max_length,
            on_delete,
            on_update,
            primary_key,
            unique,
        } = other;);
        Ok(self)
    }
}
