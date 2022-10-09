use crate::{value, DBImpl, SQLCreateColumn};

/**
The representation of an create table operation.
*/
pub struct SQLCreateTable<'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) columns: Vec<SQLCreateColumn<'post_build>>,
    pub(crate) if_not_exists: bool,
    pub(crate) lookup: Vec<value::Value<'post_build>>,
    pub(crate) trigger: Vec<(String, Vec<value::Value<'post_build>>)>,
}

impl<'post_build> SQLCreateTable<'post_build> {
    /**
    Add a column to the table.
    */
    pub fn add_column(mut self, column: SQLCreateColumn<'post_build>) -> Self {
        self.columns.push(column);
        self
    }

    /**
    Sets the IF NOT EXISTS trait on the table
    */
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /**
    This method is used to convert the current state for the given dialect in a [String].
    */
    pub fn build(mut self) -> (String, Vec<value::Value<'post_build>>) {
        match self.dialect {
            DBImpl::SQLite => (
                format!(
                    r#"CREATE TABLE{} {} ({}) STRICT;{}"#,
                    if self.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    self.name,
                    self.columns
                        .iter()
                        .map(|x| x.build(&mut self.trigger))
                        .collect::<Vec<String>>()
                        .join(", "),
                    self.trigger
                        .into_iter()
                        .map(|(trigger, bind_params)| {
                            self.lookup.extend(bind_params);
                            trigger
                        })
                        .collect::<Vec<String>>()
                        .join(" "),
                ),
                self.lookup,
            ),
            _ => todo!("Not implemented yet!"),
        }
    }
}
