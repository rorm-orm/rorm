use std::collections::HashMap;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use rorm_declaration::imr::{Annotation, DefaultValue, InternalModelFormat};

use crate::utils::re::RE;

struct AnnotationReqs {
    forbidden: HashMap<u64, Vec<Annotation>>,
    required: HashMap<u64, Vec<Annotation>>,
}

static ANNOTATION_REQS: Lazy<AnnotationReqs> = Lazy::new(|| {
    let forbidden = HashMap::from([
        (
            Annotation::AutoCreateTime.hash_shallow(),
            vec![
                Annotation::AutoUpdateTime,
                Annotation::AutoIncrement,
                Annotation::Choices(vec![]),
                Annotation::DefaultValue(DefaultValue::Boolean(true)),
                Annotation::MaxLength(0),
                Annotation::NotNull,
                Annotation::PrimaryKey,
                Annotation::Unique,
            ],
        ),
        (
            Annotation::AutoUpdateTime.hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoIncrement,
                Annotation::Choices(vec![]),
                Annotation::DefaultValue(DefaultValue::Boolean(true)),
                Annotation::MaxLength(0),
                Annotation::NotNull,
                Annotation::PrimaryKey,
                Annotation::Unique,
            ],
        ),
        (
            Annotation::AutoIncrement.hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoUpdateTime,
                Annotation::Choices(vec![]),
                Annotation::MaxLength(0),
            ],
        ),
        (
            Annotation::Choices(vec![]).hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoUpdateTime,
                Annotation::AutoIncrement,
                Annotation::MaxLength(0),
                Annotation::PrimaryKey,
                Annotation::Unique,
            ],
        ),
        (
            Annotation::DefaultValue(DefaultValue::Boolean(true)).hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoUpdateTime,
                Annotation::AutoIncrement,
                Annotation::PrimaryKey,
                Annotation::Unique,
            ],
        ),
        (
            Annotation::Index(None).hash_shallow(),
            vec![Annotation::PrimaryKey],
        ),
        (
            Annotation::MaxLength(0).hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoUpdateTime,
                Annotation::AutoIncrement,
            ],
        ),
        (
            Annotation::NotNull.hash_shallow(),
            vec![Annotation::PrimaryKey],
        ),
        (
            Annotation::PrimaryKey.hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoUpdateTime,
                Annotation::NotNull,
                Annotation::Choices(vec![]),
                Annotation::Index(None),
                Annotation::DefaultValue(DefaultValue::Boolean(true)),
            ],
        ),
        (
            Annotation::Unique.hash_shallow(),
            vec![
                Annotation::AutoCreateTime,
                Annotation::AutoUpdateTime,
                Annotation::DefaultValue(DefaultValue::Boolean(true)),
                Annotation::Choices(vec![]),
            ],
        ),
    ]);

    let required = HashMap::from([
        (Annotation::AutoCreateTime.hash_shallow(), vec![]),
        (Annotation::AutoUpdateTime.hash_shallow(), vec![]),
        (Annotation::AutoIncrement.hash_shallow(), vec![]),
        (Annotation::Choices(vec![]).hash_shallow(), vec![]),
        (
            Annotation::DefaultValue(DefaultValue::Boolean(true)).hash_shallow(),
            vec![],
        ),
        (Annotation::Index(None).hash_shallow(), vec![]),
        (Annotation::MaxLength(0).hash_shallow(), vec![]),
        (Annotation::NotNull.hash_shallow(), vec![]),
        (Annotation::PrimaryKey.hash_shallow(), vec![]),
        (Annotation::Unique.hash_shallow(), vec![]),
    ]);

    AnnotationReqs {
        forbidden,
        required,
    }
});

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
            return Err(anyhow!(
                "Model name must consists of [a-zA-Z0-9_]. Found: {}.",
                model.name.as_str()
            ));
        // Reserved for internal use
        } else if model.name.starts_with("_") || model.name.ends_with("_") {
            return Err(anyhow!(
                "Model name must not start or end with \"_\". Found: {}.",
                model.name.as_str()
            ));
        // Sqlite reserved table names
        } else if model.name.starts_with("sqlite_") {
            return Err(anyhow!(
                "Model name must not start with \"sqlite_\". Found {}.",
                model.name.as_str()
            ));
        // Mysql only allows numeric table names if they are quoted
        } else if RE.numeric_only.is_match(model.name.as_str()) {
            return Err(anyhow!(
                "Model name must not only consist of numerics. Found {}.",
                model.name.as_str()
            ));
        }

        let field_name_counter =
            count_entries(model.fields.iter().map(|x| x.name.as_str()).collect());

        let mut primary_key = false;

        if model.fields.is_empty() {
            return Err(anyhow!(
                "Model {} does not contain any fields.",
                model.name.as_str()
            ));
        }

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
                return Err(anyhow!(
                    "Field name in model {} is empty",
                    field.name.as_str()
                ));
            } else if RE.forbidden_character.is_match(field.name.as_str()) {
                return Err(anyhow!("Field name must consists of [a-zA-Z0-9_]"));
            // Reserved for internal use
            } else if field.name.starts_with("_") || field.name.ends_with("_") {
                return Err(anyhow!("Model name must not start or end with \"_\""));
            // Mysql only allows numeric table names if they are quoted
            } else if RE.numeric_only.is_match(field.name.as_str()) {
                return Err(anyhow!("Model name must not only consist of numerics"));
            }

            // Check forbidden Annotation combinations
            for annotation in &field.annotations {
                if annotation == &Annotation::PrimaryKey {
                    primary_key = true;
                }

                let forbidden = ANNOTATION_REQS
                    .forbidden
                    .get(&annotation.hash_shallow())
                    .expect("There should be cases for every Annotation");
                let required = ANNOTATION_REQS
                    .required
                    .get(&annotation.hash_shallow())
                    .expect("There should be cases for every Annotation");

                for field_annotation in &field.annotations {
                    if forbidden.contains(field_annotation) {
                        return Err(anyhow!(
                            "Found {:?} on field {} of model {}, which is forbidden by {:?}",
                            field_annotation,
                            field.name.as_str(),
                            model.name.as_str(),
                            annotation,
                        ));
                    }
                }

                for required_annotation in required {
                    if !field.annotations.contains(required_annotation) {
                        return Err(anyhow!(
                            "Annotation {:?} on field {} of model {} requires {:?}.",
                            field.annotations,
                            field.name.as_str(),
                            model.name.as_str(),
                            required_annotation,
                        ));
                    }
                }
            }
        }

        if !primary_key {
            return Err(anyhow!(
                "Model {} misses a primary key.",
                model.name.as_str()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_check_internal_models {
    use rorm_declaration::imr::{Annotation, DbType, Field, InternalModelFormat, Model};
    use strum::IntoEnumIterator;

    use crate::linter::{check_internal_models, ANNOTATION_REQS};

    macro_rules! test_model {
        ($name: ident, $test: literal, $result: literal) => {
            #[test]
            fn $name() {
                let imf = InternalModelFormat {
                    models: vec![Model {
                        name: $test.to_string(),
                        fields: vec![Field {
                            annotations: vec![Annotation::PrimaryKey],
                            db_type: DbType::Int64,
                            name: "primary".to_string(),
                            source_defined_at: None,
                        }],
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
                            annotations: vec![Annotation::PrimaryKey],
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
    fn empty_field() {
        let imf = InternalModelFormat {
            models: vec![Model {
                name: "foobar".to_string(),
                fields: vec![],
                source_defined_at: None,
            }],
        };
        assert!(check_internal_models(&imf).is_err());
    }

    #[test]
    fn duplicate_models() {
        let m = Model {
            name: "foobar".to_string(),
            fields: vec![Field {
                name: "foobar".to_string(),
                source_defined_at: None,
                db_type: DbType::Int64,
                annotations: vec![],
            }],
            source_defined_at: None,
        };
        let imf = InternalModelFormat {
            models: vec![m.clone(), m],
        };
        assert!(check_internal_models(&imf).is_err());
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
        assert!(check_internal_models(&imf).is_err());
    }

    #[test]
    fn check_annotation_reqs_required() {
        for a in Annotation::iter() {
            if !ANNOTATION_REQS.required.contains_key(&a.hash_shallow()) {
                println!("Required annotations does not contain {:?}", a);
                assert!(false);
            }
        }
    }

    #[test]
    fn check_annotation_reqs_forbidden() {
        for a in Annotation::iter() {
            if !ANNOTATION_REQS.forbidden.contains_key(&a.hash_shallow()) {
                println!("Forbidden annotations does not contain {:?}", a);
                assert!(false);
            }
        }
    }
}
