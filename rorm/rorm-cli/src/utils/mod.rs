pub mod migrations;
pub mod re;

#[macro_export]
macro_rules! log_sql {
    ($query:expr, $do_log:expr) => {{
        let q: String = $query;
        if $do_log {
            println!("SQL: {}", q);
        }
        q
    }};
}
