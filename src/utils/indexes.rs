//! Helpers for treating indexes separately from the fields declaring them
//!
//! An index is declared through [`Annotation::Index`] on the [`Field`]s it spans,
//! but it is not part of a column's definition in the database.
//! It is created and deleted through [`Operation::CreateIndex`] and
//! [`Operation::DeleteIndex`](rorm_declaration::migration::Operation::DeleteIndex),
//! which is why everything else in the migration process ignores the annotation.
//!
//! [`Operation::CreateIndex`]: rorm_declaration::migration::Operation::CreateIndex

use rorm_declaration::imr::{Annotation, Field, Index, IndexValue, InternalModelFormat};

/// Copy of `field` without any [`Annotation::Index`]
pub fn without_indexes(field: &Field) -> Field {
    Field {
        name: field.name.clone(),
        db_type: field.db_type,
        annotations: field
            .annotations
            .iter()
            .filter(|annotation| !matches!(annotation, Annotation::Index(_)))
            .cloned()
            .collect(),
        source_defined_at: field.source_defined_at.clone(),
    }
}

/// Compares two lists of annotations ignoring any [`Annotation::Index`]
pub fn annotations_eq(lhs: &[Annotation], rhs: &[Annotation]) -> bool {
    fn columnar(annotations: &[Annotation]) -> impl Iterator<Item = &Annotation> {
        annotations
            .iter()
            .filter(|annotation| !matches!(annotation, Annotation::Index(_)))
    }
    columnar(lhs).eq(columnar(rhs))
}

/// Compares two fields ignoring any [`Annotation::Index`]
pub fn fields_eq(lhs: &Field, rhs: &Field) -> bool {
    lhs.name == rhs.name
        && lhs.db_type == rhs.db_type
        && annotations_eq(&lhs.annotations, &rhs.annotations)
}

/// Gathers every model's indexes as `(model name, index)` pairs
///
/// The pairs are ordered by their model's and index' declaration,
/// so that the same state always produces the same list.
pub fn collect(state: &InternalModelFormat) -> Vec<(String, Index)> {
    state
        .models
        .iter()
        .flat_map(|model| {
            model
                .indexes()
                .into_iter()
                .map(|index| (model.name.clone(), index))
        })
        .collect()
}

/// The [`Annotation::Index`] a column at `position` of an index named `name` carries
///
/// The position is stored as the annotation's priority,
/// so that [`Model::indexes`](rorm_declaration::imr::Model::indexes)
/// restores the index' original column order.
pub fn annotation(name: Option<String>, position: usize) -> Annotation {
    Annotation::Index(name.map(|name| IndexValue {
        name,
        priority: Some(position as i32),
    }))
}
