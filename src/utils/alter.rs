//! Decides how a field can be changed in place, if at all
//!
//! A migration file is dialect agnostic: the same file is applied to whichever
//! backend a deployment uses, and `make-migrations` doesn't know which one that
//! will be. So the rule the operations returned here have to obey is:
//!
//! > They may only be emitted when, **for every supported dialect**, applying
//! > them leaves the database in the same observable state that an
//! > [`Operation::DeleteField`] followed by an [`Operation::CreateField`] would
//! > have left it in - just with the column's data intact.
//!
//! Concretely that means:
//! - Sqlite renders [`DbType::VarChar`] and [`DbType::Text`] as `TEXT`, every
//!   integer as `INTEGER` and both floats as `REAL`, so a change within one of
//!   those groups is a genuine no-op there and a plain
//!   `ALTER COLUMN ... TYPE` in postgres.
//! - Sqlite ignores [`Annotation::MaxLength`] entirely - it has no `varchar`
//!   and never enforces a string's length - so changing it is a no-op there
//!   and a check constraint in postgres.
//! - Everything else would need sqlite's `ALTER COLUMN`, which doesn't exist.
//!   Emitting operations for a `NotNull` change would silently do nothing on a
//!   sqlite deployment, leaving the migration state claiming a constraint the
//!   database doesn't have. That undetectable schema drift is worse than the
//!   drop and recreate it would replace, which is why those changes keep it.
//!
//! The operations are a delta rather than a target state, so that applying a
//! migration needs no knowledge of the column's current definition. Deciding
//! what the delta is happens here, where both sides are known.

use rorm_declaration::imr::{Annotation, DbType, Field};
use rorm_declaration::migration::Operation;

use crate::utils::indexes;

/// The operations changing `old` into `new` in place
///
/// It is `None` if the change can't be applied in place in every dialect, in
/// which case the field has to be dropped and recreated - losing its data.
///
/// An empty `Vec` means the two are already equal.
/// Any [`Annotation::Index`] on either field is ignored: an index is not part
/// of a column's definition and has its own operations.
pub fn operations(model: &str, old: &Field, new: &Field) -> Option<Vec<Operation>> {
    if old.name != new.name || !is_alterable_type(old, new) || !is_alterable_annotations(old, new) {
        return None;
    }

    let mut operations = Vec::new();

    if old.db_type != new.db_type {
        operations.push(Operation::SetFieldType {
            model: model.to_string(),
            name: new.name.clone(),
            db_type: new.db_type,
        });
    }

    // Only a `Text` column's maximum length is a constraint. A
    // `character varying` carries it in its type, so it neither has one to drop
    // nor can be given one.
    let old_max_length = max_length(old).filter(|_| old.db_type == DbType::Text);
    let new_max_length = max_length(new).filter(|_| new.db_type == DbType::Text);

    if old_max_length != new_max_length {
        // A constraint can't be redefined, only replaced
        if old_max_length.is_some() {
            operations.push(Operation::DropFieldMaxLength {
                model: model.to_string(),
                name: new.name.clone(),
            });
        }
        if let Some(max_length) = new_max_length {
            operations.push(Operation::SetFieldMaxLength {
                model: model.to_string(),
                name: new.name.clone(),
                max_length,
            });
        }
    }

    Some(operations)
}

/// Can a column of `old`'s type be changed to `new`'s?
fn is_alterable_type(old: &Field, new: &Field) -> bool {
    if old.db_type == new.db_type {
        return true;
    }

    // Postgres implements an auto incrementing column as `smallserial`,
    // `serial` or `bigserial`, which is shorthand for a column with its own
    // sequence. `ALTER COLUMN ... TYPE` widens the column but leaves the
    // sequence at its old type, to overflow at its old maximum - which is
    // exactly the bug someone widening a primary key is trying to escape.
    if new
        .annotations
        .iter()
        .any(|annotation| matches!(annotation, Annotation::AutoIncrement))
    {
        return false;
    }

    #[allow(deprecated)]
    {
        use DbType::*;
        matches!(
            (old.db_type, new.db_type),
            // Both are `TEXT` in sqlite and binary coercible in postgres.
            // `VarChar` is only a source: nothing produces it any more, and its
            // maximum length is part of its type, so it can't be set by an
            // `ALTER COLUMN ... TYPE`.
            (VarChar, Text)
            // Widening only: narrowing would truncate values in postgres while
            // silently doing nothing in sqlite.
            | (Int8, Int16 | Int32 | Int64)
            | (Int16, Int32 | Int64)
            | (Int32, Int64)
            | (Float, Double)
        )
    }
}

/// Can `old`'s annotations be changed to `new`'s?
///
/// [`Annotation::MaxLength`] is the only alterable annotation, and only on a
/// string column, where it is either part of the type ([`DbType::VarChar`]) or
/// a check constraint ([`DbType::Text`]). On any other type it isn't rendered
/// at all, so a constraint would apply `length()` to a column which has no
/// length.
fn is_alterable_annotations(old: &Field, new: &Field) -> bool {
    #[allow(deprecated)]
    let string_column = matches!(new.db_type, DbType::VarChar | DbType::Text);

    indexes::annotations_eq(&fixed(old), &fixed(new))
        && (max_length(old) == max_length(new) || string_column)
}

