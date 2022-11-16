//! Implicit join prototypes

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use ouroboros::self_referencing;
use rorm_db::conditional::BinaryCondition;
use rorm_db::conditional::Condition;
use rorm_db::value::Value;

use crate::internal::field::Field;
use crate::{ForeignModel, Model};

/// Trait to store a relation path in generics
///
/// Paths are constructed nesting [PathSteps](PathStep) and terminating the last one with `()`:
/// ```skip
/// PathStep<A, PathStep<B, PathStep<C, ()>>>
/// ```
///
/// They represent the "path" a field is access through:
/// ```skip
/// // Direct access
/// let _: FieldProxy<__Name, ())>
///     = Group::F.name;
///
/// // Access through a single relation
/// let _: FieldProxy<__Name, PathStep<__Group, ()>>
///     = User::F.group.fields().name;
///
/// // Access through two relation steps
/// let _: FieldProxy<__Name, PathStep<__Group, PathStep<__User, ()>>>
///     = Comment::F.user.fields().group.fields().name;
/// ```
pub trait Path: 'static {
    /// Add all joins required to use this path to the builder
    fn add_to_join_builder(builder: &mut JoinBuilder);
}
impl Path for () {
    fn add_to_join_builder(_builder: &mut JoinBuilder) {}
}

/// A single step in a [Path]
#[derive(Copy, Clone)]
pub struct PathStep<Head, Tail>(PhantomData<(Head, Tail)>);

impl<M, F, Tail> PathStep<F, Tail>
where
    M: Model,
    F: Field<Type = ForeignModel<M>>,
    Tail: Path,
{
    /// Create a new instance
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
impl<M, F, Tail> Path for PathStep<F, Tail>
where
    M: Model,
    F: Field<Type = ForeignModel<M>> + 'static,
    Tail: Path,
{
    fn add_to_join_builder(builder: &mut JoinBuilder) {
        let new_table = TypeId::of::<Self>();
        let existing_table = TypeId::of::<Tail>();

        if builder.joins.contains_key(&new_table) {
            return;
        }
        Tail::add_to_join_builder(builder);

        builder.joins.insert(
            new_table,
            TempJoinData {
                alias: builder.joins.len(),
                table_name: M::TABLE,
                fields: [
                    AliasableField {
                        column_name: M::Primary::NAME,
                        join_handle: new_table,
                    },
                    AliasableField {
                        column_name: F::NAME,
                        join_handle: existing_table,
                    },
                ],
            },
        );
    }
}

/// A builder passed to [Path::add_to_join_builder] to collect the required joins.
#[derive(Default, Debug)]
pub struct JoinBuilder {
    joins: HashMap<TypeId, TempJoinData>,
}
impl JoinBuilder {
    /// Create an empty instance
    pub fn new() -> Self {
        Self::default()
    }

    fn get_joined_name(&self, field: AliasableField) -> String {
        if field.join_handle == TypeId::of::<()>() {
            field.column_name.to_string()
        } else {
            format!(
                "_{}.{}",
                self.joins
                    .get(&field.join_handle)
                    .expect("Parent table should have been added!")
                    .alias,
                field.column_name,
            )
        }
    }

    /// Consume the builder a finalize its collected joins.
    pub fn finish(self) -> Joins {
        Joins {
            joins: self
                .joins
                .values()
                .map(|join| {
                    JoinDataBuilder {
                        table_name: join.table_name,
                        alias: format!("_{}", join.alias),
                        fields: join.fields.map(|field| self.get_joined_name(field)),
                        condition_builder: |fields: &[String; 2]| {
                            Condition::BinaryCondition(BinaryCondition::Equals(Box::new([
                                Condition::Value(Value::Ident(fields[0].as_str())),
                                Condition::Value(Value::Ident(fields[1].as_str())),
                            ])))
                        },
                    }
                    .build()
                })
                .collect(),
        }
    }
}

/// A list of build joins
pub struct Joins {
    joins: Vec<JoinData>,
}
impl Joins {
    /// Create a vector borrowing the joins in rorm_db's format which can be passed to it as slice.
    pub fn as_db_ready(&self) -> Vec<rorm_db::database::JoinTable> {
        self.joins.iter().map(JoinData::as_db_ready).collect()
    }
}

#[derive(Copy, Clone, Debug)]
struct TempJoinData {
    /// Alias id which is then converted into a unique alias name
    alias: usize,

    /// The foreign model's table name
    table_name: &'static str,

    /// The two fields compared for equality as the join condition
    fields: [AliasableField; 2],
}

#[derive(Copy, Clone, Debug)]
struct AliasableField {
    column_name: &'static str,
    join_handle: TypeId,
}

#[self_referencing]
struct JoinData {
    pub table_name: &'static str,
    pub alias: String,
    #[allow(dead_code)]
    pub fields: [String; 2],
    #[borrows(fields)]
    #[covariant]
    pub condition: Condition<'this>,
}
impl JoinData {
    #[allow(dead_code)]
    fn _fix_ouroboros(&self) {
        self.borrow_fields();
    }

    fn as_db_ready(&self) -> rorm_db::database::JoinTable {
        rorm_db::database::JoinTable {
            join_type: rorm_db::join_table::JoinType::Join,
            table_name: self.borrow_table_name(),
            join_alias: self.borrow_alias().as_str(),
            join_condition: self.borrow_condition(),
        }
    }
}
