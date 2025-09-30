//! A collection combines several conditions and joins them using "AND" or "OR"
//!
//! This module provides two flavours:
//! - [a dynamic one](DynamicCollection)
//! - [a static one](StaticCollection)
//!
//! Where static and dynamic mean whether the collection's size is known at compile time.
//!
//! A static collection is best created using the [`and!`](macro@crate::and)
//! and [`or!`](macro@crate::or) macros which use the same syntax as `vec!`.
//! They will automatically work around `StaticCollection`'s size restriction
//! by resorting to a `DynamicCollection` of boxes if needed.
//!
//! Both collection types can either be use with conditions or `Option` of conditions.
//! This second mode is useful when you're dealing with a lot of `Condition`s which are
//! only added conditionally. This makes a static collection effectively into a dynamic one with
//! a capacity known at compile time:
//!
//! ```rust
//! # use rorm::{and, query};
//! # use rorm_db::Database;
//! # use rorm::prelude::*;
//! # #[derive(Model)]
//! # struct User {
//! #     #[rorm(id)]
//! #     id: i64,
//! #     #[rorm(max_length = 255)]
//! #     name: String,
//! #     #[rorm(max_length = 255)]
//! #     region: String,
//! # }
//! async fn query_users(
//!     db: &Database,
//!     filter_name: Option<String>,   // Optional filters
//!     filter_region: Option<String>, // applied to query
//! ) -> Vec<User> {
//!     // The `and`'s capacity is always 2
//!     // but the actual number of conditions are only known at runtime.
//!     // Also note the "AND" of an empty set is "TRUE"
//!     // yielding the expected result of not filtering the users at all.
//!     query(db, User)
//!         .condition(and![
//!             filter_name.map(|name| User.name.equals(name)),
//!             filter_region.map(|region| User.region.equals(region)),
//!         ])
//!         .all()
//!         .await
//!         .unwrap()
//! }
//! ```

use super::Condition;
use crate::internal::query_context::flat_conditions::FlatCondition;
use crate::internal::query_context::ConditionBuilder;

/// Operator to join a collection's conditions with
#[derive(Copy, Clone, Debug)]
pub enum CollectionOperator {
    /// Join the collection's conditions with AND
    And,
    /// Join the collection's conditions with OR
    Or,
}

/// A collection of conditions with a dynamic size.
///
/// (See [module](self) for some general information)
///
/// The generic type `T` is the conditions' type, which has to be a single one.
/// (See [Disadvantage](DynamicCollection#sisadvantage))
///
/// ## Advantage:
/// - The size can change at runtime.
///
/// ## Disadvantage:
/// - All conditions have to be of the same type.
///     This can be mitigated by erasing their type using [`Condition::boxed`].
///     In this case use [`Box<dyn Condition>`] for the generic variable `T`.
#[derive(Clone)]
pub struct DynamicCollection<T> {
    /// Operator used for joining, i.e. `and` or `or`
    pub operator: CollectionOperator,

    /// Vector of conditions
    pub vector: Vec<T>,
}
impl<'a, T> DynamicCollection<T>
where
    Self: Condition<'a>,
{
    /// Create a vector of conditions joined by AND
    ///
    /// # `None`
    /// if the vector is empty
    pub fn and_checked(vector: Vec<T>) -> Option<Self> {
        Self::new_checked(CollectionOperator::And, vector)
    }

    /// Create a vector of conditions joined by AND
    ///
    /// You must check `vector` to be non-empty.
    /// We suggest using [`Self::and_checked`] instead.
    #[deprecated(note = "Use `and_checked` instead")]
    pub fn and(vector: Vec<T>) -> Self {
        Self {
            operator: CollectionOperator::And,
            vector,
        }
    }

    /// Create a vector of conditions joined by OR
    pub fn or_checked(vector: Vec<T>) -> Option<Self> {
        Self::new_checked(CollectionOperator::Or, vector)
    }

    /// Create a vector of conditions joined by OR
    ///
    /// You must check `vector` to be non-empty.
    /// We suggest using [`Self::or_checked`] instead.
    #[deprecated(note = "Use `or_checked` instead")]
    pub fn or(vector: Vec<T>) -> Self {
        Self {
            operator: CollectionOperator::Or,
            vector,
        }
    }

    /// Creates a new `DynamicCollection` after checking the `vector` to be non-empty
    fn new_checked(operator: CollectionOperator, vector: Vec<T>) -> Option<Self> {
        if vector.is_empty() {
            None
        } else {
            Some(Self { operator, vector })
        }
    }
}

