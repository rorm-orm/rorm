//! The Internal Model Representation used by our migration cli tool
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde::Serialize;

/// A collection of all models used in the resulting application
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct InternalModelFormat {
    /// List of all models
    pub models: Vec<Model>,
}

/// A single model i.e. database table
#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.fields == other.fields
    }
}

impl Hash for Model {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fields.hash(state);
        self.name.hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        data.iter().for_each(|x| x.hash(state));
    }
}

/// Model's fields i.e. the table's columns
#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.db_type == other.db_type
            && self.annotations == other.annotations
    }
}

impl Hash for Field {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.annotations.hash(state);
        self.db_type.hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        data.iter().for_each(|x| x.hash(state));
    }
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
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DbType {
    #[deprecated(note = "Use Text instead")]
    VarChar,
    Binary,
    Int8,
    Int16,
    Int32,
    Int64,
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
    Uuid,
    MacAddress,
    IpNetwork,
    BitVec,
    Text,
}

/// The subset of annotations which need to be communicated with the migration tool
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "Type", content = "Value")]
#[serde(rename_all = "snake_case")]
pub enum Annotation {
    /// Only for [DbType::Timestamp], [DbType::DateTime], [DbType::Time] and [DbType::Date].
    /// Will set the current time of the database when a row is created.
    AutoCreateTime,
    /// Only for [DbType::Timestamp], [DbType::DateTime], [DbType::Time] and [DbType::Date].
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
    /// Specifies the maximum length of the column's content.
    ///
    /// It is part of the column's type for a [`DbType::VarChar`]
    /// and a check constraint for a [`DbType::Text`].
    /// Everywhere else, and in sqlite entirely, it is ignored -
    /// sqlite has no `varchar` and never enforces a string's length.
    MaxLength(i32),
    /// NOT NULL constraint
    NotNull,
    /// The annotated column will be used as primary key
    PrimaryKey,
    /// UNIQUE constraint
    Unique,
    /// Foreign Key constraint
    ForeignKey(ForeignKey),
}

/// Represents a foreign key
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ForeignKey {
    /// Name of the table that should be referenced
    pub table_name: String,
    /// Name of the column that should be referenced
    pub column_name: String,
    /// Action to be used in case of on delete
    pub on_delete: ReferentialAction,
    /// Action to be used in case of an update
    pub on_update: ReferentialAction,
}

