//! A collection combines several conditions and joins them using "AND" or "OR"
//!
//! This module provides two flavours:
//! - [a dynamic one](DynamicCollection)
//! - a static one
//!
//! Where static and dynamic mean whether the collection's size is known at compile time.

use crate::internal::query_context::{QueryContext, QueryContextBuilder};
use rorm_db::sql::conditional;

use super::Condition;

/// Operator to join a collection's conditions with
#[derive(Copy, Clone)]
pub enum CollectionOperator {
    /// Join the collection's conditions with AND
    And,
    /// Join the collection's conditions with OR
    Or,
}

/// A collection of conditions with a dynamic size.
///
/// The generic type `T` is the conditions' type, which has to be a single one.
/// (See [Disadvantage](DynamicCollection#sisadvantage))
///
/// ## Advantage:
/// - The size can change at runtime.
///
/// ## Disadvantage:
/// - All conditions have to be of the same type.
///     This can be mitigated by erasing their type using [Condition::boxed].
///     In this case use [BoxedCondition<'a>](super::BoxedCondition) for the generic variable `T`.
#[derive(Clone)]
pub struct DynamicCollection<T> {
    /// Operator used for joining, i.e. `and` or `or`
    pub operator: CollectionOperator,

    /// Vector of conditions
    pub vector: Vec<T>,
}
impl<A> DynamicCollection<A> {
    /// Create a vector of conditions joined by AND
    pub fn and(vector: Vec<A>) -> Self {
        Self {
            operator: CollectionOperator::And,
            vector,
        }
    }

    /// Create a vector of conditions joined by OR
    pub fn or(vector: Vec<A>) -> Self {
        Self {
            operator: CollectionOperator::Or,
            vector,
        }
    }
}

impl<'a, A: Condition<'a>> Condition<'a> for DynamicCollection<A> {
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        for cond in self.vector.iter() {
            cond.add_to_builder(builder);
        }
    }

    fn as_sql(&self, context: &QueryContext) -> conditional::Condition {
        (match self.operator {
            CollectionOperator::And => conditional::Condition::Conjunction,
            CollectionOperator::Or => conditional::Condition::Disjunction,
        })(
            self.vector
                .iter()
                .map(|condition| condition.as_sql(context))
                .collect(),
        )
    }
}

/// A collection of conditions with static size.
///
/// The generic parameter `T` is a tuple of conditions.
/// Only tuple with 8 elements or less are allowed.
/// (See [Disadvantage](StaticCollection#disadvantage))
///
/// ## Advantage
/// - No type information is lost and no heap allocation required.
///
/// ## Disadvantage
/// - Due to rust's limitations, there has to be a maximum number of elements this tuple can hold.
///     Currently it is set to 8, which is an arbitrary choice, but there has to be one.
#[derive(Copy, Clone)]
pub struct StaticCollection<T> {
    /// Operator used for joining, i.e. `and` or `or`
    pub operator: CollectionOperator,

    /// Tuple of conditions
    pub tuple: T,
}
impl<T> StaticCollection<T> {
    /// Create a tuple of conditions joined by AND
    pub fn and(tuple: T) -> Self {
        Self {
            operator: CollectionOperator::And,
            tuple,
        }
    }

    /// Create a tuple of conditions joined by OR
    pub fn or(tuple: T) -> Self {
        Self {
            operator: CollectionOperator::Or,
            tuple,
        }
    }
}

/// Implement [StaticCollection] for up to a fixed tuple size
macro_rules! impl_static_collection {
    (recu $head:ident, $($tail:ident),+) => {
        impl_static_collection!(impl $head, $($tail),+);
        impl_static_collection!(recu $($tail),+);
    };
    (recu $generic:ident) => {
        impl_static_collection!(impl $generic);
    };
    (impl $($generic:ident),+) => {
        #[allow(non_snake_case)] // the macro is simpler when generic variable are reused as value variables
        impl<'a, $($generic: Condition<'a>),+> Condition<'a> for StaticCollection<($($generic,)+)> {
            fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
                let ($($generic,)+) = &self.tuple;
                $($generic.add_to_builder(builder);)+
            }

            fn as_sql(&self, context: &QueryContext) -> conditional::Condition {
                let ($($generic,)+) = &self.tuple;
                (match self.operator {
                    CollectionOperator::And => conditional::Condition::Conjunction,
                    CollectionOperator::Or => conditional::Condition::Disjunction,
                })(vec![
                    $($generic.as_sql(context),)+
                ])
            }
        }
    };
    ($($generic:ident),+) => {
        impl_static_collection!(recu $($generic),+);
    }
}
// Current max tuple size is 8
impl_static_collection!(H, G, F, E, D, C, B, A);

/// A common definition for the or! and and! macro
#[doc(hidden)]
#[macro_export]
macro_rules! create_collection {
    ($method:ident, $H:expr, $G:expr, $F:expr, $E:expr, $D:expr, $C:expr, $B:expr, $A:expr, $($other:expr),+ $(,)?) => {
        $crate::conditions::collections::DynamicCollection::ident(vec![
            $H.boxed(),
            $G.boxed(),
            $F.boxed(),
            $E.boxed(),
            $D.boxed(),
            $C.boxed(),
            $B.boxed(),
            $A.boxed(),
            $($other.boxed()),+
        ])
    };
    ($method:ident, $($other:expr),+ $(,)?) => {
        $crate::conditions::collections::StaticCollection::$method(($($other,)+))
    }
}

/// Combine several [Conditions](Condition) into a single one using "OR".
///
/// It takes a variadic number of conditions (min 1) and places them in a [collection](self).
/// Which one depends on the number of arguments.
#[macro_export]
macro_rules! or {
    ($($condition:expr),+ $(,)?) => {
        $crate::create_collection!(or, $($condition),+);
    };
}

/// Combine several [Conditions](Condition) into a single one using "AND".
///
/// It takes a variadic number of conditions (min 1) and places them in a [collection](self).
/// Which one depends on the number of arguments.
#[macro_export]
macro_rules! and {
    ($($condition:expr),+ $(,)?) => {
        $crate::create_collection!(and, $($condition),+);
    };
}