impl<'a, T: Condition<'a>> Condition<'a> for DynamicCollection<T> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        builder.push_condition(FlatCondition::StartCollection(self.operator));
        for cond in self.vector.iter() {
            cond.build(builder.reborrow());
        }
        builder.push_condition(FlatCondition::EndCollection);
    }
}
impl<'a, T: Condition<'a>> Condition<'a> for DynamicCollection<Option<T>> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        builder.push_condition(FlatCondition::StartCollection(self.operator));
        let len = builder.len_condition();
        for cond in self.vector.iter().flat_map(Option::as_ref) {
            cond.build(builder.reborrow());
        }
        if len != builder.len_condition() {
            builder.push_condition(FlatCondition::EndCollection);
        } else {
            builder.pop_condition();
            super::Value::Bool(match self.operator {
                CollectionOperator::And => true,
                CollectionOperator::Or => false,
            })
            .build(builder.reborrow());
        }
    }
}

/// A collection of conditions with static size.
///
/// (See [module](self) for some general information)
///
/// The generic parameter `T` is a tuple of conditions.
/// Only tuple with 8 elements or fewer are allowed.
/// (See [Disadvantage](StaticCollection#disadvantage))
///
/// ## Advantage
/// - No type information is lost and no heap allocation required.
///
/// ## Disadvantage
/// - Due to rust's limitations, there has to be a maximum number of elements this tuple can hold.
///     Currently, it is set to 8, which is an arbitrary choice, but there has to be one.
#[derive(Copy, Clone)]
pub struct StaticCollection<T> {
    /// Operator used for joining, i.e. `and` or `or`
    pub operator: CollectionOperator,

    /// Tuple of conditions
    pub tuple: T,
}
impl<'a, T> StaticCollection<T>
where
    Self: Condition<'a>,
{
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
            fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
                builder.push_condition(FlatCondition::StartCollection(self.operator));
                let ($($generic,)+) = &self.tuple;
                $($generic.build(builder.reborrow());)+
                builder.push_condition(FlatCondition::EndCollection);
            }
        }

        #[allow(non_snake_case)] // the macro is simpler when generic variable are reused as value variables
        impl<'a, $($generic: Condition<'a>),+> Condition<'a> for StaticCollection<($(Option<$generic>,)+)> {
            fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
                builder.push_condition(FlatCondition::StartCollection(self.operator));
                let len = builder.len_condition();
                let ($($generic,)+) = &self.tuple;
                $(if let Some(cond) = $generic {
                    cond.build(builder.reborrow());
                })+
                if len != builder.len_condition() {
                    builder.push_condition(FlatCondition::EndCollection);
                } else {
                    builder.pop_condition();
                    super::Value::Bool(match self.operator {
                        CollectionOperator::And => true,
                        CollectionOperator::Or => false,
                    })
                    .build(builder.reborrow());
                }
            }
        }
    };
    ($($generic:ident),+) => {
        impl_static_collection!(recu $($generic),+);
    }
}
// Current max tuple size is 8
impl_static_collection!(H, G, F, E, D, C, B, A);

/// A common definition for the [`or!`] and [`and!`] macro
#[doc(hidden)]
#[macro_export]
macro_rules! create_collection {
    ($method:ident, $H:expr, $G:expr, $F:expr, $E:expr, $D:expr, $C:expr, $B:expr, $A:expr, $($other:expr),+ $(,)?) => {
        $crate::conditions::collections::DynamicCollection::ident(vec![
            $crate::conditions::collections::Condition::boxed($H),
            $crate::conditions::collections::Condition::boxed($G),
            $crate::conditions::collections::Condition::boxed($F),
            $crate::conditions::collections::Condition::boxed($E),
            $crate::conditions::collections::Condition::boxed($D),
            $crate::conditions::collections::Condition::boxed($C),
            $crate::conditions::collections::Condition::boxed($B),
            $crate::conditions::collections::Condition::boxed($A),
            $(
                $crate::conditions::collections::Condition::boxed($other),
            )+
        ]).unwrap_or_else(|| unreachable!("Vector should contain at least 8 conditions"))
    };
    ($method:ident, $($other:expr),+ $(,)?) => {
        $crate::conditions::collections::StaticCollection::$method(($(
            $crate::conditions::collections::ensure_condition($other),
        )+))
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

#[doc(hidden)]
pub trait IsCondition {}
impl<'c, C: Condition<'c>> IsCondition for C {}
impl<'c, C: Condition<'c>> IsCondition for Option<C> {}

#[doc(hidden)]
pub fn ensure_condition<C: IsCondition>(value: C) -> C {
    value
}
