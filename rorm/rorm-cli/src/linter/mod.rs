use std::collections::HashMap;

use anyhow::anyhow;
use rorm_declaration::imr::InternalModelFormat;

use crate::utils::re::RE;

fn count_entries(lst: Vec<&str>) -> HashMap<&str, i32> {
    let mut m = HashMap::new();
    for x in lst {
        match m.get_mut(x) {
            None => {
                m.insert(x, 1);
            }
            Some(v) => {
                *v += 1;
            }
        }
    }

    m
}

pub fn check_internal_models(internal_models: &InternalModelFormat) -> anyhow::Result<()> {
    let model_name_counter = count_entries(
        internal_models
            .models
            .iter()
            .map(|x| x.name.as_str())
            .collect(),
    );

    for model in &internal_models.models {
        // Check for duplicate names
        if *model_name_counter.get(model.name.as_str()).unwrap() > 1 {
            return Err(anyhow!(
                "Model name {} found more than once",
                model.name.as_str()
            ));
        }

        // Check model name
        if model.name == "" {
            return Err(anyhow!("Model name must not be empty"));
        } else if RE.forbidden_character.is_match(model.name.as_str()) {
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

        let field_name_counter =
            count_entries(model.fields.iter().map(|x| x.name.as_str()).collect());

        for field in &model.fields {
            if *field_name_counter.get(field.name.as_str()).unwrap() > 1 {
                return Err(anyhow!(
                    "Field {} found more than once in model {}",
                    field.name.as_str(),
                    model.name.as_str()
                ));
            }

            // Check field name
            if field.name == "" {
                return Err(anyhow!("Field name must not be empty"));
            } else if RE.forbidden_character.is_match(field.name.as_str()) {
                return Err(anyhow!("Field name must consists of [a-zA-Z0-9_]"));
            // Reserved for internal use
            } else if field.name.starts_with("_") || field.name.ends_with("_") {
                return Err(anyhow!("Model name must not start or end with \"_\""));
            // Mysql only allows numeric table names if they are quoted
            } else if RE.numeric_only.is_match(field.name.as_str()) {
                return Err(anyhow!("Model name must not only consist of numerics"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_check_internal_models {
    use rorm_declaration::imr::{DbType, Field, InternalModelFormat, Model};

    use crate::linter::check_internal_models;

    macro_rules! test_model {
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

    macro_rules! test_field {
        ($name: ident, $test: literal, $result: literal) => {
            #[test]
            fn $name() {
                let imf = InternalModelFormat {
                    models: vec![Model {
                        name: "foobar".to_string(),
                        fields: vec![Field {
                            name: $test.to_string(),
                            annotations: vec![],
                            source_defined_at: None,
                            db_type: DbType::VarChar,
                        }],
                        source_defined_at: None,
                    }],
                };
                assert_eq!(check_internal_models(&imf).is_ok(), $result);
            }
        };
    }

    test_model!(valid, "foo", true);
    test_model!(valid_02, "foo_bar", true);
    test_model!(leading_, "_foo", false);
    test_model!(trailing_, "foo_", false);
    test_model!(empty, "", false);
    test_model!(dot, ".", false);
    test_model!(sqlite_, "sqlite_", false);
    test_model!(sqlite, "sqlite", true);
    test_model!(non_ascii, "™", false);
    test_model!(minus, "-", false);
    test_model!(numeric_only, "1234", false);
    test_model!(numeric_mixed, "123f12", true);
    test_model!(null, "\0", false);

    test_field!(valid_field, "foobar", true);
    test_field!(value_field_02, "foo_bar", true);
    test_field!(field_leading_, "_foo", false);
    test_field!(field_trailing_, "foo_", false);
    test_field!(field_empty, "", false);
    test_field!(field_non_ascii, "™", false);
    test_field!(field_minus, "-", false);
    test_field!(field_null, "\0", false);
    test_field!(field_numeric_only, "1234", false);
    test_field!(field_numeric_mixed, "1232i", true);
    test_field!(field_dot, ".", false);

    #[test]
    fn duplicate_models() {
        let m = Model {
            name: "foobar".to_string(),
            fields: vec![],
            source_defined_at: None,
        };
        let imf = InternalModelFormat {
            models: vec![m.clone(), m],
        };
        assert_eq!(check_internal_models(&imf).is_ok(), false);
    }

    #[test]
    fn duplicate_fields() {
        let f = Field {
            name: "foobar".to_string(),
            db_type: DbType::VarChar,
            annotations: vec![],
            source_defined_at: None,
        };
        let imf = InternalModelFormat {
            models: vec![Model {
                name: "foobar".to_string(),
                fields: vec![f.clone(), f],
                source_defined_at: None,
            }],
        };
        assert_eq!(check_internal_models(&imf).is_ok(), false);
    }
}
