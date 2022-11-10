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
*/
pub enum CreateIndexImpl<'until_build> {
    /**
    SQLite representation of the CREATE INDEX operation.
     */
    #[cfg(feature = "sqlite")]
    Sqlite(CreateIndexData<'until_build>),
    /**
    MySQL representation of the CREATE INDEX operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(CreateIndexData<'until_build>),
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
            #[cfg(feature = "mysql")]
            CreateIndexImpl::MySQL(ref mut d) => d.unique = true,
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.unique = true,
        };
        self
    }

    fn if_not_exists(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.if_not_exists = true,
            #[cfg(feature = "mysql")]
            CreateIndexImpl::MySQL(ref mut d) => d.if_not_exists = true,
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.if_not_exists = true,
        };
        self
    }

    fn add_column(mut self, column: &'until_build str) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.columns.push(column),
            #[cfg(feature = "mysql")]
            CreateIndexImpl::MySQL(ref mut d) => d.columns.push(column),
            #[cfg(feature = "postgres")]
            CreateIndexImpl::Postgres(ref mut d) => d.columns.push(column),
        }
        self
    }

    fn set_condition(mut self, condition: String) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateIndexImpl::Sqlite(ref mut d) => d.condition = Some(condition),
            #[cfg(feature = "mysql")]
            CreateIndexImpl::MySQL(ref mut d) => d.condition = Some(condition),
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
                    "CREATE {} INDEX{} {} ON {} ({}) {};",
                    if d.unique { "UNIQUE" } else { "" },
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name,
                    d.table_name,
                    d.columns.join(", "),
                    d.condition.as_ref().map_or("", |x| x.as_str()),
                ))
            }
            #[cfg(feature = "mysql")]
            CreateIndexImpl::MySQL(d) => {
                if d.columns.is_empty() {
                    return Err(Error::SQLBuildError(format!(
                        "Couldn't create index on {}: Missing column(s) to create the index on",
                        d.table_name
                    )));
                }

                Ok(format!(
                    "CREATE {} INDEX{} {} ON {} ({});",
                    if d.unique { "UNIQUE" } else { "" },
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name,
                    d.table_name,
                    d.columns.join(", "),
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
                    "CREATE{} INDEX{} {} ON {} ({}){};",
                    if d.unique { " UNIQUE" } else { "" },
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name,
                    d.table_name,
                    d.columns.join(", "),
                    match d.condition {
                        None => String::from(""),
                        Some(cond) => format!(" WHERE {}", cond.as_str()),
                    }
                ))
            }
        }
    }
}