/**
Action that gets trigger on update and on delete.
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ReferentialAction {
    /// Stop operation if any keys still depend on the parent table
    #[default]
    Restrict,
    /// The action is cascaded
    Cascade,
    /// The field is set to null
    SetNull,
    /// The field is set to its default
    SetDefault,
}

impl Display for ReferentialAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferentialAction::Restrict => write!(f, "RESTRICT"),
            ReferentialAction::Cascade => write!(f, "CASCADE"),
            ReferentialAction::SetNull => write!(f, "SET NULL"),
            ReferentialAction::SetDefault => write!(f, "SET DEFAULT"),
        }
    }
}

/// Represents a complex index
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

    /// Whether the index enforces its columns' values to be unique
    ///
    /// A unique index and a regular one may share a name.
    /// They are still two separate indexes.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub unique: bool,
}

impl Hash for IndexValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.priority.hash(state);
        // The models' hash is stored in the migration files
        // and has to stay the same for models without a unique index.
        if self.unique {
            self.unique.hash(state);
        }
    }
}

/// Helper for `#[serde(skip_serializing_if)]`
fn is_false(value: &bool) -> bool {
    !*value
}

/// An index of a [Model] i.e. the table's index over one or more of its columns
///
/// Indexes are not declared on a [Model] directly.
/// They are spread over its [Field]s using [Annotation::Index]
/// and gathered by [Model::indexes].
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Index {
    /// The name the index was declared under i.e. [IndexValue::name]
    ///
    /// It is `None` for an index which was declared without an [IndexValue].
    /// Such an index always spans exactly one column.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The columns the index spans, in the order they should be indexed in
    pub columns: Vec<String>,

    /// Whether the index enforces its columns' values to be unique
    /// i.e. [IndexValue::unique]
    ///
    /// Only an index with a [name](Index::name) can be unique.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub unique: bool,
}

impl Index {
    /// The identifier to create this index under in the database
    ///
    /// Unlike columns, indexes don't live in their table's namespace.
    /// Their names have to be unique across the whole database,
    /// which is why they are prefixed with their `table`'s name.
    ///
    /// A unique index deliberately doesn't end in `_key`, which is what postgres
    /// itself generates for an unnamed unique column (i.e. [Annotation::Unique]),
    /// so that it can never collide with a constraint rorm didn't name.
    pub fn sql_name(&self, table: &str) -> String {
        let suffix = if self.unique { "unique" } else { "idx" };
        match &self.name {
            Some(name) => format!("{table}_{name}_{suffix}"),
            // An index without a name always spans exactly one column
            None => format!("{table}_{}_{suffix}", self.columns.join("_")),
        }
    }
}

impl Model {
    /// Gathers the [Index]es declared by this model's [Field]s
    ///
    /// Fields sharing an [IndexValue::name] and [IndexValue::unique] are combined
    /// into a single index spanning all of them. Their order inside the index
    /// is their order of declaration, which can be overwritten using [IndexValue::priority].
    pub fn indexes(&self) -> Vec<Index> {
        /// A column of an index paired with the priority to sort it by
        struct Column<'a> {
            name: &'a str,
            priority: i32,
        }

        // Both vectors are kept in sync and ordered by the indexes' first occurrence,
        // to produce the same output for the same model every time.
        let mut keys: Vec<Option<(&str, bool)>> = Vec::new();
        let mut columns: Vec<Vec<Column>> = Vec::new();

        for field in &self.fields {
            for annotation in &field.annotations {
                let Annotation::Index(value) = annotation else {
                    continue;
                };

                let key = value
                    .as_ref()
                    .map(|value| (value.name.as_str(), value.unique));
                let priority = value.as_ref().and_then(|value| value.priority).unwrap_or(0);

                // Fields sharing a name contribute to the same index
                let index = match key.and_then(|key| keys.iter().position(|x| *x == Some(key))) {
                    Some(index) => index,
                    None => {
                        keys.push(key);
                        columns.push(Vec::new());
                        keys.len() - 1
                    }
                };

                columns[index].push(Column {
                    name: &field.name,
                    priority,
                });
            }
        }

        keys.into_iter()
            .zip(columns)
            .map(|(key, mut columns)| {
                // `sort_by_key` is stable, so columns of equal priority
                // keep their order of declaration
                columns.sort_by_key(|column| column.priority);
                Index {
                    name: key.map(|(name, _)| name.to_string()),
                    columns: columns
                        .into_iter()
                        .map(|column| column.name.to_string())
                        .collect(),
                    unique: key.is_some_and(|(_, unique)| unique),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod test_indexes {
    use crate::imr::{Annotation, DbType, Field, Index, IndexValue, Model};

    /// Builds a model whose fields are named after and annotated with `indexes`
    fn model(indexes: Vec<(&str, Option<IndexValue>)>) -> Model {
        Model {
            name: "user".to_string(),
            fields: indexes
                .into_iter()
                .map(|(name, index)| Field {
                    name: name.to_string(),
                    db_type: DbType::VarChar,
                    annotations: vec![Annotation::Index(index)],
                    source_defined_at: None,
                })
                .collect(),
            source_defined_at: None,
        }
    }

    fn named(name: &str, priority: Option<i32>) -> Option<IndexValue> {
        Some(IndexValue {
            name: name.to_string(),
            priority,
            unique: false,
        })
    }

    fn unique(name: &str, priority: Option<i32>) -> Option<IndexValue> {
        Some(IndexValue {
            name: name.to_string(),
            priority,
            unique: true,
        })
    }

    #[test]
    fn every_unnamed_index_spans_a_single_column() {
        assert_eq!(
            model(vec![("a", None), ("b", None)]).indexes(),
            vec![
                Index {
                    name: None,
                    columns: vec!["a".to_string()],
                    unique: false,
                },
                Index {
                    name: None,
                    columns: vec!["b".to_string()],
                    unique: false,
                },
            ]
        );
    }

    #[test]
    fn fields_sharing_a_name_are_combined() {
        assert_eq!(
            model(vec![
                ("a", named("ab", None)),
                ("c", None),
                ("b", named("ab", None)),
            ])
            .indexes(),
            vec![
                Index {
                    name: Some("ab".to_string()),
                    columns: vec!["a".to_string(), "b".to_string()],
                    unique: false,
                },
                Index {
                    name: None,
                    columns: vec!["c".to_string()],
                    unique: false,
                },
            ]
        );
    }

    #[test]
    fn priority_overwrites_the_order_of_declaration() {
        assert_eq!(
            model(vec![
                ("a", named("ab", Some(2))),
                ("b", named("ab", Some(1))),
            ])
            .indexes(),
            vec![Index {
                name: Some("ab".to_string()),
                columns: vec!["b".to_string(), "a".to_string()],
                unique: false,
            }]
        );
    }

    #[test]
    fn columns_of_equal_priority_keep_their_order() {
        assert_eq!(
            model(vec![
                ("a", named("abc", Some(1))),
                ("b", named("abc", Some(1))),
                ("c", named("abc", Some(0))),
            ])
            .indexes(),
            vec![Index {
                name: Some("abc".to_string()),
                columns: vec!["c".to_string(), "a".to_string(), "b".to_string()],
                unique: false,
            }]
        );
    }

    #[test]
    fn fields_sharing_a_name_and_uniqueness_are_combined() {
        assert_eq!(
            model(vec![
                ("a", unique("ab", Some(1))),
                ("b", unique("ab", Some(0))),
            ])
            .indexes(),
            vec![Index {
                name: Some("ab".to_string()),
                columns: vec!["b".to_string(), "a".to_string()],
                unique: true,
            }]
        );
    }

    #[test]
    fn unique_and_regular_indexes_sharing_a_name_are_separate() {
        assert_eq!(
            model(vec![
                ("a", named("ab", None)),
                ("b", unique("ab", None)),
                ("c", named("ab", None)),
            ])
            .indexes(),
            vec![
                Index {
                    name: Some("ab".to_string()),
                    columns: vec!["a".to_string(), "c".to_string()],
                    unique: false,
                },
                Index {
                    name: Some("ab".to_string()),
                    columns: vec!["b".to_string()],
                    unique: true,
                },
            ]
        );
    }

    #[test]
    fn a_model_without_index_annotations_has_no_indexes() {
        assert_eq!(
            Model {
                name: "user".to_string(),
                fields: vec![Field {
                    name: "id".to_string(),
                    db_type: DbType::Int64,
                    annotations: vec![Annotation::PrimaryKey],
                    source_defined_at: None,
                }],
                source_defined_at: None,
            }
            .indexes(),
            vec![]
        );
    }

    #[test]
    fn sql_names_are_prefixed_with_their_table() {
        let unnamed = Index {
            name: None,
            columns: vec!["login".to_string()],
            unique: false,
        };
        assert_eq!(unnamed.sql_name("user"), "user_login_idx");

        let named = Index {
            name: Some("full_name".to_string()),
            columns: vec!["last_name".to_string(), "first_name".to_string()],
            unique: false,
        };
        assert_eq!(named.sql_name("user"), "user_full_name_idx");
    }

    #[test]
    fn sql_names_of_unique_indexes_differ_from_regular_ones() {
        let unique = Index {
            name: Some("full_name".to_string()),
            columns: vec!["last_name".to_string(), "first_name".to_string()],
            unique: true,
        };
        assert_eq!(unique.sql_name("user"), "user_full_name_unique");
    }

    #[test]
    fn hash_of_non_unique_index_value_is_unchanged() {
        use std::hash::{DefaultHasher, Hash, Hasher};

        /// [`IndexValue`] before it gained [`IndexValue::unique`]
        #[derive(Hash)]
        struct OldIndexValue {
            name: String,
            priority: Option<i32>,
        }

        fn hash(value: impl Hash) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }

        let old = hash(OldIndexValue {
            name: "full_name".to_string(),
            priority: Some(1),
        });
        assert_eq!(hash(named("full_name", Some(1)).unwrap()), old);
        assert_ne!(hash(unique("full_name", Some(1)).unwrap()), old);
    }
}

/// A column's default value which is any non object / array json value
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum DefaultValue {
    /// Use hexadecimal to represent binary data
    String(String),
    /// i64 is used as it can represent any integer defined in DbType
    Integer(i64),
    /// Ordered float is used as f64 does not Eq and Order which are needed for Hash
    Float(OrderedFloat<f64>),
    /// Just a bool. Nothing interesting here.
    Boolean(bool),
}
