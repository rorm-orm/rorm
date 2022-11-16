//! The module should be used to create sql queries for different SQL dialects.
#![warn(missing_docs)]

#[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
compile_error!("One of the features sqlite, postgres, mysql must be activated");

/// Implementation of SQL ALTER TABLE statements
pub mod alter_table;
///This module defines the conditional statements
pub mod conditional;
/// Implementation of SQL CREATE COLUMN statements
pub mod create_column;
/// Implementation of SQL CREATE INDEX
pub mod create_index;
/// Implementation of SQL CREATE TABLE statements
pub mod create_table;
/// Implementation of SQL CREATE TRIGGER statements
pub mod create_trigger;
/// Implementation of SQL DELETE operation
pub mod delete;
/// Implementation of SQL DROP TABLE statements
pub mod drop_table;
/// Definition of error types that can occur.
pub mod error;
/// Implementation of SQL INSERT statements
pub mod insert;
/// Implementation of JOIN statements
pub mod join_table;
/// Implementation of limit clauses
pub mod limit_clause;
/// Implementation of SQL ON CONFLICT extensions
pub mod on_conflict;
/// Implementation of ORDER BY expressions
pub mod ordering;
/// Implementation of SQL SELECT statements
pub mod select;
/// Implementation of identifiers in select queries
pub mod select_column;
/// Implementation of SQL UPDATE statements
pub mod update;
/// Implementation of supported datatypes
pub mod value;

mod db_specific;

use rorm_declaration::imr::{Annotation, DbType};

use crate::alter_table::{AlterTable, AlterTableData, AlterTableImpl, AlterTableOperation};
use crate::conditional::Condition;
use crate::create_column::{CreateColumnImpl, SQLAnnotation};
use crate::create_index::{CreateIndex, CreateIndexData, CreateIndexImpl};
use crate::create_table::{CreateTable, CreateTableData, CreateTableImpl};
use crate::create_trigger::{
    SQLCreateTrigger, SQLCreateTriggerOperation, SQLCreateTriggerPointInTime,
};
use crate::delete::{Delete, DeleteData, DeleteImpl};
use crate::drop_table::{DropTable, DropTableData, DropTableImpl};
use crate::insert::{Insert, InsertData, InsertImpl};
use crate::join_table::{JoinTableData, JoinTableImpl, JoinType};
use crate::on_conflict::OnConflict;
use crate::select::{Select, SelectData, SelectImpl};
use crate::update::{Update, UpdateData, UpdateImpl};
use crate::value::Value;

#[cfg(feature = "mysql")]
use crate::create_column::CreateColumnMySQLData;
#[cfg(feature = "postgres")]
use crate::create_column::CreateColumnPostgresData;
#[cfg(feature = "sqlite")]
use crate::create_column::CreateColumnSQLiteData;
use crate::ordering::OrderByEntry;
use crate::select_column::{SelectColumnData, SelectColumnImpl};

/**
The main interface for creating sql strings
*/
#[derive(Copy, Clone)]
pub enum DBImpl {
    /// Implementation of SQLite
    #[cfg(feature = "sqlite")]
    SQLite,
    /// Implementation of Postgres
    #[cfg(feature = "postgres")]
    Postgres,
    /// Implementation of MySQL / MariaDB
    #[cfg(feature = "mysql")]
    MySQL,
}

