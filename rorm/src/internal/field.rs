//! defines the [Field] struct which is stored under `Model::F`.

use std::marker::PhantomData;

use rorm_declaration::hmr::{annotations, Source};
use rorm_declaration::{hmr, imr};

use crate::annotation_builder;
use crate::internal::as_db_type::AsDbType;

/// All relevant information about a model's field
///
/// ## Generic Parameters
/// - `T` is the rust data type stored in this field
/// - `D` is the data type as which this field is stored in the db
///     (Since this can depend on some attributes, namely `#[rorm(choices)]`, it needs to be stored separately from `T`)
/// - `M` is the [Model](crate::model::Model) this field belongs to.
/// - `A` is the concrete type of the generic [annotation builder](annotation_builder::Annotations).
#[derive(Copy, Clone)]
pub struct Field<T, D, M, A> {
    /// This field's position in the model.
    pub index: usize,

    /// Name of this field
    pub name: &'static str,

    /// List of annotations this field has set
    pub annotations: A,

    /// Optional definition of the location of field in the source code
    pub source: Option<Source>,

    #[doc(hidden)]
    pub _phantom: PhantomData<(T, D, M)>,
}

impl<T: AsDbType, D, M, A> Field<T, D, M, A> {
    /// Reexport [`AsDbType::from_primitive`]
    ///
    /// This method makes macros' syntax slightly cleaner
    pub fn convert_primitive(&self, primitive: T::Primitive) -> T {
        T::from_primitive(primitive)
    }

    /// Has the field the NotNull annotation in the db?
    ///
    /// Used in compile checks.
    pub const fn is_not_null(&self) -> bool {
        !T::IS_NULLABLE
    }
}

impl<
        T: AsDbType,
        D,
        M,
        A: annotation_builder::AnnotationsDescriptor + annotation_builder::ImplicitNotNull,
    > Field<T, D, M, A>
{
    /// This method is called at compile time by the derive macro to perform cross annotation checks.
    pub const fn check_annotations(&self) {
        let mut annotations: rorm_declaration::lints::Annotations = A::FOOTPRINT;
        annotations.not_null = !T::IS_NULLABLE && !A::IMPLICIT_NOT_NULL;
        annotations.foreign_key = T::IS_FOREIGN.is_some();
        if let Err(err) = annotations.check() {
            panic!("{}", err);
        }
    }
}

impl<
        T: AsDbType,
        D: hmr::db_type::DbType,
        M,
        A: annotations::AsImr<Imr = Vec<imr::Annotation>> + annotation_builder::ImplicitNotNull,
    > From<&'_ Field<T, D, M, A>> for imr::Field
{
    fn from(field: &'_ Field<T, D, M, A>) -> Self {
        let mut annotations = field.annotations.as_imr();
        if !T::IS_NULLABLE && !A::IMPLICIT_NOT_NULL {
            annotations.push(imr::Annotation::NotNull);
        }
        if let Some((table, column)) = T::IS_FOREIGN {
            annotations.push(imr::Annotation::ForeignKey(imr::ForeignKey {
                table_name: table.to_string(),
                column_name: column.to_string(),
            }))
        }
        imr::Field {
            name: field.name.to_string(),
            db_type: D::IMR,
            annotations,
            source_defined_at: field.source.map(Into::into),
        }
    }
}
