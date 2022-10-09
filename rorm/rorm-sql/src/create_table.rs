use std::fmt::Write;

use crate::error::Error;
use crate::{value, DBImpl, SQLCreateColumn};

/**
The representation of an create table operation.
*/
pub struct SQLCreateTable<'until_build, 'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) db_name: &'until_build str,
    pub(crate) name: String,
    pub(crate) columns: Vec<SQLCreateColumn<'post_build>>,
    pub(crate) if_not_exists: bool,
    pub(crate) lookup: Vec<value::Value<'post_build>>,
    pub(crate) trigger: Vec<(String, Vec<value::Value<'post_build>>)>,
}

impl<'until_build, 'post_build> SQLCreateTable<'until_build, 'post_build> {
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
    pub fn build(mut self) -> Result<(String, Vec<value::Value<'post_build>>), Error> {
        let mut s = format!(
            "CREATE TABLE{} ",
            if self.if_not_exists {
                " IF NOT EXISTS"
            } else {
                ""
            }
        );

        match self.dialect {
            DBImpl::SQLite | DBImpl::MySQL => {
                write!(s, "{}", self.name).unwrap();
            }
            _ => todo!("Not implemented yet!"),
        }

        write!(s, " (").unwrap();

        for (idx, x) in self.columns.iter().enumerate() {
            x.build(&mut s, &mut self.trigger)?;
            if idx != self.columns.len() - 1 {
                write!(s, ",\n").unwrap();
            }
        }

        write!(
            s,
            ") {}; ",
            match self.dialect {
                DBImpl::SQLite => "STRICT",
                _ => "",
            }
        )
        .unwrap();

        for (trigger, bind_params) in self.trigger {
            self.lookup.extend(bind_params);
            write!(s, "{} ", trigger).unwrap();
        }

        Ok((s, self.lookup))
    }
}
