/**
Representation of an ON CONFLICT case in SQL.

// TODO: Check ignore cases for different database backends
// Mariadb: https://mariadb.com/kb/en/insert-ignore/
*/
#[derive(Debug, Copy, Clone)]
pub enum OnConflict {
    /// Aborts the current operation and rolls back all changes made from the current operation.
    /// In case of an active transaction only the current statement is affected.
    /// Prior successfully executed statement won't be rolled back
    ABORT,
    /// In case of an active transaction rolls back all statements.
    /// If there's no transaction, the behaviour is equivalent with [OnConflict::ABORT]
    ROLLBACK,
}
