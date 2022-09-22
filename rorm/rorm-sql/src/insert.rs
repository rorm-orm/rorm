use crate::{DBImpl, Value};

/**
Representation of the INSERT operation in SQL.
*/
pub struct SQLInsert<'until_build, 'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) into_clause: String,
    pub(crate) columns: &'until_build [&'until_build str],
    pub(crate) values: &'until_build [Value<'post_build>],
    pub(crate) lookup: Vec<Value<'post_build>>,
}

impl<'until_build, 'post_build> SQLInsert<'until_build, 'post_build> {
    /**
    This method is used to build the INSERT query.

    It returns the build query as well as a vector of values to bind to it.
    */
    pub fn build(mut self) -> (String, Vec<Value<'post_build>>) {
        match self.dialect {
            DBImpl::SQLite => (
                format!(
                    "INSERT INTO {} ({}) VALUES ({});",
                    self.into_clause,
                    self.columns.join(", "),
                    self.values
                        .iter()
                        .map(|x| match x {
                            Value::Ident(s) => {
                                *s
                            }
                            _ => {
                                self.lookup.push(*x);
                                "?"
                            }
                        })
                        .collect::<Vec<&str>>()
                        .join(", ")
                ),
                self.lookup,
            ),
            _ => todo!("Not implemented yet"),
        }
    }
}
