/// The Internal Model Representation used by our migration cli tool
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct App {
    pub models: Vec<Model>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Model {
    pub name: String,

    pub fields: Vec<Field>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "source_defined_at")]
    pub source: Option<Source>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Field {
    pub name: String,

    #[serde(rename = "type")]
    pub db_type: DbType,

    pub annotations: Vec<Annotation>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "source_defined_at")]
    pub source: Option<Source>,
}

/// Location in the source code a Model or Field originates from.
/// Used for better error messages in the migration tool.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Source {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[non_exhaustive]
#[allow(dead_code)]
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

#[non_exhaustive]
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum Annotation {
    AutoCreateTime,
    AutoUpdateTime,
    Choices(Vec<String>),
    DefaultValue(DefaultValue),
    Index(Option<Index>),
    MaxLength(i32),
    NotNull,
    PrimaryKey,
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Index {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// A column's default value which is any non object / array json value
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DefaultValue {
    String(String),
    Float(f64),
    Integer(i32),
    Boolean(bool),
}
