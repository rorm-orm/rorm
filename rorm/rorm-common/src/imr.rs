//! The Internal Model Representation used by our migration cli tool
use serde::{Deserialize, Serialize};

/// A collection of all models used in the resulting application
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct InternalModelFormat {
    pub models: Vec<Model>,
}

/// A single model i.e. database table
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Model {
    pub name: String,

    pub fields: Vec<Field>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_defined_at: Option<Source>,
}

/// Model's fields i.e. the table's columns
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Field {
    pub name: String,

    #[serde(rename = "Type")]
    pub db_type: DbType,

    pub annotations: Vec<Annotation>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_defined_at: Option<Source>,
}

/// Location in the source code a Model or Field originates from
/// Used for better error messages in the migration tool
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// All column types supported by the migration tool
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    UInt64,
    #[serde(rename = "float_number")]
    Float,
    #[serde(rename = "double_number")]
    Double,
    Boolean,
    Date,
    Datetime,
    Timestamp,
    Time,
    Choices,
    Set,
}

/// The subset of annotations which need to be communicated with the migration tool
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "Type", content = "Value")]
#[serde(rename_all = "snake_case")]
pub enum Annotation {
    AutoCreateTime,
    AutoUpdateTime,
    Choices(Vec<String>),
    DefaultValue(DefaultValue),
    Index(Option<IndexValue>),
    MaxLength(i32),
    NotNull,
    PrimaryKey,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IndexValue {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// A column's default value which is any non object / array json value
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DefaultValue {
    /// Use hexadecimal to represent binary data
    String(String),
    /// i128 is used as it can represent any integer defined in DbType
    Integer(i128),
    Float(f64),
    Boolean(bool),
}
