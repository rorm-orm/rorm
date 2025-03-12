//! The query context holds some of a query's data which rorm-db borrows.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::ops::{Deref, DerefMut};
use std::{fmt, mem};

use rorm_db::sql::join_table::JoinType;
use rorm_db::sql::ordering::Ordering;
use tracing::{trace, trace_span, Span};

use crate::conditions::{BinaryOperator, Condition, Value};
use crate::crud::selector::AggregatedColumn;
use crate::fields::proxy::FieldProxyImpl;
use crate::internal::field::Field;
use crate::internal::query_context::flat_conditions::{FlatCondition, GetConditionError};
use crate::internal::relation_path::{Path, PathField, PathId};
use crate::Model;

pub mod flat_conditions;

/// Context for creating queries.
///
/// Since rorm-db borrows all of its parameters, there has to be someone who own it.
/// This struct owns all the implicit data required to query something i.e. join and alias information.
#[derive(Debug)]
pub struct QueryContext<'v> {
    span: Span,
    base_path: Option<PathId>,
    join_aliases: HashMap<PathId, String>,
    selects: Vec<Select>,
    joins: Vec<Join>,
    order_bys: Vec<OrderBy>,
    conditions: Vec<FlatCondition>,
    values: Vec<Value<'v>>,
}
impl Default for QueryContext<'_> {
    fn default() -> Self {
        Self {
            span: Span::none(),
            base_path: Default::default(),
            join_aliases: Default::default(),
            selects: Default::default(),
            joins: Default::default(),
            order_bys: Default::default(),
            conditions: Default::default(),
            values: Default::default(),
        }
    }
}
impl<'v> QueryContext<'v> {
    /// Create an empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a field to select returning its index and alias
    pub fn select_field<F: Field, P: Path>(&mut self) -> (usize, String) {
        let path_id = P::add_to_context(self);
        let alias = format!("{}", NumberAsAZ(self.selects.len()));
        let index = self.selects.len();

        self.selects.push(Select {
            table_name: path_id,
            column_name: F::NAME,
            select_alias: alias.clone(),
            aggregation: None,
        });

        self.span.in_scope(|| {
            trace!(
                table_name = self.join_aliases.get(&path_id),
                column_name = F::NAME,
                alias,
                index,
                "QueryContext::select_field"
            )
        });

        (index, alias)
    }

    /// Add a field to aggregate returning its index and alias
    pub fn select_aggregation<I: FieldProxyImpl, R>(
        &mut self,
        column: AggregatedColumn<I, R>,
    ) -> (usize, String) {
        let path_id = I::Path::add_to_context(self);
        let alias = format!("{}", NumberAsAZ(self.selects.len()));
        let index = self.selects.len();

        self.selects.push(Select {
            table_name: path_id,
            column_name: I::Field::NAME,
            select_alias: alias.clone(),
            aggregation: Some(column.sql),
        });

        self.span.in_scope(|| {
            trace!(
                table_name = self.join_aliases.get(&path_id),
                column_name = I::Field::NAME,
                alias,
                index,
                aggregation = ?column.sql,
                "QueryContext::select_aggregation"
            )
        });

        (index, alias)
    }

