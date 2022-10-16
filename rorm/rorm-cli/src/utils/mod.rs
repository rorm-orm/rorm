pub mod bind;
pub mod migrations;
pub mod re;

use std::io;
use std::io::Write;

#[macro_export]
macro_rules! log_sql {
    ($query:expr, $do_log:expr) => {{
        let log_sql_q: String = $query;
        if $do_log {
            println!("{}", log_sql_q);
        }
        log_sql_q
    }};
}

pub fn question(question: &str) -> bool {
    loop {
        print!("{} [yN] ", question);
        io::stdout().flush().expect("Flushing stdout should work!");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                input = input.trim().to_string();
                if input.trim().is_empty() || input == "N" {
                    return false;
                } else if input == "y" {
                    return true;
                }
            }
            Err(error) => {
                println!("error: {error}");
                return false;
            }
        }
    }
}
