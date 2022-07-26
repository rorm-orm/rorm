use crate::DBImpl;

/**
The representation of the drop table statement.
*/
pub struct SQLDropTable {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) if_exists: bool,
}

impl SQLDropTable {
    /**
    Drops the table only, if it exists.
     */
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        return self;
    }

    /**
    This method is used to build the drop table statement.
    */
    pub fn build(self) -> String {
        return match self.dialect {
            DBImpl::SQLite => {
                format!(
                    "DROP TABLE {} {};",
                    self.name.as_str(),
                    if self.if_exists { "IF EXISTS" } else { "" }
                )
            }
        };
    }
}
