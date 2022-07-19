use rorm_common::imr;

/// This slice is populated by the [`Model`] macro with all models.
///
/// [`Model`]: rorm_macro::Model
#[allow(non_camel_case_types)]
#[linkme::distributed_slice]
pub static MODELS: [&'static dyn ModelDefinition] = [..];

/// A ModelDefinition provides methods to do something similar to reflection on model structs.
///
/// This trait is only implemented on empty types and used as dyn objects i.e. it is a higher
/// level representation for a function table.
/// It is automatically implemented for you by the [`derive(Model)`] attribute.
///
/// [`derive(Model)`]: rorm::Model
// sync and send is required in order to store it as a static
pub trait ModelDefinition: Sync + Send {
    fn as_rorm(&self) -> Model;

    /// Build the Intermediate Model Representation
    fn as_imr(&self) -> imr::Model {
        self.as_rorm().into()
    }
}

pub struct Model {
    pub name: &'static str,
    pub fields: Vec<Field>,

    // Only forwarded to imr
    pub source: Option<imr::Source>,
}

impl From<Model> for imr::Model {
    fn from(model: Model) -> Self {
        imr::Model {
            name: model.name.to_string(),
            fields: model.fields.into_iter().map(From::from).collect(),
            source_defined_at: model.source,
        }
    }
}

pub struct Field {
    pub name: &'static str,
    pub db_type: imr::DbType,
    pub annotations: Vec<imr::Annotation>,

    // Only forwarded to imr
    pub source: Option<imr::Source>,
}

impl From<Field> for imr::Field {
    fn from(field: Field) -> Self {
        imr::Field {
            name: field.name.to_string(),
            db_type: field.db_type,
            annotations: field.annotations.into_iter().map(From::from).collect(),
            source_defined_at: field.source,
        }
    }
}