impl DBImpl {
    /**
    The entry point to create a table.

    **Parameter**:
    - `name`: Name of the table
    */
    pub fn create_table<'until_build, 'post_build>(
        &self,
        name: &'until_build str,
    ) -> impl CreateTable<'until_build, 'post_build>
    where
        'post_build: 'until_build,
    {
        let d = CreateTableData {
            name,
            columns: vec![],
            if_not_exists: false,
            lookup: vec![],
            statements: vec![],
        };

        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => CreateTableImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => CreateTableImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => CreateTableImpl::Postgres(d),
        }
    }

    /**
    The entry point to create a trigger.

    **Parameter**:
    - `name`: Name of the trigger.
    - `table_name`: Name of the table to create the trigger on.
    - `point_in_time`: [Option] of [SQLCreateTriggerPointInTime]: When to execute the trigger.
    - `operation`: [SQLCreateTriggerOperation]: The operation that invokes the trigger.
    */
    pub fn create_trigger(
        &self,
        name: &str,
        table_name: &str,
        point_in_time: Option<SQLCreateTriggerPointInTime>,
        operation: SQLCreateTriggerOperation,
    ) -> SQLCreateTrigger {
        SQLCreateTrigger {
            name: name.to_string(),
            table_name: table_name.to_string(),
            if_not_exists: false,
            point_in_time,
            operation,
            statements: vec![],
            for_each_row: false,
        }
    }

    /**
    The entry point to create an index.

    **Parameter**:
    - `name`: Name of the index.
    - `table_name`: Table to create the index on.
    */
    pub fn create_index<'until_build>(
        &self,
        name: &'until_build str,
        table_name: &'until_build str,
    ) -> impl CreateIndex<'until_build> {
        let d = CreateIndexData {
            name,
            table_name,
            unique: false,
            if_not_exists: false,
            columns: vec![],
            condition: None,
        };

        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => CreateIndexImpl::Sqlite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => CreateIndexImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => CreateIndexImpl::Postgres(d),
        }
    }

    /**
    The entry point to drop a table.

    **Parameter**:
    - `name`: Name of the table to drop.
    */
    pub fn drop_table<'until_build>(
        &self,
        name: &'until_build str,
    ) -> impl DropTable + 'until_build {
        let d = DropTableData {
            name,
            if_exists: false,
        };
        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => DropTableImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => DropTableImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => DropTableImpl::Postgres(d),
        }
    }

    /**
    The entry point to alter a table.

    **Parameter**:
    - `name`: Name of the table to execute the operation on.
    - `operation`: [AlterTableOperation]: The operation to execute.
    */
    pub fn alter_table<'until_build, 'post_build>(
        &self,
        name: &'until_build str,
        operation: AlterTableOperation<'until_build, 'post_build>,
    ) -> impl AlterTable<'post_build> + 'until_build
    where
        'post_build: 'until_build,
    {
        let d = AlterTableData {
            name,
            operation,
            lookup: vec![],
            statements: vec![],
        };

        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => AlterTableImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => AlterTableImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => AlterTableImpl::Postgres(d),
        }
    }

    /**
    The entry point to create a column in a table.

    **Parameter**:
    - `table_name`: Name of the table.
    - `name`: Name of the column.
    - `data_type`: [DbType]: Data type of the column
    - `annotations`: slice of [Annotation]: List of annotations.
    */
    pub fn create_column<'until_build, 'post_build>(
        &self,
        table_name: &'until_build str,
        name: &'until_build str,
        data_type: DbType,
        annotations: &'post_build [Annotation],
    ) -> CreateColumnImpl<'until_build, 'post_build> {
        #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
        let _ = table_name;

        // Sort the annotations
        let mut a = vec![];

        for x in annotations {
            if x.eq_shallow(&Annotation::PrimaryKey) {
                a.push(SQLAnnotation { annotation: x });
            }
        }

        for x in annotations {
            if !x.eq_shallow(&Annotation::PrimaryKey) {
                a.push(SQLAnnotation { annotation: x });
            }
        }

        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => CreateColumnImpl::SQLite(CreateColumnSQLiteData {
                name,
                table_name,
                data_type,
                annotations: a,
                statements: None,
                lookup: None,
            }),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => CreateColumnImpl::MySQL(CreateColumnMySQLData {
                name,
                data_type,
                annotations: a,
                statements: None,
                lookup: None,
            }),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => CreateColumnImpl::Postgres(CreateColumnPostgresData {
                name,
                table_name,
                data_type,
                annotations: a,
                statements: None,
            }),
        }
    }

    /**
    Build a select query.

    **Parameter**:
    - `columns`: The columns to select.
    - `from_clause`: Specifies from what to select. This can be a table name or another query itself.
    - `joins`: List of join tables.
    */
    pub fn select<'until_build, 'post_build>(
        &self,
        columns: &'until_build [SelectColumnImpl],
        from_clause: &'until_build str,
        joins: &'until_build [JoinTableImpl<'until_build, 'post_build>],
        order_by_clause: &'until_build [OrderByEntry<'until_build>],
    ) -> impl Select<'until_build, 'post_build> {
        let d = SelectData {
            join_tables: joins,
            resulting_columns: columns,
            limit: None,
            offset: None,
            from_clause,
            where_clause: None,
            distinct: false,
            lookup: vec![],
            order_by_clause,
        };
        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => SelectImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => SelectImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => SelectImpl::Postgres(d),
        }
    }

    /**
    Build an INSERT query.

    **Parameter**:
    - `into_clause`: The table to insert into.
    - `insert_columns`: The column names to insert into.
    - `insert_values`: slice of slice of [Value]: The values to insert.
    */
    pub fn insert<'until_build, 'post_build>(
        &self,
        into_clause: &'until_build str,
        insert_columns: &'until_build [&'until_build str],
        insert_values: &'until_build [&'until_build [Value<'post_build>]],
    ) -> impl Insert<'post_build>
    where
        'until_build: 'post_build,
    {
        let d = InsertData {
            into_clause,
            columns: insert_columns,
            row_values: insert_values,
            lookup: vec![],
            on_conflict: OnConflict::ABORT,
        };
        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => InsertImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => InsertImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => InsertImpl::Postgres(d),
        }
    }

    /**
    Build a delete operation.

    **Parameter**:
    - `table_name`: Name of the table to delete from.
    */
    pub fn delete<'until_build, 'post_query>(
        &self,
        table_name: &'until_build str,
    ) -> impl Delete<'until_build, 'post_query> {
        let d = DeleteData {
            model: table_name,
            lookup: vec![],
            where_clause: None,
        };
        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => DeleteImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => DeleteImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => DeleteImpl::Postgres(d),
        }
    }

    /**
    Build an update operation.

    **Parameter**:
    - `table_name`: Name of the table the updates should be executed for.
    */
    pub fn update<'until_build, 'post_query>(
        &self,
        table_name: &'until_build str,
    ) -> impl Update<'until_build, 'post_query> {
        let d = UpdateData {
            model: table_name,
            on_conflict: OnConflict::ABORT,
            updates: vec![],
            where_clause: None,
            lookup: vec![],
        };
        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => UpdateImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => UpdateImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => UpdateImpl::Postgres(d),
        }
    }

    /**
    The entry point for a JOIN expression builder.

    **Parameter**:
    - `join_type`: [JoinType]: Type for a JOIN expression
    - `table_name`: Table to perform the join on
    - `join_alias`: Alias for the join table
    - `join_condition`: [Condition] to apply to the join
    */
    pub fn join_table<'until_build, 'post_query>(
        &self,
        join_type: JoinType,
        table_name: &'until_build str,
        join_alias: &'until_build str,
        join_condition: &'until_build Condition<'post_query>,
    ) -> JoinTableImpl<'until_build, 'post_query> {
        let d = JoinTableData {
            join_type,
            table_name,
            join_alias,
            join_condition,
        };

        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => JoinTableImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => JoinTableImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => JoinTableImpl::Postgres(d),
        }
    }

    /**
    The entry point for a column selector builder.

    **Parameter**:
    - `table_name`: Optional table name
    - `column_name`: Name of the column
    - `select_alias`: Alias for the selector
     */
    pub fn select_column<'until_build>(
        &self,
        table_name: Option<&'until_build str>,
        column_name: &'until_build str,
        select_alias: Option<&'until_build str>,
    ) -> SelectColumnImpl<'until_build> {
        let d = SelectColumnData {
            table_name,
            column_name,
            select_alias,
        };

        match self {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => SelectColumnImpl::SQLite(d),
            #[cfg(feature = "mysql")]
            DBImpl::MySQL => SelectColumnImpl::MySQL(d),
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => SelectColumnImpl::Postgres(d),
        }
    }
}
