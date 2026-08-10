use crate::error::Error;

/**
Representation of a CREATE INDEX builder.
*/
pub trait CreateIndex<'until_build> {
    /**
    Creates a unique index.

    Null values are considered different from all other null values.
     */
    fn unique(self) -> Self;

    /**
    Creates the index only if it doesn't exist yet.
     */
    fn if_not_exists(self) -> Self;

    /**
    Adds a column to the index.

    **Parameter**:
    - `column`: String representing the column to index.
     */
    fn add_column(self, column: &'until_build str) -> Self;

    /**
    Sets the condition to apply. This will build a partial index.

    **Parameter**:
    - `condition`: String representing condition to apply the index to
     */
    fn set_condition(self, condition: String) -> Self;

    /**
    This method is used to build the create index operation
     */
    fn build(self) -> Result<String, Error>;
}

/**
Representation of a create index operation
*/
pub struct CreateIndexData<'until_build> {
    pub(crate) name: &'until_build str,
    pub(crate) table_name: &'until_build str,
    pub(crate) unique: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) columns: Vec<&'until_build str>,
    pub(crate) condition: Option<String>,
}

/**
Implementation of database specific implementations of the [CreateIndex] trait.

Should only be constructed via [crate::DBImpl::create_index].
*/
pub enum CreateIndexImpl<'until_build> {
    /**
    SQLite representation of the CREATE INDEX operation.
     */
    #[cfg(feature = "sqlite")]
    Sqlite(CreateIndexData<'until_build>),
    /**
    Postgres representation of the CREATE INDEX operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(CreateIndexData<'until_build>),
}

impl<'until_build> CreateIndex<'until_build> for CreateIndexImpl<'until_build> {
    fn unique(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.unique = true,
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.unique = true,
        };
        self
    }

    fn if_not_exists(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.if_not_exists = true,
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.if_not_exists = true,
        };
        self
    }

    fn add_column(mut self, column: &'until_build str) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.columns.push(column),
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.columns.push(column),
        }
        self
    }

    fn set_condition(mut self, condition: String) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.condition = Some(condition),
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.condition = Some(condition),
        }
        self
    }

    fn build(self) -> Result<String, Error> {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(d) => {
                if d.columns.is_empty() {
                    return Err(Error::SQLBuildError(format!(
                        "Couldn't create index on {}: Missing column(s) to create the index on",
                        d.table_name
                    )));
                }

                Ok(format!(
                    "CREATE{} INDEX{} \"{}\" ON \"{}\" ({}){};",
                    if d.unique { " UNIQUE" } else { "" },
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name,
                    d.table_name,
                    quote_columns(&d.columns),
                    match d.condition {
                        None => String::from(""),
                        Some(cond) => format!(" WHERE {}", cond.as_str()),
                    }
                ))
            }
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(d) => {
                if d.columns.is_empty() {
                    return Err(Error::SQLBuildError(format!(
                        "Couldn't create index on {}: Missing column(s) to create the index on",
                        d.table_name
                    )));
                }

                Ok(format!(
                    "CREATE{} INDEX{} \"{}\" ON \"{}\" ({}){};",
                    if d.unique { " UNIQUE" } else { "" },
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name,
                    d.table_name,
                    quote_columns(&d.columns),
                    match d.condition {
                        None => String::from(""),
                        Some(cond) => format!(" WHERE {}", cond.as_str()),
                    }
                ))
            }
        }
    }
}

/// Joins the `columns` into the comma separated list of a CREATE INDEX statement
fn quote_columns(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|column| format!("\"{column}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod test {
    use crate::create_index::CreateIndex;
    use crate::DBImpl;

    #[cfg(feature = "sqlite")]
    #[test]
    fn single_column_sqlite() {
        assert_eq!(
            DBImpl::SQLite
                .create_index("user_login_idx", "user")
                .add_column("login")
                .build()
                .unwrap(),
            r#"CREATE INDEX "user_login_idx" ON "user" ("login");"#
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn single_column_postgres() {
        assert_eq!(
            DBImpl::Postgres
                .create_index("user_login_idx", "user")
                .add_column("login")
                .build()
                .unwrap(),
            r#"CREATE INDEX "user_login_idx" ON "user" ("login");"#
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn multiple_columns_sqlite() {
        assert_eq!(
            DBImpl::SQLite
                .create_index("user_full_name_idx", "user")
                .add_column("last_name")
                .add_column("first_name")
                .build()
                .unwrap(),
            r#"CREATE INDEX "user_full_name_idx" ON "user" ("last_name", "first_name");"#
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn multiple_columns_postgres() {
        assert_eq!(
            DBImpl::Postgres
                .create_index("user_full_name_idx", "user")
                .add_column("last_name")
                .add_column("first_name")
                .build()
                .unwrap(),
            r#"CREATE INDEX "user_full_name_idx" ON "user" ("last_name", "first_name");"#
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn unique_and_if_not_exists_sqlite() {
        assert_eq!(
            DBImpl::SQLite
                .create_index("user_login_idx", "user")
                .add_column("login")
                .unique()
                .if_not_exists()
                .build()
                .unwrap(),
            r#"CREATE UNIQUE INDEX IF NOT EXISTS "user_login_idx" ON "user" ("login");"#
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn condition_sqlite() {
        assert_eq!(
            DBImpl::SQLite
                .create_index("user_login_idx", "user")
                .add_column("login")
                .set_condition("\"login\" IS NOT NULL".to_string())
                .build()
                .unwrap(),
            r#"CREATE INDEX "user_login_idx" ON "user" ("login") WHERE "login" IS NOT NULL;"#
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn missing_columns_is_an_error() {
        assert!(DBImpl::SQLite
            .create_index("user_login_idx", "user")
            .build()
            .is_err());
    }
}
