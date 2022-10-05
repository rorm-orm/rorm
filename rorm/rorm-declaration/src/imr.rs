//! The Internal Model Representation used by our migration cli tool
use std::hash::{Hash, Hasher};

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

/// A collection of all models used in the resulting application
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct InternalModelFormat {
    /// List of all models
    pub models: Vec<Model>,
}

/// A single model i.e. database table
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Model {
    /// Name of the table
    pub name: String,

    /// List of columns of the table
    pub fields: Vec<Field>,

    /// Optional source reference to enhance error messages
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_defined_at: Option<Source>,
}

/// Model's fields i.e. the table's columns
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Field {
    /// Name of the column
    pub name: String,

    /// Type of the column
    #[serde(rename = "Type")]
    pub db_type: DbType,

    /// List of annotations, constraints, etc.
    pub annotations: Vec<Annotation>,

    /// Optional source reference to enhance error messages
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_defined_at: Option<Source>,
}

/// Location in the source code a [Model] or [Field] originates from
/// Used for better error messages in the migration tool
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    /// Filename of the source code of the [Model] or [Field]
    pub file: String,
    /// Line of the [Model] or [Field]
    pub line: usize,
    /// Column of the [Model] or [Field]
    pub column: usize,
}

/// All column types supported by the migration tool
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DbType {
    VarChar,
    VarBinary,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    // no UInt64 because it can't be safely represented on any db
    #[serde(rename = "float_number")]
    Float,
    #[serde(rename = "double_number")]
    Double,
    Boolean,
    Date,
    DateTime,
    Timestamp,
    Time,
    Choices,
    Set,
}

/// The subset of annotations which need to be communicated with the migration tool
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[serde(tag = "Type", content = "Value")]
#[serde(rename_all = "snake_case")]
pub enum Annotation {
    /// Only for [DbType::Timestamp], [DbType::Datetime], [DbType::Time] and [DbType::Date].
    /// Will set the current time of the database when a row is created.
    AutoCreateTime,
    /// Only for [DbType::Timestamp], [DbType::Datetime], [DbType::Time] and [DbType::Date].
    /// Will set the current time of the database when a row is updated.
    AutoUpdateTime,
    /// AUTO_INCREMENT constraint
    AutoIncrement,
    /// A list of choices to set
    Choices(Vec<String>),
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

impl PartialEq for Annotation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Annotation::AutoUpdateTime, Annotation::AutoCreateTime) => true,
            (Annotation::AutoIncrement, Annotation::AutoIncrement) => true,
            (Annotation::Choices(_), Annotation::Choices(_)) => true,
            (Annotation::DefaultValue(_), Annotation::DefaultValue(_)) => true,
            (Annotation::Index(_), Annotation::Index(_)) => true,
            (Annotation::MaxLength(_), Annotation::MaxLength(_)) => true,
            (Annotation::NotNull, Annotation::NotNull) => true,
            (Annotation::PrimaryKey, Annotation::PrimaryKey) => true,
            (Annotation::Unique, Annotation::Unique) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (Annotation::AutoUpdateTime, Annotation::AutoCreateTime) => false,
            (Annotation::AutoIncrement, Annotation::AutoIncrement) => false,
            (Annotation::Choices(_), Annotation::Choices(_)) => false,
            (Annotation::DefaultValue(_), Annotation::DefaultValue(_)) => false,
            (Annotation::Index(_), Annotation::Index(_)) => false,
            (Annotation::MaxLength(_), Annotation::MaxLength(_)) => false,
            (Annotation::NotNull, Annotation::NotNull) => false,
            (Annotation::PrimaryKey, Annotation::PrimaryKey) => false,
            (Annotation::Unique, Annotation::Unique) => false,
            _ => true,
        }
    }
}

impl Hash for Annotation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Annotation::AutoCreateTime => state.write_i8(0),
            Annotation::AutoUpdateTime => state.write_i8(1),
            Annotation::AutoIncrement => state.write_i8(2),
            Annotation::Choices(_) => state.write_i8(3),
            Annotation::DefaultValue(_) => state.write_i8(4),
            Annotation::Index(_) => state.write_i8(5),
            Annotation::MaxLength(_) => state.write_i8(6),
            Annotation::NotNull => state.write_i8(7),
            Annotation::PrimaryKey => state.write_i8(8),
            Annotation::Unique => state.write_i8(9),
        }
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for x in data {
            x.hash(state);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::imr::{Annotation, IndexValue};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_annotation_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        Annotation::MaxLength(1).hash(&mut hasher1);
        Annotation::MaxLength(12313).hash(&mut hasher2);

        Annotation::Index(None).hash(&mut hasher1);
        Annotation::Index(Some(IndexValue {
            priority: None,
            name: "foo".to_string(),
        }))
        .hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_annotation_partial_eq() {
        assert_eq!(Annotation::MaxLength(1), Annotation::MaxLength(2));
        assert_eq!(
            Annotation::Index(None),
            Annotation::Index(Some(IndexValue {
                priority: None,
                name: "foo".to_string()
            }))
        );
    }
}

/// Represents a complex index
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct IndexValue {
    /// Name of the index. Can be used multiple times in a [Model] to create an
    /// index with multiple columns.
    pub name: String,

    /// The order to put the columns in while generating an index.
    /// Only useful if multiple columns with the same name are present.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// A column's default value which is any non object / array json value
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(untagged)]
pub enum DefaultValue {
    /// Use hexadecimal to represent binary data
    String(String),
    /// i128 is used as it can represent any integer defined in DbType
    Integer(i128),
    /// Ordered float is used as f64 does not Eq and Order which are needed for Hash
    Float(OrderedFloat<f64>),
    /// Just a bool. Nothing interesting here.
    Boolean(bool),
}
