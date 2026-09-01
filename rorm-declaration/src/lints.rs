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

    /// Does the field have an [Annotation::Index] *without* a name?
    ///
    /// A named index may span several columns, which makes it valid on columns
    /// where a single column index wouldn't be, most notably a primary key.
    /// Since this struct can't express which columns an index spans,
    /// named indexes are not tracked here at all.
    pub index: bool,

    /// Does the field have the [Annotation::MaxLength]?
    pub max_length: bool,

    /// Does the field have the [Annotation::NotNull]?
    pub not_null: bool,

    /// Does the field have the [Annotation::PrimaryKey]?
    pub primary_key: bool,

    /// Does the field have the [Annotation::Unique]?
    pub unique: bool,

    /// Does the field have the [Annotation::ForeignKey]?
    pub foreign_key: bool,
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
            A { index: true, primary_key: true, .. } => "An unnamed Index and PrimaryKey are mutually exclusive; name the index to span the column as part of a composite index",
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
                // A named index may span several columns and is therefore not tracked
                Annotation::Index(index) => result.index |= index.is_none(),
                Annotation::MaxLength(_) => result.max_length = true,
                Annotation::NotNull => result.not_null = true,
                Annotation::PrimaryKey => result.primary_key = true,
                Annotation::Unique => result.unique = true,
                Annotation::ForeignKey(_) => result.foreign_key = true,
            }
        }
        result
    }
}

#[cfg(test)]
mod test_index_on_primary_key {
    use crate::imr::{Annotation, IndexValue};
    use crate::lints::Annotations;

    fn check(annotations: &[Annotation]) -> Result<(), &'static str> {
        Annotations::from(annotations).check()
    }

    #[test]
    fn unnamed_index_on_primary_key_is_redundant() {
        assert!(check(&[Annotation::PrimaryKey, Annotation::Index(None)]).is_err());
    }

    #[test]
    fn named_index_on_primary_key_is_allowed() {
        // A primary key is a perfectly valid column of a composite index,
        // e.g. "(collection, uuid)" for filtering by a foreign key
        // and sorting by the primary key.
        assert!(check(&[
            Annotation::PrimaryKey,
            Annotation::Index(Some(IndexValue {
                name: "collection_uuid".to_string(),
                priority: Some(2),
            })),
        ])
        .is_ok());
    }

    #[test]
    fn an_unnamed_index_is_still_caught_next_to_a_named_one() {
        assert!(check(&[
            Annotation::PrimaryKey,
            Annotation::Index(Some(IndexValue {
                name: "collection_uuid".to_string(),
                priority: None,
            })),
            Annotation::Index(None),
        ])
        .is_err());
    }
}
