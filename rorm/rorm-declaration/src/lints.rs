//! Some common lints whose code can be shared between rorm-macro and rorm-cli.

use crate::imr::Annotation;

/// Simple struct storing whether a specific annotation is set on a given field or not.
#[derive(Copy, Clone, Default, Debug)]
pub struct Annotations {
    /// Does the field have the [Annotation::AutoCreateTime]?
    pub auto_create_time: bool,

    /// Does the field have the [Annotation::AutoUpdateTime]?
    pub auto_update_time: bool,

    /// Does the field have the [Annotation::AutoIncrement]?
    pub auto_increment: bool,

    /// Does the field have the [Annotation::Choices]?
    pub choices: bool,

    /// Does the field have the [Annotation::DefaultValue]?
    pub default: bool,

    /// Does the field have the [Annotation::Index]?
    pub index: bool,

    /// Does the field have the [Annotation::MaxLength]?
    pub max_length: bool,

    /// Does the field have the [Annotation::NotNull]?
    pub not_null: bool,

    /// Does the field have the [Annotation::PrimaryKey]?
    pub primary_key: bool,

    /// Does the field have the [Annotation::Unique]?
    pub unique: bool,
}

impl Annotations {
    /// Check whether this set of annotations is valid.
    ///
    /// Returns a non-empty error message, when it is not.
    // Disable auto-format to make the following match compacter and more readable.
    #[rustfmt::skip]
    pub const fn check(self) -> Result<(), &'static str> {
        // Alias to reduce line length and noise
        use Annotations as A;

        let msg = match self {
            A { auto_create_time: true, auto_increment: true, .. } => "AutoCreateTime and AutoIncrement are mutually exclusive",
            A { auto_create_time: true, choices: true, .. } => "AutoCreateTime and Choices are mutually exclusive",
            A { auto_create_time: true, default: true, .. } => "AutoCreateTime and DefaultValue are mutually exclusive",
            A { auto_create_time: true, max_length: true, .. } => "AutoCreateTime and MaxLength are mutually exclusive",
            A { auto_create_time: true, primary_key: true, .. } => "AutoCreateTime and PrimaryKey are mutually exclusive",
            A { auto_create_time: true, unique: true, .. } => "AutoCreateTime and Unique are mutually exclusive",
            A { auto_update_time: true, auto_increment: true, .. } => "AutoUpdateTime and AutoIncrement are mutually exclusive",
            A { auto_update_time: true, choices: true, .. } => "AutoUpdateTime and Choices are mutually exclusive",
            A { auto_update_time: true, max_length: true, .. } => "AutoUpdateTime and MaxLength are mutually exclusive",
            A { auto_update_time: true, primary_key: true, .. } => "AutoUpdateTime and PrimaryKey are mutually exclusive",
            A { auto_update_time: true, unique: true, .. } => "AutoUpdateTime and Unique are mutually exclusive",
            A { auto_increment: true, choices: true, .. } => "AutoIncrement and Choices are mutually exclusive",
            A { auto_increment: true, max_length: true, .. } => "AutoIncrement and MaxLength are mutually exclusive",
            A { choices: true, max_length: true, .. } => "Choices and MaxLength are mutually exclusive",
            A { choices: true, primary_key: true, .. } => "Choices and PrimaryKey are mutually exclusive",
            A { choices: true, unique: true, .. } => "Choices and Unique are mutually exclusive",
            A { default: true, auto_update_time: true, .. } => "DefaultValue and AutoUpdateTime are mutually exclusive",
            A { default: true, auto_increment: true, .. } => "DefaultValue and AutoIncrement are mutually exclusive",
            A { default: true, primary_key: true, .. } => "DefaultValue and PrimaryKey are mutually exclusive",
            A { default: true, unique: true, .. } => "DefaultValue and Unique are mutually exclusive",
            A { index: true, primary_key: true, .. } => "Index and PrimaryKey are mutually exclusive",
            A { not_null: true, primary_key: true, .. } => "NotNull and PrimaryKey are mutually exclusive",

            A { auto_increment: true, primary_key: false, .. } => "AutoIncrement requires PrimaryKey",

            A { auto_update_time: true, not_null: true, auto_create_time: false, default: false, ..} => "AutoUpdateTime in combination with NotNull requires ether DefaultValue or AutoCreateTime",

            _ => "",
        };

        // Create Result based on error message length to avoid using Err() in the match expression.
        if !msg.is_empty() {
            Err(msg)
        } else {
            Ok(())
        }
    }
}

impl From<&[Annotation]> for Annotations {
    fn from(annotations: &[Annotation]) -> Self {
        let mut result = Annotations::default();
        for annotation in annotations {
            match annotation {
                Annotation::AutoCreateTime => result.auto_create_time = true,
                Annotation::AutoUpdateTime => result.auto_update_time = true,
                Annotation::AutoIncrement => result.auto_increment = true,
                Annotation::Choices(_) => result.choices = true,
                Annotation::DefaultValue(_) => result.default = true,
                Annotation::Index(_) => result.index = true,
                Annotation::MaxLength(_) => result.max_length = true,
                Annotation::NotNull => result.not_null = true,
                Annotation::PrimaryKey => result.primary_key = true,
                Annotation::Unique => result.unique = true,
            }
        }
        result
    }
}
