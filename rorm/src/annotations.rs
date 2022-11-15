//! This module implements a struct to store annotations

use rorm_declaration::hmr::annotations::{
    AsImr, AutoCreateTime, AutoIncrement, AutoUpdateTime, Choices, DefaultValue, Index, MaxLength,
    OnDelete, OnUpdate, PrimaryKey, Unique,
};
use rorm_declaration::{imr, lints};

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
}

impl AsImr for Annotations {
    type Imr = Vec<imr::Annotation>;

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
            on_delete: _, // Has to be set by field
            on_update: _, //
            primary_key,
            unique,
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
        }
    }

    /// Does any annotations imply not null such that setting it in sql would be an error?
    pub const fn implicit_not_null(&self) -> bool {
        self.primary_key.is_some()
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
            not_null: false, // Has to be set by field
            primary_key: self.primary_key.is_some(),
            unique: self.unique.is_some(),
            foreign_key: false, // Has to be set by field
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
                    $($field),+
                } = other;

                $(
                    if self.$field.is_none() {
                        self.$field = $field;
                    } else if $field.is_some() {
                        return Err(stringify!($field));
                    }
                )+
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