    /// Adds a condition to the query context and returns its index
    /// which can be used to retrieve it `rorm-sql` representation.
    ///
    /// (Use the index with [`QueryContext::get_condition`])
    pub fn add_condition(&mut self, condition: &(impl Condition<'v> + ?Sized)) -> usize {
        condition.build(ConditionBuilder {
            context: self,
            only_accept_paths: true,
        });
        let index = self.conditions.len();
        condition.build(ConditionBuilder {
            context: self,
            only_accept_paths: false,
        });
        self.span.in_scope(|| {
            trace!(
                condition = ?self.conditions.get(index..),
                index,
                "QueryContext::add_condition"
            )
        });

        index
    }

    /// Add a field to order by
    pub fn order_by_field<F: Field, P: Path>(&mut self, ordering: Ordering) {
        let path_id = P::add_to_context(self);
        self.order_bys.push(OrderBy {
            column_name: F::NAME,
            table_name: path_id,
            ordering,
        });

        self.span.in_scope(|| {
            trace!(
                table_name = self.join_aliases.get(&path_id),
                column_name = F::NAME,
                ?ordering,
                "QueryContext::order_by_field"
            )
        });
    }

    /// Create a vector borrowing the joins in rorm_db's format which can be passed to it as slice.
    pub fn get_joins(&self) -> Vec<rorm_db::database::JoinTable> {
        self.joins
            .iter()
            .map(
                |Join {
                     table_name,
                     join_alias,
                     join_condition,
                 }| rorm_db::database::JoinTable {
                    join_type: JoinType::Join,
                    table_name,
                    join_alias: self.join_aliases.get(join_alias).unwrap(),
                    join_condition: Cow::Owned(self.get_condition(*join_condition)),
                },
            )
            .collect()
    }

    /// Create a vector borrowing the selects in rorm_db's format which can be passed to it as slice.
    pub fn get_selects(&self) -> Vec<rorm_db::database::ColumnSelector> {
        self.selects
            .iter()
            .map(
                |Select {
                     table_name,
                     column_name,
                     select_alias,
                     aggregation,
                 }| {
                    rorm_db::database::ColumnSelector {
                        table_name: Some(self.join_aliases.get(table_name).unwrap()),
                        column_name,
                        select_alias: Some(select_alias.as_str()),
                        aggregation: *aggregation,
                    }
                },
            )
            .collect()
    }

    /// Retrieves the `rorm-sql` representation of a previously added `Condition`.
    ///
    /// # Errors
    /// If the index is invalid (wasn't returned by a previous call to [`QueryContext::add_condition`])
    /// or the `Condition`'s implementation left the query context in an invalid state.
    ///
    /// Since both cases are programmers' faults,
    /// you could consider [`QueryContext::get_condition`] which simply panics.
    pub fn try_get_condition(
        &self,
        index: usize,
    ) -> Result<rorm_db::sql::conditional::Condition, GetConditionError> {
        let (head, mut tail) = self
            .conditions
            .get(index..)
            .and_then(|subslice| {
                let mut nodes = subslice.iter().copied();
                nodes.next().zip(Some(nodes))
            })
            .ok_or(GetConditionError::MissingNodes)?;
        self.get_condition_inner(head, &mut tail)
    }

    /// Retrieves the `rorm-sql` representation of a previously added `Condition`.
    ///
    /// If you want an error instead of panicking, use [`QueryContext::try_get_condition`].
    ///
    /// [`QueryContext::get_condition_opt`] might be a handy shorthand,
    /// when working with `ConditionMarker` or other sources of optional conditions
    ///
    /// # Panics
    /// If the index is invalid (wasn't returned by a previous call to [`QueryContext::add_condition`])
    /// or the `Condition`'s implementation left the query context in an invalid state.
    pub fn get_condition(&self, index: usize) -> rorm_db::sql::conditional::Condition {
        self.try_get_condition(index)
            .expect("Got invalid condition index")
    }

    /// Shorthand for calling [`Self::get_condition`] on an optional index
    pub fn get_condition_opt(
        &self,
        index: Option<usize>,
    ) -> Option<rorm_db::sql::conditional::Condition> {
        index.map(|index| self.get_condition(index))
    }

    /// Create a vector borrowing the order bys in rorm_db's format which can be passed to it as slice.
    pub fn get_order_bys(&self) -> Vec<rorm_db::sql::ordering::OrderByEntry> {
        self.order_bys
            .iter()
            .map(|order_by| rorm_db::sql::ordering::OrderByEntry {
                ordering: order_by.ordering,
                table_name: Some(self.join_aliases.get(&order_by.table_name).unwrap()),
                column_name: order_by.column_name,
            })
            .collect()
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
        let table_name = self.selects.first()?.table_name;
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

    /// Creates a temporary scope in which every path used will be implicitly appended to a base path `P`.
    ///
    /// The caller is responsible for ensuring those joins to be valid.
    /// Failing to do so can lead to weird and hard to troubleshoot bugs in rorm's internals.
    /// Similarly, the `QueryContext` may not be used until the guard returned by this method is dropped.
    ///
    /// ```
    /// # use rorm::fields::proxy::{FieldProxy, FieldProxyImpl};
    /// # use rorm::internal::query_context::QueryContext;
    /// # use rorm::internal::relation_path::{PathId, Path};
    /// # use rorm::prelude::*;
    /// # #[derive(Model)]
    /// # struct Group {
    /// #     #[rorm(id)]
    /// #     id: i64,
    /// #     #[rorm(max_length = 255)]
    /// #     name: String,
    /// # }
    /// # #[derive(Model)]
    /// # struct User {
    /// #     #[rorm(id)]
    /// #     id: i64,
    /// #     group: ForeignModel<Group>,
    /// # }
    /// # #[derive(Model)]
    /// # struct Comment {
    /// #     #[rorm(id)]
    /// #     id: i64,
    /// #     user: ForeignModel<User>,
    /// # }
    /// use rorm::crud::selector::Selector;
    ///
    /// let mut ctx = QueryContext::new();
    /// Comment.user.group.id.select(&mut ctx);
    /// {
    ///     let mut ctx = ctx.with_base_path::<(__Comment_user, Comment)>();
    ///     User.group.name.select(&mut *ctx);
    /// }
    /// let selects = ctx.get_selects();
    /// assert_eq!(selects[0].table_name, selects[1].table_name);
    /// ```
    pub fn with_base_path<'ctx, P: Path>(&'ctx mut self) -> WithBasePath<'ctx, 'v> {
        let new_base_path = P::add_to_context(self);

        let new_span = self.span.in_scope(|| {
            trace!(
                table_name = self.join_aliases.get(&new_base_path),
                "QueryContext::with_base_path"
            );
            trace_span!(
                "QueryContext::with_base_path",
                table_name = self.join_aliases.get(&new_base_path),
            )
        });

        WithBasePath {
            prev_span: mem::replace(&mut self.span, new_span),
            prev_base_path: mem::replace(&mut self.base_path, Some(new_base_path)),
            ctx: self,
        }
    }
}
/// Guard like wrapper for `QueryContext` returned by [`QueryContext::with_base_path`]
pub struct WithBasePath<'ctx, 'v> {
    prev_span: Span,
    prev_base_path: Option<PathId>,

