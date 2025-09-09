//! `const fn` in traits
//!
//! Traits can have 3 kinds of associated objects: `fn`, `const` and `type`.
//! `fn` can't be const yet, so we have to abuse one of the other two to build a workaround.
//! `const` could be used, but their type and therefore their possible implementations have
//! to be defined by the trait author. (See [example](#alternative) below)
//! This module uses the third option (associated types).
//!
//! It provides two central traits:
//! - [`Contains`] which defines values i.e. arguments and returns
//! - [`ConstFn`] which defines functions
//!
//! # Basic Idea
//!
//! The basic idea is to have some type `add` (lower case because it should represent a function)
//! which implements `ConstFn<(i32, i32), i32>`.
//! The trait `ConstFn<(i32, i32), i32>` contains one associated constant
//! which takes a `(i32, i32)` as argument and is of type `i32`:
//! ```compile_fail
//! pub trait IdealConstFn<Args, Ret> {
//!     const RETURN<const ARGS: Args>: Ret;
//! }
//!
//! # #[allow(non_camel_case_types)]
//! pub struct add;
//! impl IdealConstFn<(i32, i32), i32> for add {
//!     const RETURN<const ARGS: (i32, i32)>: i32 = {
//!         let (x, y) = ARGS;
//!
//!         // The actual function body
//!         x + 1
//!     };
//! }
//!
//! const THREE: i32 = <add as IdealConstFn<(i32, i32), i32>>::RETURN::<(1, 2)>;
//! ```
//!
//! # Actual implementation
//!
//! However, this ideal has two problems:
//! 1. associated consts can't be generic
//! 2. const generics can't be tuples
//!
//! Solving 1. is not much of an issue, just introduce another generic as indirection:
//! ```compile_fail
//! pub trait NotSoIdealConstFn<Args, Ret> {
//!     type Call<const ARGS: Args>: Return<Ret>;
//! }
//! pub trait Return<Ret> {
//!     const RETURN: Ret;
//! }
//!
//! pub struct add;
//! impl NotSoIdealConstFn<(i32, i32), i32> for add {
//!     type Call<const ARGS: (i32, i32)> = addBody<ARGS>;
//! }
//! pub struct addBody<const: ARGS: (i32, i32)>;
//! impl<const ARGS: (i32, i32)> Return<i32> for addBody<ARGS> {
//!     const RETURN: i32 = {
//!         let (x, y) = ARGS;
//!
//!         // The actual function body
//!         x + 1
//!     };
//! }
//! ```
//!
//! Solving 2. is doable but makes calling `ConstFn`s really unergonomic.
//! Since we can't have const generics, we have to fall back to normal generics.
//! However, this means, we have to have one type for each value we might want to pass to a function:
//! ```
//! pub trait Contains<T> {
//!     const ITEM: T;
//! }
//! pub struct OneAndOne;
//! impl Contains<(i32, i32)> for OneAndOne {
//!     const ITEM: (i32, i32) = (1, 1);
//! }
//! ```
//!
//! Actually `Contains<T>` and `Returns<Ret>` look the same, so we can just use `Contains` which brings us to the actual implementation:
//! ```
//! pub trait Contains<T> {
//!     const ITEM: T;
//! }
//! pub trait ConstFn<Arg, Ret> {
//!     // Named it `Body` instead of `Call`
//!     // which makes more sense from the implementors perspective.
//!     type Body<T: Contains<Arg>>: Contains<Ret>;
//! }
//!
//! # #[allow(non_camel_case_types)]
//! pub struct add;
//! impl ConstFn<(i32, i32), i32> for add {
//!     type Body<T: Contains<(i32, i32)>> = addBody<T>;
//! }
//! # #[allow(non_camel_case_types)]
//! pub struct addBody<T>([T; 0]);
//! impl<T: Contains<(i32, i32)>> Contains<i32> for addBody<T> {
//!     const ITEM: i32 = {
//!         let (x, y) = T::ITEM;
//!
//!         // actual body
//!         x + y
//!     };
//! }
//!
//! pub struct OneAndTwo;
//! impl Contains<(i32, i32)> for OneAndTwo {
//!     const ITEM: (i32, i32) = (1, 2);
//! }
//! const THREE: i32 = <<add as ConstFn<(i32, i32), i32>>::Body::<OneAndTwo> as Contains<i32>>::ITEM;
//! ```
//!
//! # Alternative
//! As described in the introduction paragraph, you could use associated `const`s.
//!
//! Their huge advantage is readability:
//!     There is no type magic, just basic rust.
//!
//! However, they also have a huge disadvantage:
//!     The trait author has to decide on the fixed set of possible methods an implementor may choose from.
//!
//! ```rust
//! trait SomeTrait {
//!     const SOME_METHOD: SomeMethod;
//! }
//! enum SomeMethod {
//!     ImplemenationA,
//!     ImplemenationB,
//!     // ...
//! }
//! impl SomeMethod {
//!     const fn call(self) {
//!         match self {
//!             Self::ImplemenationA => {},
//!             Self::ImplemenationB => {},
//!             // ...
//!         }
//!     }
//! }
//! ```

