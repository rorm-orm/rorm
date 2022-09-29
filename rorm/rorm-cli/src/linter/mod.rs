use anyhow::anyhow;
use rorm_declaration::imr::InternalModelFormat;

use crate::utils::re::RE;

pub fn check_internal_models(internal_models: &InternalModelFormat) -> anyhow::Result<()> {
    for model in &internal_models.models {
        if model.name == "" {
            return Err(anyhow!("Model name must not be empty"));
        } else if RE.table_forbidden_character.is_match(model.name.as_str()) {
            return Err(anyhow!("Model name must consists of [a-zA-Z0-9_]"));
        // Reserved for internal use
        } else if model.name.starts_with("_") || model.name.ends_with("_") {
            return Err(anyhow!("Model name must not start or end with \"_\""));
        // Sqlite reserved table names
        } else if model.name.starts_with("sqlite_") {
            return Err(anyhow!("Model name must not start with \"sqlite_\""));
        // Mysql only allows numeric table names if they are quoted
        } else if RE.numeric_only.is_match(model.name.as_str()) {
            return Err(anyhow!("Model name must not only consist of numerics"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_check_internal_models {
    use rorm_declaration::imr::{InternalModelFormat, Model};

    use crate::linter::check_internal_models;

    macro_rules! impl_test {
        ($name: ident, $test: literal, $result: literal) => {
            #[test]
            fn $name() {
                let imf = InternalModelFormat {
                    models: vec![Model {
                        name: $test.to_string(),
                        fields: vec![],
                        source_defined_at: None,
                    }],
                };
                assert_eq!(check_internal_models(&imf).is_ok(), $result);
            }
        };
    }

    impl_test!(valid, "foo", true);
    impl_test!(valid_02, "foo_bar", true);
    impl_test!(leading_, "_foo", false);
    impl_test!(trailing_, "foo_", false);
    impl_test!(empty, "", false);
    impl_test!(dot, ".", false);
    impl_test!(sqlite_, "sqlite_", false);
    impl_test!(sqlite, "sqlite", true);
    impl_test!(non_ascii, "™", false);
    impl_test!(minus, "-", false);
    impl_test!(numeric_only, "1234", false);
    impl_test!(numeric_mixed, "123f12", true);
    impl_test!(null, "\0", false);
}
