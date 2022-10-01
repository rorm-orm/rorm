use once_cell::sync::Lazy;
use regex::Regex;

pub struct Regexes {
    pub numeric_only: Regex,
    pub forbidden_character: Regex,
    pub migration_allowed_name: Regex,
}

pub static RE: Lazy<Regexes> = Lazy::new(|| Regexes {
    numeric_only: Regex::new(r#"^\d+$"#).unwrap(),
    forbidden_character: Regex::new(r#"[^a-zA-Z0-9_]"#).unwrap(),
    migration_allowed_name: Regex::new(r#"^[0-9]{4}_\w+\.toml$"#).unwrap(),
});
