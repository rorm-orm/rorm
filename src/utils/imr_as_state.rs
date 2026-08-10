//! Extension trait to treat [`imr`] types as queryable and updatable state

use rorm_declaration::imr::{Field, InternalModelFormat, Model};
use rorm_declaration::migration::Operation;
use thiserror::Error;

use crate::utils::indexes;

/// Extension trait for [`InternalModelFormat`]
///
/// It provides methods to treat `InternalModelFormat` as an updatable
/// representation of the database's current schema.
/// Most noteworthy is [`apply_operation`](InternalModelFormatExt::apply_operation)
/// which modifies the `InternalModelFormat`
/// to reflect the changes made by a single migration `Operation`.
pub trait InternalModelFormatExt {
    /// Modifies `self` to reflect the changes made by a single migration `Operation`.
    fn apply_operation(&mut self, operation: &Operation) -> Result<(), ApplyOperationError> {
        match operation {
            Operation::CreateModel { name, fields } => {
                check_model_not_exists(self.as_ref(), name)?;
                self.as_mut().models.push(Model {
                    name: name.clone(),
                    fields: fields.iter().map(indexes::without_indexes).collect(),
                    source_defined_at: None,
                });
            }
            Operation::RenameModel { old, new } => {
                check_model_not_exists(self.as_ref(), new)?;
                let model = get_model(self.as_mut(), old)?;
                model.name = new.clone();
            }
            Operation::DeleteModel { name } => {
                check_model_exists(self.as_ref(), name)?;
                self.as_mut().models.retain(|x| x.name != *name);
            }
            Operation::CreateField { model, field } => {
                let model = get_model(self.as_mut(), model)?;
                check_field_not_exists(model, &field.name)?;
                model.fields.push(indexes::without_indexes(field));
            }
            Operation::RenameField {
                table_name,
                old,
                new,
            } => {
                let model = get_model(self.as_mut(), table_name)?;
                check_field_not_exists(model, new)?;
                let field = get_field(model, old)?;
                field.name = new.clone();
            }
            Operation::DeleteField { model, name } => {
                let model = get_model(self.as_mut(), model)?;
                check_field_exists(model, name)?;
                model.fields.retain(|z| z.name != *name);
            }
            Operation::CreateIndex { model, index } => {
                let model = get_model(self.as_mut(), model)?;
                for (position, column) in index.columns.iter().enumerate() {
                    let annotation = indexes::annotation(index.name.clone(), position);
                    get_field(model, column)?.annotations.push(annotation);
                }
            }
            Operation::DeleteIndex { model, index } => {
                let name = index.sql_name(model);
                let model = get_model(self.as_mut(), model)?;
                for (position, column) in index.columns.iter().enumerate() {
                    let annotation = indexes::annotation(index.name.clone(), position);
                    let annotations = &mut get_field(model, column)?.annotations;
                    let Some(position) = annotations.iter().position(|x| *x == annotation) else {
                        return Err(ApplyOperationError::UnknownIndex {
                            index: name,
                            column: column.clone(),
                        });
                    };
                    annotations.remove(position);
                }
            }
            Operation::RawSQL { structure_safe, .. } => {
                if !*structure_safe {
                    return Err(ApplyOperationError::UnsafeRawSql);
                }
            }
        }
        Ok(())
    }

    /// Checks whether there is a model named `name`
    fn has_model(&self, name: &str) -> bool {
        self.get_model(name).is_some()
    }

    /// Retrieves a model by its `name`
    fn get_model(&self, name: &str) -> Option<&Model> {
        self.as_ref().models.iter().find(|x| x.name == *name)
    }

    /// Retrieves a model by its `name`
    fn get_model_mut(&mut self, name: &str) -> Option<&mut Model> {
        self.as_mut().models.iter_mut().find(|x| x.name == *name)
    }

    #[doc(hidden)]
    fn as_ref(&self) -> &InternalModelFormat;

    #[doc(hidden)]
    fn as_mut(&mut self) -> &mut InternalModelFormat;
}

/// Extension trait for [`Model`]
pub trait ModelExt {
    /// Checks whether there is a field named `name`
    fn has_field(&self, name: &str) -> bool {
        self.get_field(name).is_some()
    }

    /// Retrieves a field by its `name`
    fn get_field(&self, name: &str) -> Option<&Field> {
        self.as_ref().fields.iter().find(|x| x.name == *name)
    }

    /// Retrieves a field by its `name`
    fn get_field_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.as_mut().fields.iter_mut().find(|x| x.name == *name)
    }

    #[doc(hidden)]
    fn as_ref(&self) -> &Model;

    #[doc(hidden)]
    fn as_mut(&mut self) -> &mut Model;
}

/// Error produced by [`apply_operation`](InternalModelFormatExt::apply_operation)
#[derive(Debug, Error)]
pub enum ApplyOperationError {
    /// The operation referenced an unknown model
    #[error("Unknown model {model}")]
    UnknownModel { model: String },

    /// The operation referenced an unknown field
    #[error("Unknown field {field} for model {model}")]
    UnknownField { model: String, field: String },

    /// The operation deleted an index whose column doesn't take part in it
    #[error("Index {index} does not span column {column}")]
    UnknownIndex { index: String, column: String },

    /// The operation is an unsafe RawSQL
    #[error("Encountered RawSQL which is not marked as structure safe")]
    UnsafeRawSql,
}

