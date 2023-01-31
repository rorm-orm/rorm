//! The query context holds some of a query's data which rorm-db borrows.

use std::collections::HashMap;

use ouroboros::self_referencing;

use crate::conditional::{BinaryCondition, Condition};
use crate::internal::field::{Field, FieldProxy, RawField};
use crate::internal::relation_path::{JoinAlias, Path, PathImpl, PathStep};
use crate::value::Value;
use crate::Model;

/// A [Path]'s hashable representation
type PathId = std::any::TypeId;

/// A [FieldProxy](crate::internal::field::FieldProxy)'s hashable representation
type ProxyId = std::any::TypeId;

/// Builder for a [QueryContext]
#[derive(Default, Debug)]
pub struct QueryContextBuilder {
    joins: HashMap<PathId, TempJoinData>,
    fields: HashMap<ProxyId, String>,
}
impl QueryContextBuilder {
    /// Create an empty instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Recursively add a relation path to the builder
    ///
    /// The generic parameters are the parameters defining the outer most [PathStep].
    pub(crate) fn add_relation_path<M, F, P>(&mut self)
    where
        M: Model,
        F: RawField,
        P: Path,
        PathStep<F, P>: PathImpl<F::Type>,
    {
        let new_table = PathId::of::<PathStep<F, P>>();

        if self.joins.contains_key(&new_table) {
            return;
        }
        P::add_to_join_builder(self);

        self.joins.insert(
            new_table,
            TempJoinData::Static {
                alias: PathStep::<F, P>::ALIAS,
                table_name: M::TABLE,
                fields: PathStep::<F, P>::JOIN_FIELDS,
            },
        );
    }

    /// Add a [FieldProxy] ensuring its relation path is joined and its column is on the correct table
    pub fn add_field_proxy<F: RawField, P: Path>(&mut self) {
        P::add_to_join_builder(self);
    }

    /// Consume the builder and produce a [QueryContext]
    pub fn finish(self) -> QueryContext {
        QueryContext {
            joins: self.joins.into_values().map(Join::from).collect(),
            fields: self.fields,
        }
    }
}

/// Context for creating queries.
///
/// Since rorm-db borrows all of its parameters, there has to be someone who own it.
/// This struct owns all the implicit data required to query something i.e. join and alias information.
#[derive(Debug, Default)]
pub struct QueryContext {
    joins: Vec<Join>,
    fields: HashMap<ProxyId, String>,
}
impl QueryContext {
    /// Create an empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a vector borrowing the joins in rorm_db's format which can be passed to it as slice.
    pub fn get_joins(&self) -> Vec<rorm_db::database::JoinTable> {
        self.joins.iter().map(Join::as_db_format).collect()
    }

    /// Get a field's column name joined with its table
    pub fn get_field<F: Field, P: Path>(&self) -> &str {
        self.fields
            .get(&ProxyId::of::<FieldProxy<F, P>>())
            .expect("Here be error handling?") // TODO
            .as_str()
    }
}

/// Unfinished version of [JoinData]
#[derive(Clone, Debug)]
enum TempJoinData {
    Static {
        alias: &'static str,

        table_name: &'static str,

        fields: [[&'static str; 2]; 2],
    },
    #[allow(dead_code)]
    Dynamic {
        alias: usize,

        table_name: &'static str,

        fields: [String; 2],
    },
}

#[derive(Debug)]
enum Join {
    Static {
        table_name: &'static str,
        join_alias: &'static str,
        join_condition: Condition<'static>,
    },
    #[allow(dead_code)]
    Dynamic(DynamicJoin),
}

impl Join {
    fn as_db_format(&self) -> rorm_db::database::JoinTable {
        let (table_name, join_alias, join_condition): (&str, &str, &Condition) = match self {
            Join::Dynamic(join) => (
                join.borrow_table_name(),
                join.borrow_alias(),
                join.borrow_condition(),
            ),
            Join::Static {
                table_name,
                join_alias,
                join_condition,
            } => (table_name, join_alias, join_condition),
        };
        rorm_db::database::JoinTable {
            join_type: rorm_db::join_table::JoinType::Join,
            table_name,
            join_alias,
            join_condition,
        }
    }
}
impl From<TempJoinData> for Join {
    fn from(join_data: TempJoinData) -> Self {
        match join_data {
            TempJoinData::Static {
                alias,
                table_name,
                fields: [[table_a, column_a], [table_b, column_b]],
            } => Join::Static {
                table_name,
                join_alias: alias,
                join_condition: Condition::BinaryCondition(BinaryCondition::Equals(Box::new([
                    Condition::Value(Value::Column {
                        table_name: Some(table_a),
                        column_name: column_a,
                    }),
                    Condition::Value(Value::Column {
                        table_name: Some(table_b),
                        column_name: column_b,
                    }),
                ]))),
            },
            TempJoinData::Dynamic {
                alias,
                table_name,
                fields,
            } => Join::Dynamic(
                DynamicJoinBuilder {
                    table_name,
                    alias: format!("_{alias}"),
                    fields,
                    condition_builder: |fields: &[String; 2]| {
                        Condition::BinaryCondition(BinaryCondition::Equals(Box::new([
                            Condition::Value(Value::Ident(fields[0].as_str())),
                            Condition::Value(Value::Ident(fields[1].as_str())),
                        ])))
                    },
                }
                .build(),
            ),
        }
    }
}

#[self_referencing]
#[derive(Debug)]
struct DynamicJoin {
    /// The foreign model's table name
    pub table_name: &'static str,

    /// Alias to join the table as
    pub alias: String,

    /// Fields' names required in the join condition
    pub fields: [String; 2],

    /// Condition comparing two fields for equality
    #[borrows(fields)]
    #[covariant]
    pub condition: Condition<'this>,
}

#[allow(dead_code)]
fn _fix_ouroboros(join: &DynamicJoin) {
    join.borrow_fields();
}
