/**
Formats the given input to a escaped postgres string.
*/
pub(crate) fn fmt(input: &str) -> String {
    if input.contains("'") {
        format!("'{}'", input.replace("'", "\\'"))
    } else {
        format!("'{}'", input)
    }
}
