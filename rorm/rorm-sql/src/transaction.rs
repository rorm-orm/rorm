use crate::DBImpl;

/**
Representation of a SQL transaction
*/
pub struct SQLTransaction {
    pub(crate) dialect: DBImpl,
    pub(crate) statements: Vec<String>,
}

impl SQLTransaction {
    /**
    Adds a statement to the transaction
    */
    pub fn add_statement(mut self, statement: String) -> Self {
        self.statements.push(statement);
        return self;
    }

    /**
    Finishes the current migration
    */
    pub fn finish(self) -> String {
        match self.dialect {
            DBImpl::SQLite => {
                format!("BEGIN; {} COMMIT;", self.statements.join(" "))
            }
            _ => todo!("Not implemented yet!"),
        }
    }
}