/// Attaches a constant value of type `T`.
///
/// See [module docs](self) for more information about how and why.
pub trait Contains<T> {
    /// The value attached to / represented by `Self`
    const ITEM: T;
}

/// A `const fn(...Arg) -> Ret` which can be used in traits.
///
/// See [`const_fn!`](crate::const_fn) for an easy way to implement this.
///
/// See [module docs](self) for more information about how and why.
pub trait ConstFn<Arg, Ret> {
    /// A type which is generic over `T` and uses `T::ITEM` in its `Contains` implementation
    /// to compute the "return" value.
    type Body<T: Contains<Arg>>: Contains<Ret>;
}

/// Converts a normal function into a [`ConstFn`].
///
/// Only accepts a very simple `fn` syntax!
///
/// ```
/// # use ::rorm::const_fn;
/// const_fn! {
///     fn add(x: i32, y: i32) -> i32 {
///         x + y
///     }
/// }
/// ```
#[macro_export]
macro_rules! const_fn {
    /*
     * Start by parsing until start of generics
     */

    // Parse start non-generic function
    ($(#[$attr:meta])* $vis:vis fn $fn_name:ident($($rest:tt)+) -> $ret_type:ty $body:block) => {
        $crate::const_fn!(
            @parse_args
            ret_type: $ret_type,
            body: $body,
            rest: [$($rest)+],
            {
                attrs: [$(#[$attr])*],
                vis: $vis,
                name: $fn_name,
            }
        );
    };
    // Parse start of generic function
    ($(#[$attr:meta])* $vis:vis fn $fn_name:ident<$($rest:tt)+) => {
        $crate::const_fn!(
            @parse_generic
            [$($rest)+]
            { [] [] [(),] }
            {
                attrs: [$(#[$attr])*],
                vis: $vis,
                name: $fn_name,
            }
        );
    };

    /*
     * Parse the inside of <...>
     */

    // Parse some const generic
    (
        @parse_generic
        [const $N:ident: $Type:ty, $($rest:tt)+ ]
        { [$($type_generics:tt)*] [$($impl_generics:tt)*] [$($phantom_generics:tt)*] }
        { $($passthrough:tt)* }
    ) => {
        $crate::const_fn!(
            @parse_generic
            [$($rest)+]
            {
                [$($type_generics)* $N, ]
                [$($impl_generics)* const $N: $Type, ]
                [$($phantom_generics)*]
            }
            { $($passthrough)* }
        );
    };
    // Parse last const generic
    (
        @parse_generic
        [const $N:ident: $Type:ty >($($rest:tt)+) -> $ret_type:ty $body:block ]
        { [$($type_generics:tt)*] [$($impl_generics:tt)*] [$($phantom_generics:tt)*] }
        { $($passthrough:tt)* }
    ) => {
        $crate::const_fn!(
            @parse_args
            [$($type_generics)* $N, ],
            [$($impl_generics)* const $N: $Type, ],
            [$($phantom_generics)*],
            ret_type: $ret_type,
            body: $body,
            rest: [$($rest)+],
            {$($passthrough)*}
        );
    };
    // Parse some type generic
    (
        @parse_generic
        [$T:ident $(: $bound:path)?, $($rest:tt)+ ]
        { [$($type_generics:tt)*] [$($impl_generics:tt)*] [$($phantom_generics:tt)*] }
        { $($passthrough:tt)* }
    ) => {
        $crate::const_fn!(
            @parse_generic
            [$($rest)+]
            {
                [$($type_generics)* $T, ]
                [$($impl_generics)* $T $(: $bound)?, ]
                [$($phantom_generics)* $T,]
            }
            { $($passthrough)* }
        );
    };
    // Parse last type generic
    (
        @parse_generic
        [$T:ident $(: $bound:path)? >($($rest:tt)+) -> $ret_type:ty $body:block ]
        { [$($type_generics:tt)*] [$($impl_generics:tt)*] [$($phantom_generics:tt)*] }
        { $($passthrough:tt)* }
    ) => {
        $crate::const_fn!(
            @parse_args
            [$($type_generics)* $T, ],
            [$($impl_generics)* $T $(: $bound)?, ],
            [$($phantom_generics)* $T,],
            ret_type: $ret_type,
            body: $body,
            rest: [$($rest)+],
            {$($passthrough)*}
        );
    };
    // Parse end of generic function
    //
    // This arm will trigger if the function used a trailing comma.
    (
        @parse_generic
        [> ($($rest:tt)+) -> $ret_type:ty $body:block]
        { [$($type_generics:tt)*] [$($impl_generics:tt)*] [$($phantom_generics:tt)*] }
        { $($passthrough:tt)* }
    ) => {
        $crate::const_fn!(
            @parse_args
            [$($type_generics)+],
            [$($impl_generics)+],
            [$($phantom_generics)*],
            ret_type: $ret_type,
            body: $body,
            rest: [$($rest)+],
            {$($passthrough)*}
        );
    };

    /*
     * Finish by parsing the arguments
     */

    // Parse normal arguments
    (
        @parse_args
        $(
            [$($type_generics:tt)+],
            [$($impl_generics:tt)+],
            [$($phantom_generics:tt)*],
        )?
        ret_type: $RetType:ty,
        body: $body:block,
        rest: [$( $arg_name:tt : $ArgType:ty ),+ $(,)?],
        {
            attrs: [$($attr:tt)*],
            vis: $vis:vis,
            name: $name:ident,
        }
    ) => {
        $($attr)* $vis const fn $name $(< $($impl_generics)+ >)?($( $arg_name : $ArgType ),+) -> $RetType $body

        $crate::raw_const_fn!(
            attrs: [
                #[doc = concat!("`ConstFn` version of [`", stringify!($name), "`](fn@", stringify!($name), ")")]
                #[allow(non_camel_case_types)]
            ],
            vis: $vis,
            name: $name,
            $(
                type_generics: [$($type_generics)+],
                impl_generics: [$($impl_generics)+],
                phantom_generics: ($($phantom_generics)*),
            )?
            arg_type: ($($ArgType,)+),
            arg_param: Arg,
            ret_type: $RetType,
            body: {
                let ($($arg_name,)+) = Arg::ITEM;
                $name $( ::<$($type_generics)+> )? ($($arg_name,)*)
            },
        );
    };
    // Parse raw arguments
    (
        @parse_args
        $(
            [$($type_generics:tt)+],
            [$($impl_generics:tt)+],
            [$($phantom_generics:tt)*],
        )?
        ret_type: $RetType:ty,
        body: $body:block,
        rest: [#[raw] $arg_name:ident: $ArgType:ty],
        {
            attrs: [$($attr:tt)*],
            vis: $vis:vis,
            name: $name:ident,
        }
    ) => {
        $crate::raw_const_fn!(
            attrs: [
                #[doc = concat!("`ConstFn` version of [`", stringify!($name), "`](fn@", stringify!($name), ")")]
                #[allow(non_camel_case_types)]
            ],
            vis: $vis,
            name: $name,
            $(
                type_generics: [$($type_generics)+],
                impl_generics: [$($impl_generics)+],
                phantom_generics: ($($phantom_generics)*),
            )?
            arg_type: $ArgType,
            arg_param: $arg_name,
            ret_type: $RetType,
            body: $body,
        );
    };
}

/// Defines a [`ConstFn`]
///
/// This macros API is very direct and expects a lot of knowledge from its user.
///
/// Normal developer should try [`const_fn!`](crate::const_fn).
/// This macro is its implementation detail.
/// It is exported if `const_fn` is not flexible enough.
#[doc(hidden)]
#[macro_export]
macro_rules! raw_const_fn {
    (
        attrs: [$(#[$attr:meta])*],
        vis: $vis:vis,
        name: $Name:ident,
        $(
            type_generics: [$($TG:tt)+],
            impl_generics: [$($IG:tt)+],
            phantom_generics: $PG:ty,
        )?
        arg_type: $ArgType:ty,
        arg_param: $Arg:ident,
        ret_type: $RetType:ty,
        body: $body:block,
    ) => {
        $(#[$attr])*
        $vis struct $Name $(< $($IG)+ >)? {
            $( phantom: ::core::marker::PhantomData<$PG> )?
        }

        const _: () = {
            impl $( < $($IG)+ > )? $crate::fields::utils::const_fn::ConstFn<$ArgType, $RetType> for $Name $( < $($TG)+ > )? {
                type Body<Arg: $crate::fields::utils::const_fn::Contains<$ArgType>> = Body<(Self, Arg)>;
            }
            $vis struct Body<T>(::std::marker::PhantomData<T>);
            impl<$Arg: $crate::fields::utils::const_fn::Contains<$ArgType> $( , $($IG)+)?>
                $crate::fields::utils::const_fn::Contains<$RetType> for Body<($Name $( < $($TG)+ > )?, $Arg)>
            {
                const ITEM: $RetType = $body;
            }
        };
    };
}

/// Implements `Contains<T>` for `C` where `T` is any tuple and `C` is a tuple
/// whose elements implement `Contains<_>` for their corresponding element in `T`
macro_rules! impl_tuples {
    ($( ($($C:ident : $T:ident),+) ),+$(,)?) => {$(
        impl<$($T, $C: Contains<$T>),+> Contains<($($T,)+)> for ($($C,)+) {
            const ITEM: ($($T,)+) = ($($C::ITEM,)+);
        }
    )+};
}

impl_tuples! [
    (C1: T1),
    (C1: T1, C2: T2),
    (C1: T1, C2: T2, C3: T3),
    (C1: T1, C2: T2, C3: T3, C4: T4),
    (C1: T1, C2: T2, C3: T3, C4: T4, C5: T5),
];

#[cfg(debug_assertions)]
mod compile_tests {
    const_fn! {
        fn non_generic(_arg: ()) -> () {}
    }
    const_fn! {
        fn single_const_generic<const N: usize>(_arg: ()) -> () {}
    }
    const_fn! {
        fn single_type_generic<T>(_arg: ()) -> () {}
    }
    const_fn! {
        fn single_type_generic_with_bound<T: Copy>(_arg: ()) -> () {}
    }
    const_fn! {
        fn mixed_generics<const N: usize, T, U, const O: usize>(_arg: ()) -> () {}
    }

    const_fn! {
        fn raw_arg(#[raw] _Arg: ()) -> () {
            _Arg::ITEM
        }
    }
}