/// Helper used by [`apply_operation`](InternalModelFormatExt::apply_operation)
fn check_model_exists(state: &InternalModelFormat, model: &str) -> Result<(), ApplyOperationError> {
    if state.has_model(model) {
        Ok(())
    } else {
        Err(ApplyOperationError::UnknownModel {
            model: model.to_string(),
        })
    }
}

/// Helper used by [`apply_operation`](InternalModelFormatExt::apply_operation)
fn check_model_not_exists(
    state: &InternalModelFormat,
    model: &str,
) -> Result<(), ApplyOperationError> {
    if state.has_model(model) {
        todo!()
    } else {
        Ok(())
    }
}

/// Helper used by [`apply_operation`](InternalModelFormatExt::apply_operation)
fn get_model<'a>(
    state: &'a mut InternalModelFormat,
    model: &str,
) -> Result<&'a mut Model, ApplyOperationError> {
    state
        .get_model_mut(model)
        .ok_or_else(|| ApplyOperationError::UnknownModel {
            model: model.to_string(),
        })
}

/// Helper used by [`apply_operation`](InternalModelFormatExt::apply_operation)
fn check_field_exists(model: &Model, field: &str) -> Result<(), ApplyOperationError> {
    if model.has_field(field) {
        Ok(())
    } else {
        Err(ApplyOperationError::UnknownField {
            model: model.name.clone(),
            field: field.to_string(),
        })
    }
}

/// Helper used by [`apply_operation`](InternalModelFormatExt::apply_operation)
fn check_field_not_exists(model: &Model, field: &str) -> Result<(), ApplyOperationError> {
    if model.has_field(field) {
        todo!()
    } else {
        Ok(())
    }
}

/// Helper used by [`apply_operation`](InternalModelFormatExt::apply_operation)
fn get_field<'a>(model: &'a mut Model, field: &str) -> Result<&'a mut Field, ApplyOperationError> {
    // Rust's lifetime checks are somewhat limited in this scenario.
    // Its reordering and dead code elimination is hopefully better. ;)
    let error = ApplyOperationError::UnknownField {
        model: model.name.clone(),
        field: field.to_string(),
    };
    model.get_field_mut(field).ok_or(error)
}

impl InternalModelFormatExt for InternalModelFormat {
    fn as_ref(&self) -> &InternalModelFormat {
        self
    }

    fn as_mut(&mut self) -> &mut InternalModelFormat {
        self
    }
}

impl ModelExt for Model {
    fn as_ref(&self) -> &Model {
        self
    }

    fn as_mut(&mut self) -> &mut Model {
        self
    }
}

#[cfg(test)]
mod test_indexes {
    use rorm_declaration::imr::{
        Annotation, DbType, Field, Index, IndexValue, InternalModelFormat,
    };
    use rorm_declaration::migration::Operation;

    use crate::utils::imr_as_state::InternalModelFormatExt;

    /// The operation creating a `user` model whose fields are annotated with `indexes`
    fn create_model(indexes: Vec<(&str, Option<IndexValue>)>) -> Operation {
        Operation::CreateModel {
            name: "user".to_string(),
            fields: indexes
                .into_iter()
                .map(|(name, index)| Field {
                    name: name.to_string(),
                    db_type: DbType::VarChar,
                    annotations: vec![Annotation::Index(index)],
                    source_defined_at: None,
                })
                .collect(),
        }
    }

    fn state(operations: Vec<Operation>) -> InternalModelFormat {
        let mut state = InternalModelFormat { models: vec![] };
        for operation in &operations {
            state.apply_operation(operation).expect("Operation applies");
        }
        state
    }

    /// Indexes a state's single model has
    fn indexes(state: &InternalModelFormat) -> Vec<Index> {
        state.models[0].indexes()
    }

    #[test]
    fn annotations_on_created_fields_are_ignored() {
        // Migrations written before index operations existed carry the annotation
        // on their fields, but they never created an index in the database.
        let state = state(vec![create_model(vec![(
            "login",
            Some(IndexValue {
                name: "login".to_string(),
                priority: None,
            }),
        )])]);

        assert_eq!(indexes(&state), vec![]);
    }

    #[test]
    fn create_index_restores_the_index() {
        let index = Index {
            name: Some("full_name".to_string()),
            columns: vec!["last_name".to_string(), "first_name".to_string()],
        };
        let state = state(vec![
            create_model(vec![("first_name", None), ("last_name", None)]),
            Operation::CreateIndex {
                model: "user".to_string(),
                index: index.clone(),
            },
        ]);

        // Especially the column order has to survive the round trip
        assert_eq!(indexes(&state), vec![index]);
    }

    #[test]
    fn delete_index_removes_the_index() {
        let index = Index {
            name: None,
            columns: vec!["login".to_string()],
        };
        let state = state(vec![
            create_model(vec![("login", None)]),
            Operation::CreateIndex {
                model: "user".to_string(),
                index: index.clone(),
            },
            Operation::DeleteIndex {
                model: "user".to_string(),
                index,
            },
        ]);

        assert_eq!(indexes(&state), vec![]);
        // The annotation may not linger on the field either
        assert_eq!(state.models[0].fields[0].annotations, vec![]);
    }

    #[test]
    fn deleting_an_index_which_was_never_created_fails() {
        let mut state = state(vec![create_model(vec![("login", None)])]);

        assert!(state
            .apply_operation(&Operation::DeleteIndex {
                model: "user".to_string(),
                index: Index {
                    name: None,
                    columns: vec!["login".to_string()],
                },
            })
            .is_err());
    }
}
