/**
Trait representing a drop index builder.
*/
pub trait DropIndex {
    /**
    Drops the index only, if it exists.
     */
    fn if_exists(self) -> Self;

    /**
    This method is used to build the drop index statement.
     */
    fn build(self) -> String;
}

/**
The representation of data of the drop index statement.
*/
#[derive(Debug, Copy, Clone)]
pub struct DropIndexData<'until_build> {
    pub(crate) name: &'until_build str,
    pub(crate) if_exists: bool,
}

/**
Implementation of the [DropIndex] trait for the different implementations.

Should only be constructed via [crate::DBImpl::drop_index].
*/
#[derive(Debug)]
pub enum DropIndexImpl<'until_build> {
    /**
    SQLite representation of the DROP INDEX operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(DropIndexData<'until_build>),
    /**
    Postgres representation of the DROP INDEX operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(DropIndexData<'until_build>),
}

impl DropIndex for DropIndexImpl<'_> {
    fn if_exists(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            DropIndexImpl::SQLite(ref mut d) => d.if_exists = true,
            #[cfg(feature = "postgres")]
            DropIndexImpl::Postgres(ref mut d) => d.if_exists = true,
        };
        self
    }

    fn build(self) -> String {
        match self {
            #[cfg(feature = "sqlite")]
            DropIndexImpl::SQLite(d) => format!(
                "DROP INDEX{} \"{}\";",
                if d.if_exists { " IF EXISTS" } else { "" },
                d.name
            ),

            #[cfg(feature = "postgres")]
            DropIndexImpl::Postgres(d) => format!(
                "DROP INDEX{} \"{}\";",
                if d.if_exists { " IF EXISTS" } else { "" },
                d.name
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::drop_index::DropIndex;
    use crate::DBImpl;

    #[cfg(feature = "sqlite")]
    #[test]
    fn drop_index_sqlite() {
        assert_eq!(
            DBImpl::SQLite.drop_index("user_login_idx").build(),
            r#"DROP INDEX "user_login_idx";"#
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn drop_index_postgres() {
        assert_eq!(
            DBImpl::Postgres.drop_index("user_login_idx").build(),
            r#"DROP INDEX "user_login_idx";"#
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn drop_index_if_exists() {
        assert_eq!(
            DBImpl::SQLite
                .drop_index("user_login_idx")
                .if_exists()
                .build(),
            r#"DROP INDEX IF EXISTS "user_login_idx";"#
        );
    }
}
