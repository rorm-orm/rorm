/**
Trait representing a drop table builder.
*/
pub trait DropTable {
    /**
    Drops the table only, if it exists.
     */
    fn if_exists(self) -> Self;

    /**
    This method is used to build the drop table statement.
     */
    fn build(self) -> String;
}

/**
The representation of data of the drop table statement.
*/
#[derive(Debug, Copy, Clone)]
pub struct DropTableData<'until_build> {
    pub(crate) name: &'until_build str,
    pub(crate) if_exists: bool,
}

/**
Implementation of the [DropTable] trait for the different implementations.

Should only be constructed via [crate::DBImpl::drop_table].
*/
#[derive(Debug)]
pub enum DropTableImpl<'until_build> {
    /**
    SQLite representation of the DROP TABLE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(DropTableData<'until_build>),
    /**
    MySQL representation of the DROP TABLE operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(DropTableData<'until_build>),
    /**
    Postgres representation of the DROP TABLE operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(DropTableData<'until_build>),
}

impl<'until_build> DropTable for DropTableImpl<'until_build> {
    fn if_exists(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            DropTableImpl::SQLite(ref mut d) => d.if_exists = true,
            #[cfg(feature = "mysql")]
            DropTableImpl::MySQL(ref mut d) => d.if_exists = true,
            #[cfg(feature = "postgres")]
            DropTableImpl::Postgres(ref mut d) => d.if_exists = true,
        };
        self
    }

    fn build(self) -> String {
        match self {
            #[cfg(feature = "sqlite")]
            DropTableImpl::SQLite(d) => format!(
                "DROP TABLE {}{};",
                d.name,
                if d.if_exists { " IF EXISTS" } else { "" }
            ),

            #[cfg(feature = "mysql")]
            DropTableImpl::MySQL(d) => format!(
                "DROP TABLE {}{};",
                d.name,
                if d.if_exists { " IF EXISTS" } else { "" }
            ),

            #[cfg(feature = "postgres")]
            DropTableImpl::Postgres(d) => format!(
                "DROP TABLE \"{}\"{};",
                d.name,
                if d.if_exists { " IF EXISTS" } else { "" }
            ),
        }
    }
}
