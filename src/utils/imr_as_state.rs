//! Extension trait to treat [`imr`] types as queryable and updatable state

use rorm_declaration::imr::{Field, InternalModelFormat, Model};
use rorm_declaration::migration::Operation;
use thiserror::Error;

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
                    fields: fields.clone(),
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
                model.fields.push(field.clone());
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
