/// Formats the given input to a escaped postgres string.
pub(crate) fn fmt(input: &str) -> String {
    if input.contains('\'') {
        format!("'{}'", input.replace('\'', "\\'"))
    } else {
        format!("'{input}'")
    }
}

/**
The name of the check constraint enforcing a column's max length

Unlike a column, a constraint has to be named to be dropped again,
so its name has to be derivable from the table and column alone.
It is prefixed with its `table`, mirroring
[`Index::sql_name`](rorm_declaration::imr::Index::sql_name),
to make a constraint violation easy to trace back.

It deliberately doesn't end in `_check`, which is what postgres itself
generates for an unnamed column check, so that it can never collide with a
constraint rorm didn't create.
*/
pub(crate) fn max_length_check_name(table: &str, column: &str) -> String {
    format!("{table}_{column}_max_length")
}
