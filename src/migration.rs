use serde::{Deserialize, Serialize};

use crate::imr::{Annotation, DbType, Field, Index};

/**
The presentation of a migration file
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MigrationFile {
    /// The migration of the migration file
    pub migration: Migration,
}

/**
Representation for a migration.
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Migration {
    /// Hash of the migration
    pub hash: String,

    /// Marks the migration initial state
    pub initial: bool,

    /// ID of the migration, derived from filename
    #[serde(skip)]
    pub id: u16,

    /// Name of the migration, derived from filename
    #[serde(skip)]
    pub name: String,

    /// Migration this migration depends on
    pub dependency: Option<u16>,

    /// List of migrations this migration replaces
    pub replaces: Vec<u16>,

    /// The operations to execute
    pub operations: Vec<Operation>,
}

/**
The representation for all possible database operations
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "Type")]
pub enum Operation {
    /// Representation of a CreateModel operation
    #[serde(rename_all = "PascalCase")]
    CreateModel {
        /// Name of the model
        name: String,
        /// List of fields associated to the model
        fields: Vec<Field>,
    },

    /// Representation of a RenameModel operation
    #[serde(rename_all = "PascalCase")]
    RenameModel {
        /// Old name of the model
        old: String,
        /// New name of the model
        new: String,
    },

    /// Representation of a DeleteModel operation
    #[serde(rename_all = "PascalCase")]
    DeleteModel {
        /// Name of the model
        name: String,
    },

    /// Representation of a CreateField operation
    #[serde(rename_all = "PascalCase")]
    CreateField {
        /// Name of the model
        model: String,
        /// The field that should be created
        field: Field,
    },

    /// Representation of a RenameField operation
    #[serde(rename_all = "PascalCase")]
    RenameField {
        /// Name of the table the column lives in
        table_name: String,

        /// Old name of the column
        old: String,

        /// New name of the column
        new: String,
    },

    /// Representation of an AlterField operation
    ///
    /// It changes an existing column in place, preserving its data.
    ///
    /// `make-migrations` only emits it for changes every supported dialect can
    /// perform in place; everything else stays a [`Operation::DeleteField`]
    /// followed by a [`Operation::CreateField`], which loses the column's data.
    #[serde(rename_all = "PascalCase")]
    AlterField {
        /// Name of the model
        model: String,

        /// Name of the column
        name: String,

        /// The type to change the column to
        ///
        /// It is `None` if the column's type doesn't have to be touched at all,
        /// which is worth avoiding: postgres rebuilds a column's indexes and
        /// constraints on every `ALTER COLUMN ... TYPE`,
        /// even one which doesn't actually change the type.
        ///
        /// It is always set for a [`DbType::VarChar`],
        /// whose [`Annotation::MaxLength`] is part of its type.
        #[serde(skip_serializing_if = "Option::is_none", default)]
        new_type: Option<DbType>,

        /// The column's complete new list of annotations
        ///
        /// Unlike `new_type` this is not a diff. A migration is applied without
        /// any knowledge of the column's current annotations, so the operation
        /// has to declare the state to assert instead of the change to make.
        ///
        /// It notably can't be skipped when it is unchanged: turning a
        /// `character varying(n)` into a `text` leaves the annotations equal,
        /// but moves the maximum length from the type into a constraint.
        // This serializes as an array of tables, which toml requires to come
        // after all of a table's values, so it has to stay the last field.
        new_annotations: Vec<Annotation>,
    },

    /// Representation of a DeleteField operation
    #[serde(rename_all = "PascalCase")]
    DeleteField {
        /// Name of the model
        model: String,
        /// Name of the field to delete
        name: String,
    },

    /// Representation of a CreateIndex operation
    #[serde(rename_all = "PascalCase")]
    CreateIndex {
        /// Name of the model to create the index on
        model: String,
        /// The index that should be created
        index: Index,
    },

    /// Representation of a DeleteIndex operation
    #[serde(rename_all = "PascalCase")]
    DeleteIndex {
        /// Name of the model the index was created on
        model: String,
        /// The index that should be deleted
        index: Index,
    },

    /// Representation of a RawSQL operation
    #[serde(rename_all = "PascalCase")]
    RawSQL {
        /// The provided raw sql does not change the structure of the database.
        /// The migrator can assume that the layout stayed the same and will continue
        /// generating new migrations based on `.models.json`
        #[serde(default)]
        structure_safe: bool,
        /// SQL for sqlite
        #[serde(rename = "SQLite")]
        sqlite: String,
        /// SQL for postgres
        #[serde(rename = "Postgres")]
        postgres: String,
        /// SQL for mysql
        #[serde(rename = "MySQL")]
        mysql: String,
    },
}
