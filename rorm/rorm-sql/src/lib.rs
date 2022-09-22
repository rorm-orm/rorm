//! The module should be used to create sql queries for different SQL dialects.
#![warn(missing_docs)]

/// Implementation of SQL ALTER TABLE statements
pub mod alter_table;
///This module defines the conditional statements
pub mod conditional;
/// Implementation of SQL CREATE COLUMN statements
pub mod create_column;
/// Implementation of SQL CREATE INDEX statements
pub mod create_index;
/// Implementation of SQL CREATE TABLE statements
pub mod create_table;
/// Implementation of SQL CREATE TRIGGER statements
pub mod create_trigger;
/// Implementation of SQL DROP TABLE statements
pub mod drop_table;
/// Definition of error types that can occur.
pub mod error;
/// Implementation of SQL INSERT statements
pub mod insert;
/// Implementation of SQL SELECT statements
pub mod select;
/// Implementation of SQL Transactions
pub mod transaction;
/// Implementation of supported datatypes
pub mod value;

use rorm_declaration::imr::{Annotation, DbType};

use crate::alter_table::{SQLAlterTable, SQLAlterTableOperation};
use crate::create_column::{SQLAnnotation, SQLCreateColumn};
use crate::create_index::SQLCreateIndex;
use crate::create_table::SQLCreateTable;
use crate::create_trigger::{
    SQLCreateTrigger, SQLCreateTriggerOperation, SQLCreateTriggerPointInTime,
};
use crate::drop_table::SQLDropTable;
use crate::insert::SQLInsert;
use crate::select::SQLSelect;
use crate::transaction::SQLTransaction;
use crate::value::Value;

/**
The main interface for creating sql strings
*/
pub enum DBImpl {
    /// Implementation of SQLite
    SQLite,
    /// Implementation of Postgres
    Postgres,
    /// Implementation of MySQL / MariaDB
    MySQL,
}

impl DBImpl {
    /**
    The entry point to create a table.

    `name`: [&str]: Name of the table
    */
    pub fn create_table(&self, name: &str) -> SQLCreateTable {
        match self {
            DBImpl::SQLite { .. } => SQLCreateTable {
                dialect: DBImpl::SQLite,
                name: name.to_string(),
                columns: vec![],
                if_not_exists: false,
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    The entry point to create a trigger.

    `name`: [&str]: Name of the trigger.
    `table_name`: [&str]: Name of the table to create the trigger on.
    `point_in_time`: [Option<SQLCreateTriggerPointInTime>]: When to execute the trigger.
    `operation`: [SQLCreateTriggerOperation]: The operation that invokes the trigger.
    */
    pub fn create_trigger(
        &self,
        name: &str,
        table_name: &str,
        point_in_time: Option<SQLCreateTriggerPointInTime>,
        operation: SQLCreateTriggerOperation,
    ) -> SQLCreateTrigger {
        match self {
            DBImpl::SQLite => SQLCreateTrigger {
                dialect: DBImpl::SQLite,
                name: name.to_string(),
                table_name: table_name.to_string(),
                if_not_exists: false,
                point_in_time,
                operation,
                statements: vec![],
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    The entry point to create an index.

    `name`: [&str]: Name of the index.
    `table_name`: [&str]: Table to create the index on.
    */
    pub fn create_index(&self, name: &str, table_name: &str) -> SQLCreateIndex {
        match self {
            DBImpl::SQLite => SQLCreateIndex {
                dialect: DBImpl::SQLite,
                name: name.to_string(),
                table_name: table_name.to_string(),
                unique: false,
                if_not_exists: false,
                columns: vec![],
                condition: None,
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    The entry point to start a transaction
    */
    pub fn start_transaction(&self) -> SQLTransaction {
        match self {
            DBImpl::SQLite => SQLTransaction {
                dialect: DBImpl::SQLite,
                statements: vec![],
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    The entry point to drop a table.

    `name`: [&str]: Name of the table to drop.
    */
    pub fn drop_table(&self, name: &str) -> SQLDropTable {
        match self {
            DBImpl::SQLite => SQLDropTable {
                dialect: DBImpl::SQLite,
                name: name.to_string(),
                if_exists: false,
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    The entry point to alter a table.

    `name`: [&str]: Name of the table to execute the operation on.
    `operation`: [crate::alter_table::SQLAlterTableOperation]: The operation to execute.
    */
    pub fn alter_table(&self, name: &str, operation: SQLAlterTableOperation) -> SQLAlterTable {
        match self {
            DBImpl::SQLite => SQLAlterTable {
                dialect: DBImpl::SQLite,
                name: name.to_string(),
                operation,
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    The entry point to create a column in a table.

    - `table_name`: [&str]: Name of the table.
    - `name`: [&str]: Name of the column.
    - `data_type`: [DbType]: Data type of the column
    - `annotations`: [Vec<Annotation>]: List of annotations.
    */
    pub fn create_column(
        &self,
        table_name: &str,
        name: &str,
        data_type: DbType,
        annotations: Vec<Annotation>,
    ) -> SQLCreateColumn {
        match self {
            DBImpl::SQLite => SQLCreateColumn {
                dialect: DBImpl::SQLite,
                name: name.to_string(),
                table_name: table_name.to_string(),
                data_type,
                annotations: annotations
                    .into_iter()
                    .map(|x| SQLAnnotation { annotation: x })
                    .collect(),
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    Build a select query.

    **Parameter**:
    - `columns`: The columns to select.
    - `from_clause` specifies from what to select. This can be a table name or another query itself.
    */
    pub fn select<'until_build>(
        &self,
        columns: &'until_build [&'until_build str],
        from_clause: &str,
    ) -> SQLSelect<'until_build, '_> {
        match self {
            DBImpl::SQLite => SQLSelect {
                dialect: DBImpl::SQLite,
                resulting_columns: columns,
                from_clause: from_clause.to_string(),
                where_clause: None,
                limit: None,
                offset: None,
                distinct: false,
                lookup: vec![],
            },
            _ => todo!("Not implemented yet!"),
        }
    }

    /**
    Build an INSERT query.

    **Parameter**:
    - `into_clause`: The table to insert into.
    - `insert_columns`: The column names to insert into.
    - `insert_values`: The values to insert.
    */
    pub fn insert<'until_build, 'post_build>(
        &self,
        into_clause: &str,
        insert_columns: &'until_build [&'until_build str],
        insert_values: &'until_build [Value<'post_build>],
    ) -> SQLInsert<'until_build, 'post_build> {
        match self {
            DBImpl::SQLite => SQLInsert {
                dialect: DBImpl::SQLite,
                into_clause: into_clause.to_string(),
                columns: insert_columns,
                values: insert_values,
                lookup: vec![],
            },
            _ => todo!("Not implemented yet!"),
        }
    }
}