/// The annotations of `field` which can only be changed by recreating it
fn fixed(field: &Field) -> Vec<Annotation> {
    field
        .annotations
        .iter()
        .filter(|annotation| !matches!(annotation, Annotation::MaxLength(_)))
        .cloned()
        .collect()
}

/// The value of `field`'s [`Annotation::MaxLength`], if it has one
fn max_length(field: &Field) -> Option<i32> {
    field
        .annotations
        .iter()
        .find_map(|annotation| match annotation {
            Annotation::MaxLength(max_length) => Some(*max_length),
            _ => None,
        })
}

#[cfg(test)]
mod test {
    use rorm_declaration::imr::{Annotation, DbType, DefaultValue, Field, ForeignKey, IndexValue};
    use rorm_declaration::migration::Operation;

    use crate::utils::alter::operations;

    fn field(db_type: DbType, annotations: Vec<Annotation>) -> Field {
        Field {
            name: "login".to_string(),
            db_type,
            annotations,
            source_defined_at: None,
        }
    }

    /// The operations changing a `login` column, described by their kind and
    /// payload so the assertions stay readable
    fn delta(old: &Field, new: &Field) -> Option<Vec<String>> {
        Some(
            operations("user", old, new)?
                .into_iter()
                .map(|operation| match operation {
                    Operation::SetFieldType { model, name, db_type } => {
                        assert_eq!((model.as_str(), name.as_str()), ("user", "login"));
                        format!("type={db_type:?}")
                    }
                    Operation::SetFieldMaxLength { model, name, max_length } => {
                        assert_eq!((model.as_str(), name.as_str()), ("user", "login"));
                        format!("set={max_length}")
                    }
                    Operation::DropFieldMaxLength { model, name } => {
                        assert_eq!((model.as_str(), name.as_str()), ("user", "login"));
                        "drop".to_string()
                    }
                    other => panic!("Unexpected operation {other:?}"),
                })
                .collect(),
        )
    }

    fn types(old: DbType, new: DbType) -> Option<Vec<String>> {
        delta(&field(old, vec![]), &field(new, vec![]))
    }

    #[allow(deprecated)]
    const VARCHAR: DbType = DbType::VarChar;

    /// The change every existing deployment gets: the type moves and the
    /// maximum length becomes a constraint. A `character varying` never had one,
    /// so nothing is dropped.
    #[test]
    fn varchar_to_text_sets_the_type_and_adds_the_constraint() {
        assert_eq!(
            delta(
                &field(VARCHAR, vec![Annotation::MaxLength(255), Annotation::NotNull]),
                &field(
                    DbType::Text,
                    vec![Annotation::MaxLength(255), Annotation::NotNull]
                ),
            ),
            Some(vec!["type=Text".to_string(), "set=255".to_string()])
        );
    }

    /// Losing the annotation on the way leaves only the type change - there is
    /// still no constraint to drop.
    #[test]
    fn varchar_to_text_without_a_max_length_drops_nothing() {
        assert_eq!(
            delta(
                &field(VARCHAR, vec![Annotation::MaxLength(255)]),
                &field(DbType::Text, vec![]),
            ),
            Some(vec!["type=Text".to_string()])
        );
    }

    /// A constraint can't be redefined, only replaced
    #[test]
    fn changing_a_max_length_replaces_the_constraint() {
        assert_eq!(
            delta(
                &field(DbType::Text, vec![Annotation::MaxLength(255)]),
                &field(DbType::Text, vec![Annotation::MaxLength(300)]),
            ),
            Some(vec!["drop".to_string(), "set=300".to_string()])
        );
    }

    #[test]
    fn removing_a_max_length_only_drops() {
        assert_eq!(
            delta(
                &field(DbType::Text, vec![Annotation::MaxLength(255)]),
                &field(DbType::Text, vec![]),
            ),
            Some(vec!["drop".to_string()])
        );
    }

    #[test]
    fn adding_a_max_length_only_sets() {
        assert_eq!(
            delta(
                &field(DbType::Text, vec![]),
                &field(DbType::Text, vec![Annotation::MaxLength(255)]),
            ),
            Some(vec!["set=255".to_string()])
        );
    }

    /// An unchanged field needs no statement, but is still alterable
    #[test]
    fn an_unchanged_field_has_an_empty_delta() {
        let annotations = vec![Annotation::MaxLength(255), Annotation::NotNull];
        assert_eq!(
            delta(
                &field(DbType::Text, annotations.clone()),
                &field(DbType::Text, annotations)
            ),
            Some(vec![])
        );
    }

