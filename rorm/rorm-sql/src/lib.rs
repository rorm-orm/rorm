use crate::alter_table::{SQLAlterTable, SQLAlterTableOperation};
use crate::create_index::SQLCreateIndex;
use crate::create_table::SQLCreateTable;
use crate::create_trigger::{
    SQLCreateTrigger, SQLCreateTriggerOperation, SQLCreateTriggerPointInTime,
};
use crate::drop_table::SQLDropTable;
use crate::transaction::SQLTransaction;

pub mod alter_table;
pub mod create_index;
pub mod create_table;
pub mod create_trigger;
pub mod drop_table;
pub mod transaction;

/**
The main interface for creating sql strings
*/
pub enum DBImpl {
    SQLite,
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
        }
    }

    /**
    The entry point to create a trigger.

    `name`: [&str]: Name of the trigger.
    `table_name`: [&str]: Name of the table to create the trigger on.
    `point_in_time`: [Option<SQLCreateTriggerPointInTime]: When to execute the trigger.
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
        }
    }

    /**
    The entry point to create an index.

    `name`: [&str]: Name of the index.
    `table_name`: [&str]: Table to create the index on.
    ``
    */
    pub fn create_index(&self, name: &str, table_name: &str) -> SQLCreateIndex {
        match self {
            DBImpl::SQLite => SQLCreateIndex {
                name: name.to_string(),
                table_name: table_name.to_string(),
                unique: false,
                if_not_exists: false,
                columns: vec![],
                condition: None,
            },
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DBImpl;
    use rorm_common::imr::{Annotation, DbType};

    #[test]
    fn sqlite_01() {
        assert_eq!(
            DBImpl::SQLite.create_table("test").build().unwrap(),
            "CREATE TABLE test () STRICT;".to_string()
        );
    }

    #[test]
    fn sqlite_02() {
        assert_eq!(
            DBImpl::SQLite
                .create_table("test")
                .if_not_exists()
                .build()
                .unwrap(),
            "CREATE TABLE IF NOT EXISTS test () STRICT;".to_string()
        )
    }

    #[test]
    fn sqlite_03() {
        assert_eq!(
            DBImpl::SQLite
                .create_table("test")
                .add_column("id", DbType::UInt64, vec![])
                .build()
                .unwrap(),
            "CREATE TABLE test (id INTEGER) STRICT;".to_string()
        )
    }

    #[test]
    fn sqlite_04() {
        assert_eq!(
            DBImpl::SQLite
                .create_table("test")
                .add_column("id", DbType::UInt64, vec![Annotation::PrimaryKey])
                .build()
                .unwrap(),
            "CREATE TABLE test (id INTEGER PRIMARY KEY) STRICT;"
        )
    }

    #[test]
    fn sqlite_05() {
        assert_eq!(
            DBImpl::SQLite
                .create_table("test")
                .add_column("id", DbType::UInt64, vec![Annotation::PrimaryKey])
                .add_column("foo", DbType::VarChar, vec![Annotation::NotNull])
                .build()
                .unwrap(),
            "CREATE TABLE test (id INTEGER PRIMARY KEY,foo TEXT NOT NULL) STRICT;"
        )
    }
}
