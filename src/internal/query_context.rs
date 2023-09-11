//! The query context holds some of a query's data which rorm-db borrows.

use std::borrow::Cow;
use std::collections::HashSet;

use rorm_db::sql::conditional::{BinaryCondition, Condition};
use rorm_db::sql::value::Value;

use crate::aggregate::AggregationFunc;
use crate::internal::field::Field;
use crate::internal::relation_path::{JoinAlias, Path, PathImpl, PathStep};
use crate::Model;

/// A [Path]'s hashable representation
type PathId = std::any::TypeId;

/// Context for creating queries.
///
/// Since rorm-db borrows all of its parameters, there has to be someone who own it.
/// This struct owns all the implicit data required to query something i.e. join and alias information.
#[derive(Debug, Default)]
pub struct QueryContext {
    handled_paths: HashSet<PathId>,
    joins: Vec<Join>,
    selects: Vec<Select>,
}
impl QueryContext {
    /// Create an empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a field to select returning its index and alias
    pub fn select_field<F: Field, P: Path>(&mut self) -> (usize, String) {
        P::add_to_context(self);
        let alias = format!("{path}__{field}", path = P::ALIAS, field = F::NAME);
        self.selects.push(Select {
            table_name: Cow::Borrowed(P::ALIAS),
            column_name: F::NAME,
            select_alias: alias.clone(),
            aggregation: None,
        });
        (self.selects.len() - 1, alias)
    }

    /// Add a field to aggregate returning its index and alias
    pub fn select_aggregation<A: AggregationFunc, F: Field, P: Path>(&mut self) -> (usize, String) {
        P::add_to_context(self);
        let alias = format!(
            "{path}__{field}___{func}",
            path = P::ALIAS,
            field = F::NAME,
            func = A::NAME,
        );
        self.selects.push(Select {
            table_name: Cow::Borrowed(P::ALIAS),
            column_name: F::NAME,
            select_alias: alias.clone(),
            aggregation: Some(A::SQL),
        });
        (self.selects.len() - 1, alias)
    }

    /// Create a vector borrowing the joins in rorm_db's format which can be passed to it as slice.
    pub fn get_joins(&self) -> Vec<rorm_db::database::JoinTable> {
        self.joins.iter().map(Join::as_db_format).collect()
    }

    /// Create a vector borrowing the selects in rorm_db's format which can be passed to it as slice.
    pub fn get_selects(&self) -> Vec<rorm_db::database::ColumnSelector> {
        self.selects.iter().map(Select::as_db_format).collect()
    }

    /// Create a vector borrowing the selects only by their `column_name` to be used in `INSERT RETURNING`.
    ///
    /// This method also checks, if the context would be valid in the first place.
    pub fn get_returning(&self) -> Option<Vec<&'static str>> {
        // Disallow joins
        if !self.joins.is_empty() {
            return None;
        }

        let mut returning = Vec::with_capacity(self.selects.len());
        let table_name = self.selects.first()?.table_name.as_ref();
        for select in &self.selects {
            // Disallow aggregation
            if select.aggregation.is_some() {
                return None;
            }

            // Disallow different tables (theoretically unnecessary?)
            if select.table_name != table_name {
                return None;
            }

            returning.push(select.column_name);
        }
        Some(returning)
    }
}
impl QueryContext {
    /// **Use [`Path::add_to_context`], this method is its impl detail!**
    ///
    /// Recursively add a relation path to the builder
    ///
    /// The generic parameters are the parameters defining the outer most [PathStep].
    pub(crate) fn add_relation_path<M, F, P>(&mut self)
    where
        M: Model,
        F: Field,
        P: Path,
        PathStep<F, P>: PathImpl<F::Type>,
    {
        let new_table = PathId::of::<PathStep<F, P>>();

        if self.handled_paths.insert(new_table) {
            P::add_to_context(self);

            self.joins.push(
                TempJoinData::Static {
                    alias: PathStep::<F, P>::ALIAS,
                    table_name: M::TABLE,
                    fields: PathStep::<F, P>::JOIN_FIELDS,
                }
                .into(),
            );
        }
    }
}

#[derive(Debug, Clone)]
struct Select {
    table_name: Cow<'static, str>,
    column_name: &'static str,
    select_alias: String,
    aggregation: Option<rorm_db::sql::aggregation::SelectAggregator>,
}
impl Select {
    fn as_db_format(&self) -> rorm_db::database::ColumnSelector {
        let Self {
            table_name,
            column_name,
            select_alias,
            aggregation,
        } = self;
        rorm_db::database::ColumnSelector {
            table_name: Some(table_name.as_ref()),
            column_name,
            select_alias: Some(select_alias.as_str()),
            aggregation: *aggregation,
        }
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
}

#[derive(Debug)]
enum Join {
    Static {
        table_name: &'static str,
        join_alias: &'static str,
        join_condition: Condition<'static>,
    },
}

impl Join {
    fn as_db_format(&self) -> rorm_db::database::JoinTable {
        let (table_name, join_alias, join_condition): (&str, &str, &Condition) = match self {
            Join::Static {
                table_name,
                join_alias,
                join_condition,
            } => (table_name, join_alias, join_condition),
        };
        rorm_db::database::JoinTable {
            join_type: rorm_db::sql::join_table::JoinType::Join,
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
        }
    }
}
