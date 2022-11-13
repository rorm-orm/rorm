use std::fmt::Write;

/**
Trait representing a column builder.
*/
pub trait SelectColumn {
    /**
    Build the column selector in the provided String.
    */
    fn build(&self, s: &mut String);
}

/**
Representation of a select identifier.

This is due to the presence of join expressions and some reserved keywords in postgres.
 */
#[derive(Debug, Clone, Copy)]
pub struct SelectColumnData<'until_build> {
    /// Optional name of the table
    pub table_name: Option<&'until_build str>,
    /// Name of the column
    pub column_name: &'until_build str,
    /// Optional alias to set for the column
    pub select_alias: Option<&'until_build str>,
}

/**
Representation of the column selector and implementation of the [SelectColumn] trait.

Should only be constructed via [DBImpl::select_column].
 */
#[derive(Clone, Debug)]
pub enum SelectColumnImpl<'until_build> {
    /// SQLite representation of a column selector expression.
    #[cfg(feature = "sqlite")]
    SQLite(SelectColumnData<'until_build>),
    /// MySQL representation of a column selector expression.
    #[cfg(feature = "mysql")]
    MySQL(SelectColumnData<'until_build>),
    /// Postgres representation of a column selector expression.
    #[cfg(feature = "postgres")]
    Postgres(SelectColumnData<'until_build>),
}

impl<'until_build> SelectColumn for SelectColumnImpl<'until_build> {
    fn build(&self, s: &mut String) {
        match self {
            #[cfg(feature = "sqlite")]
            SelectColumnImpl::SQLite(d) => {
                if let Some(table_name) = d.table_name {
                    write!(s, "{}.", table_name).unwrap();
                }

                write!(s, "{}", d.column_name).unwrap();

                if let Some(alias) = d.select_alias {
                    write!(s, " AS {}", alias).unwrap();
                }
            }
            #[cfg(feature = "mysql")]
            SelectColumnImpl::MySQL(d) => {
                if let Some(table_name) = d.table_name {
                    write!(s, "{}.", table_name).unwrap();
                }

                write!(s, "{}", d.column_name).unwrap();

                if let Some(alias) = d.select_alias {
                    write!(s, " AS {}", alias).unwrap();
                }
            }
            #[cfg(feature = "postgres")]
            SelectColumnImpl::Postgres(d) => {
                if let Some(table_name) = d.table_name {
                    write!(s, "\"{}\".", table_name).unwrap();
                }

                write!(s, "\"{}\"", d.column_name).unwrap();

                if let Some(alias) = d.select_alias {
                    write!(s, " AS {}", alias).unwrap();
                }
            }
        }
    }
}
