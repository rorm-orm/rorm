use serde::{Deserialize, Serialize};

use crate::imr::{DbType, Field, Index};

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

    /// Representation of a SetFieldType operation
    ///
    /// It changes an existing column's type in place, preserving its data.
    ///
    /// The type has to be one which is fully described by itself. A
    /// [`DbType::VarChar`] carries its maximum length and a [`DbType::Choices`]
    /// its enum, so neither can be set through this operation.
    #[serde(rename_all = "PascalCase")]
    SetFieldType {
        /// Name of the model
        model: String,

        /// Name of the column
        name: String,

        /// The type to change the column to
        #[serde(rename = "DbType")]
        db_type: DbType,
    },

    /// Representation of a SetFieldMaxLength operation
    ///
    /// It constrains a [`DbType::Text`] column to a maximum length. Postgres
    /// enforces it with a check constraint; sqlite has no `varchar` and never
    /// checks a string's length, so there it does nothing.
    ///
    /// A column which already has a maximum length has to
    /// [drop](Operation::DropFieldMaxLength) it first: a constraint can't be
    /// redefined, only replaced.
    #[serde(rename_all = "PascalCase")]
    SetFieldMaxLength {
        /// Name of the model
        model: String,

        /// Name of the column
        name: String,

        /// The maximum number of characters the column may hold
        max_length: i32,
    },

    /// Representation of a DropFieldMaxLength operation
    ///
    /// It removes the constraint [`Operation::SetFieldMaxLength`] created.
    #[serde(rename_all = "PascalCase")]
    DropFieldMaxLength {
        /// Name of the model
        model: String,

        /// Name of the column
        name: String,
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
