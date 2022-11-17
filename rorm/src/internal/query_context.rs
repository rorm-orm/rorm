//! The query context holds some of a query's data which rorm-db borrows.

use std::collections::HashMap;

use ouroboros::self_referencing;

use crate::conditional::{BinaryCondition, Condition};
use crate::internal::field::{Field, FieldProxy};
use crate::internal::relation_path::{Path, PathStep};
use crate::value::Value;
use crate::{ForeignModel, Model};

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
        F: Field<Type = ForeignModel<M>>,
        P: Path,
    {
        let new_table = PathId::of::<PathStep<F, P>>();
        let existing_table = PathId::of::<P>();

        if self.joins.contains_key(&new_table) {
            return;
        }
        P::add_to_join_builder(self);

        let parent_alias = (existing_table != PathId::of::<()>()).then(|| {
            self.joins
                .get(&existing_table)
                .expect("The parent should just have been added")
                .alias
        });
        let new_alias = self.joins.len();

        self.joins.insert(
            new_table,
            TempJoinData {
                alias: new_alias,
                table_name: M::TABLE,
                fields: [
                    format!("_{new_alias}.{}", M::Primary::NAME),
                    parent_alias
                        .map(|parent_alias| format!("_{parent_alias}.{}", F::NAME))
                        .unwrap_or_else(|| F::NAME.to_string()),
                ],
            },
        );
    }

    /// Add a [FieldProxy] ensuring its relation path is joined and its column is on the correct table
    pub fn add_field_proxy<F: Field, P: Path>(&mut self, _: FieldProxy<F, P>) {
        let proxy_id = ProxyId::of::<FieldProxy<F, P>>();
        let path_id = PathId::of::<P>();

        if self.fields.contains_key(&proxy_id) {
            return;
        }

        if path_id == ProxyId::of::<()>() {
            self.fields.insert(proxy_id, F::NAME.to_string());
            return;
        }

        let alias = if let Some(join) = self.joins.get(&path_id) {
            join.alias
        } else {
            P::add_to_join_builder(self);
            self.joins
                .get(&path_id)
                .expect("The path should have just been added")
                .alias
        };

        self.fields
            .insert(proxy_id, format!("_{alias}.{}", F::NAME));
    }

    /// Consume the builder and produce a [QueryContext]
    pub fn finish(self) -> QueryContext {
        QueryContext {
            joins: self
                .joins
                .into_values()
                .map(
                    |TempJoinData {
                         table_name,
                         alias,
                         fields,
                     }| {
                        JoinDataBuilder {
                            table_name,
                            alias: format!("_{}", alias),
                            fields,
                            condition_builder: |fields: &[String; 2]| {
                                Condition::BinaryCondition(BinaryCondition::Equals(Box::new([
                                    Condition::Value(Value::Ident(fields[0].as_str())),
                                    Condition::Value(Value::Ident(fields[1].as_str())),
                                ])))
                            },
                        }
                        .build()
                    },
                )
                .collect(),
            fields: self.fields,
        }
    }
}

/// Context for creating queries.
///
/// Since rorm-db borrows all of its parameters, there has to be someone who own it.
/// This struct owns all the implicit data required to query something i.e. join and alias information.
#[derive(Debug)]
pub struct QueryContext {
    joins: Vec<JoinData>,
    fields: HashMap<ProxyId, String>,
}
impl QueryContext {
    /// Create a vector borrowing the joins in rorm_db's format which can be passed to it as slice.
    pub fn as_db_ready(&self) -> Vec<rorm_db::database::JoinTable> {
        self.joins.iter().map(JoinData::as_db_ready).collect()
    }
}

/// Unfinished version of [JoinData]
#[derive(Clone, Debug)]
struct TempJoinData {
    /// Alias id which is then converted into a unique alias name
    alias: usize,

    /// The foreign model's table name
    table_name: &'static str,

    /// The two fields compared for equality as the join condition
    fields: [String; 2],
}

/// "Owned" version of [rorm_db::JoinTable](rorm_db::database::JoinTable)
#[self_referencing]
#[derive(Debug)]
struct JoinData {
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