    ctx: &'ctx mut QueryContext<'v>,
}
impl Drop for WithBasePath<'_, '_> {
    fn drop(&mut self) {
        mem::swap(&mut self.ctx.span, &mut self.prev_span);
        mem::swap(&mut self.ctx.base_path, &mut self.prev_base_path);
    }
}
impl<'v> Deref for WithBasePath<'_, 'v> {
    type Target = QueryContext<'v>;

    fn deref(&self) -> &Self::Target {
        &*self.ctx
    }
}
impl DerefMut for WithBasePath<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ctx
    }
}

impl QueryContext<'_> {
    /// **Use [`Path::add_to_context`], this method is its impl detail!**
    ///
    /// Add the origin model to the builder
    pub(crate) fn add_origin_path<M: Model>(&mut self) -> PathId {
        let path_id = M::id(self.base_path);
        if self.base_path.is_none() {
            self.join_aliases
                .entry(path_id)
                .or_insert_with(|| M::TABLE.to_string());
        }
        path_id
    }

    /// **Use [`Path::add_to_context`], this method is its impl detail!**
    ///
    /// Recursively add a relation path to the builder
    ///
    /// The generic parameters are the parameters defining the outer most [PathStep].
    pub(crate) fn add_relation_path<F, P>(&mut self) -> PathId
    where
        F: Field + PathField<<F as Field>::Type>,
        P: Path<Current = <F::ParentField as Field>::Model>,
    {
        let path_id = <P::Step<F>>::id(self.base_path);
        if !self.join_aliases.contains_key(&path_id) {
            let parent_id = P::add_to_context(self);
            let alias = format!("{}", NumberAsAZ(self.join_aliases.len()));
            self.join_aliases.insert(path_id, alias);
            self.joins.push({
                Join {
                    table_name: <<F as PathField<_>>::ChildField as Field>::Model::TABLE,
                    join_alias: path_id,
                    join_condition: self.conditions.len(),
                }
            });
            self.conditions.extend([
                FlatCondition::BinaryCondition(BinaryOperator::Equals),
                FlatCondition::Column(path_id, <F as PathField<_>>::ChildField::NAME),
                FlatCondition::Column(parent_id, <F as PathField<_>>::ParentField::NAME),
            ]);
        }
        path_id
    }
}

