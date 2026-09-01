/// Formats the given input to a escaped mariadb string.
pub(crate) fn fmt(input: &str) -> String {
    if input.contains('\'') {
        format!("'{}'", input.replace('\'', "\\'"))
    } else {
        format!("'{input}'")
    }
}
