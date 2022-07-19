use crate::create_table::SQLCreateTable;
use crate::create_trigger::{
    SQLCreateTrigger, SQLCreateTriggerOperation, SQLCreateTriggerPointInTime,
};

pub mod create_table;
pub mod create_trigger;

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