/// A [`&mut QueryContext`] with restricted API which is passed to [`Condition::build`]
pub struct ConditionBuilder<'r, 'v> {
    context: &'r mut QueryContext<'v>,
    only_accept_paths: bool,
}

impl<'v> ConditionBuilder<'_, 'v> {
    pub(crate) fn reborrow<'r>(&'r mut self) -> ConditionBuilder<'r, 'v> {
        ConditionBuilder::<'r, 'v> {
            context: &mut *self.context,
            only_accept_paths: self.only_accept_paths,
        }
    }

    pub(crate) fn push_condition(&mut self, condition: FlatCondition) -> usize {
        if self.only_accept_paths {
            return usize::MAX;
        }

        let index = self.context.conditions.len();
        self.context.conditions.push(condition);
        index
    }

    pub(crate) fn pop_condition(&mut self) {
        if self.only_accept_paths {
            return;
        }

        self.context.conditions.pop();
    }

    pub(crate) fn len_condition(&mut self) -> usize {
        self.context.conditions.len()
    }

    pub(crate) fn push_value(&mut self, value: Value<'v>) -> usize {
        if self.only_accept_paths {
            return usize::MAX;
        }

        let index = self.context.values.len();
        self.context.values.push(value);
        index
    }

    pub(crate) fn add_path<P: Path>(&mut self) -> PathId {
        P::add_to_context(self.context)
    }
}

#[derive(Debug, Clone)]
struct Select {
    table_name: PathId,
    column_name: &'static str,
    select_alias: String,
    aggregation: Option<rorm_db::sql::aggregation::SelectAggregator>,
}

#[derive(Debug, Clone)]
struct Join {
    table_name: &'static str,
    join_alias: PathId,
    join_condition: usize,
}

#[derive(Debug, Clone)]
struct OrderBy {
    column_name: &'static str,
    table_name: PathId,
    ordering: Ordering,
}

/// Adapter to display a number using the alphabet as digits
struct NumberAsAZ(usize);
impl fmt::Display for NumberAsAZ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const LEN: usize = 26;
        static ALPHABET: [char; LEN] = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        let mut x = self.0;
        match x {
            0..LEN => f.write_char(ALPHABET[x]),
            _ => {
                while x >= LEN {
                    f.write_char(ALPHABET[x % LEN])?;
                    x /= LEN;
                    x -= 1;
                }
                f.write_char(ALPHABET[x])
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::NumberAsAZ;

    #[test]
    fn test_number_as_az() {
        assert_eq!(NumberAsAZ(0).to_string(), "a");
        assert_eq!(NumberAsAZ(25).to_string(), "z");
        assert_eq!(NumberAsAZ(26).to_string(), "aa");
        assert_eq!(NumberAsAZ(27).to_string(), "ba");
        assert_eq!(NumberAsAZ(51).to_string(), "za");
        assert_eq!(NumberAsAZ(52).to_string(), "ab");
    }
}
