use crate::on_conflict::OnConflict;
use crate::{DBImpl, Value};

/**
Representation of the INSERT operation in SQL.
*/
pub struct SQLInsert<'until_build, 'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) into_clause: String,
    pub(crate) columns: &'until_build [&'until_build str],
    pub(crate) row_values: &'until_build [&'until_build [Value<'post_build>]],
    pub(crate) lookup: Vec<Value<'post_build>>,
    pub(crate) on_conflict: OnConflict,
}

impl<'until_build, 'post_build> SQLInsert<'until_build, 'post_build> {
    /**
    Turns on ROLLBACK mode.

    Only useful in case of an active transaction.

    If the insert fails, the complete transaction will be rolled back.
    The default case is to just stop the transaction, but not rollback any
    prior successful executed queries.
    */
    pub fn rollback_transaction(mut self) -> Self {
        self.on_conflict = OnConflict::ROLLBACK;
        self
    }

    /**
    This method is used to build the INSERT query.

    It returns the build query as well as a vector of values to bind to it.
    */
    pub fn build(mut self) -> (String, Vec<Value<'post_build>>) {
        (
            format!(
                "INSERT {} INTO {} ({}) VALUES {};",
                match self.on_conflict {
                    OnConflict::ABORT => "OR ABORT",
                    OnConflict::ROLLBACK => "OR ROLLBACK",
                },
                self.into_clause,
                self.columns.join(", "),
                self.row_values
                    .iter()
                    .map(|x| format!(
                        "({})",
                        x.iter()
                            .map(|y| match y {
                                Value::Ident(s) => {
                                    *s
                                }
                                _ => {
                                    self.lookup.push(*y);
                                    "?"
                                }
                            })
                            .collect::<Vec<&str>>()
                            .join(", ")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            self.lookup,
        )
    }
}
