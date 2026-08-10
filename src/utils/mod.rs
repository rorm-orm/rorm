use std::io;
use std::io::Write;

use tracing::error;

pub mod imr_as_state;
pub mod indexes;
pub mod migrations;
pub mod re;

pub(crate) fn question(question: &str) -> bool {
    loop {
        print!("{question} [yN] ");
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
                error!("error: {error}");
                return false;
            }
        }
    }
}