    #[test]
    fn integers_widen_but_do_not_narrow() {
        assert_eq!(
            types(DbType::Int32, DbType::Int64),
            Some(vec!["type=Int64".to_string()])
        );
        assert_eq!(
            types(DbType::Int8, DbType::Int16),
            Some(vec!["type=Int16".to_string()])
        );
        assert_eq!(types(DbType::Int64, DbType::Int32), None);
    }

    #[test]
    fn float_widens_to_double_but_not_back() {
        assert_eq!(
            types(DbType::Float, DbType::Double),
            Some(vec!["type=Double".to_string()])
        );
        assert_eq!(types(DbType::Double, DbType::Float), None);
    }

    /// Nothing produces a `character varying` any more, and its maximum length
    /// is part of its type, so it is only ever a source.
    #[test]
    fn text_does_not_convert_back_to_varchar() {
        assert_eq!(types(DbType::Text, VARCHAR), None);
    }

    /// Sqlite renders them all as `TEXT`, but postgres' `timestamptz` ->
    /// `timestamp` would silently discard the timezone.
    #[test]
    fn date_and_time_types_do_not_convert() {
        assert_eq!(types(DbType::DateTime, DbType::Timestamp), None);
        assert_eq!(types(DbType::Text, DbType::DateTime), None);
    }

    #[test]
    fn unrelated_types_do_not_convert() {
        assert_eq!(types(DbType::Text, DbType::Int64), None);
        assert_eq!(types(DbType::Binary, DbType::Text), None);
        assert_eq!(types(DbType::Text, DbType::Choices), None);
    }

    /// `ALTER COLUMN ... TYPE` would leave the sequence behind a `serial`
    /// column at its old type, to overflow at its old maximum.
    #[test]
    fn widening_an_auto_increment_column_is_not_alterable() {
        assert_eq!(
            delta(
                &field(DbType::Int32, vec![Annotation::AutoIncrement]),
                &field(DbType::Int64, vec![Annotation::AutoIncrement]),
            ),
            None
        );
    }

    /// Only its type is off limits - the column itself may still be altered
    #[test]
    fn an_auto_increment_column_keeping_its_type_is_alterable() {
        assert_eq!(
            delta(
                &field(DbType::Int64, vec![Annotation::AutoIncrement]),
                &field(DbType::Int64, vec![Annotation::AutoIncrement]),
            ),
            Some(vec![])
        );
    }

    /// `length()` has nothing to be applied to on a `bigint`, so a stray
    /// annotation may not become a constraint.
    #[test]
    fn max_length_is_not_alterable_on_a_non_string_column() {
        assert_eq!(
            delta(
                &field(DbType::Int64, vec![Annotation::MaxLength(5)]),
                &field(DbType::Int64, vec![Annotation::MaxLength(9)]),
            ),
            None
        );
    }

    /// Sqlite can't change any of these, so altering would silently do nothing
    /// there while postgres applied it - schema drift.
    #[test]
    fn the_other_annotations_are_not_alterable() {
        let cases = [
            Annotation::NotNull,
            Annotation::Unique,
            Annotation::PrimaryKey,
            Annotation::AutoIncrement,
            Annotation::AutoCreateTime,
            Annotation::AutoUpdateTime,
            Annotation::DefaultValue(DefaultValue::Integer(1)),
            Annotation::Choices(vec!["a".to_string()]),
            Annotation::ForeignKey(ForeignKey::default()),
        ];
        for annotation in cases {
            assert_eq!(
                delta(
                    &field(DbType::Text, vec![]),
                    &field(DbType::Text, vec![annotation.clone()])
                ),
                None,
                "adding {annotation:?}"
            );
            assert_eq!(
                delta(
                    &field(DbType::Text, vec![annotation.clone()]),
                    &field(DbType::Text, vec![])
                ),
                None,
                "removing {annotation:?}"
            );
        }
    }

    /// An index is created and deleted through its own operations,
    /// so it may not influence whether a column can be altered.
    #[test]
    fn index_annotations_are_ignored() {
        let index = Annotation::Index(Some(IndexValue {
            name: "login".to_string(),
            priority: Some(0),
        }));
        assert_eq!(
            delta(
                &field(VARCHAR, vec![Annotation::MaxLength(255), index]),
                &field(DbType::Text, vec![Annotation::MaxLength(255)]),
            ),
            Some(vec!["type=Text".to_string(), "set=255".to_string()])
        );
    }

    /// A column's annotations are a set, so listing them in another order is
    /// not a change.
    #[test]
    fn reordered_annotations_are_not_a_change() {
        assert_eq!(
            delta(
                &field(DbType::Text, vec![Annotation::NotNull, Annotation::Unique]),
                &field(DbType::Text, vec![Annotation::Unique, Annotation::NotNull]),
            ),
            Some(vec![])
        );
    }

    #[test]
    fn a_renamed_field_is_not_alterable() {
        let mut new = field(DbType::Text, vec![]);
        new.name = "username".to_string();
        assert_eq!(delta(&field(DbType::Text, vec![]), &new), None);
    }
}
