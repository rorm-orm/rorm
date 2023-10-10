//! Provide the two central traits:
//! - [`Contains`] which defines values i.e. arguments
//! - [`ConstFn`] which defines functions

pub trait Contains<T> {
    const ITEM: T;
}

pub trait ConstFn<Arg, Ret> {
    type Body<T: Contains<Arg>>: Contains<Ret>;
}

/// Converts a normal function into a [`ConstFn`].
///
/// - Due to limitations with `tt` the body requires double braces!
/// - Only accepts a very simple `fn` syntax!
#[macro_export]
macro_rules! const_fn {
    ($(#[$attr:meta])* $vis:vis fn $fun_name:ident($( $arg_name:tt : $arg_type:ty ),+) -> $ret_type:ty $body:block) => {
        $(#[$attr])*
        $vis struct $fun_name;
        const _: () = {
            impl $crate::ConstFn<($($arg_type,)+), $ret_type> for $fun_name {
                type Body<T: $crate::Contains<($($arg_type,)+)>> = Body<T>;
            }
            $vis struct Body<T: $crate::Contains<($($arg_type,)+)>>(::std::marker::PhantomData<T>);
            impl<T: $crate::Contains<($($arg_type,)+)>> $crate::Contains<$ret_type> for Body<T> {
                const ITEM: $ret_type = {
                    let ($($arg_name,)+) = T::ITEM;
                    $body
                };
            }
        };
    };
    ($(#[doc = $doc:literal])* $vis:vis fn $fun_name:ident<$generic:ident $(: $bound:path)?> ($( $arg_name:tt : $arg_type:ty ),*) -> $ret_type:ty $body:block) => {
        $(#[doc = $doc])*
        $vis struct $fun_name<$generic $(:$bound)?>(::std::marker::PhantomData<$generic>);
        const _: () = {
            impl<$generic $(:$bound)?> $crate::ConstFn<($($arg_type,)*), $ret_type> for $fun_name<$generic> {
                type Body<Arg: $crate::Contains<($($arg_type,)*)>> = Body<Arg, $generic>;
            }
            $vis struct Body<Arg: $crate::Contains<($($arg_type,)*)>, $generic $(:$bound)?>(::std::marker::PhantomData<Arg>, ::std::marker::PhantomData<$generic>);
            impl<Arg: $crate::Contains<($($arg_type,)*)>, $generic $(:$bound)?> $crate::Contains<$ret_type> for Body<Arg, $generic> {
                const ITEM: $ret_type = {
                    let ($($arg_name,)*) = Arg::ITEM;
                    $body
                };
            }
        };
    };
}

mod wrappers {
    macro_rules! impl_wrappers {
        ($( $wrapper:ident: $typ:ty ),+$(,)?) => {$(
            pub struct $wrapper<const ITEM: $typ>;
            impl<const ITEM: $typ> $crate::Contains<$typ> for $wrapper<ITEM> {
                const ITEM: $typ = ITEM;
            }
        )+};
    }
    impl_wrappers![
        I8: i8,
        I16: i16,
        I32: i32,
        I64: i64,
        Isize: isize,
        U8: u8,
        U16: u16,
        U32: u32,
        U64: u64,
        Usize: usize
    ];
}
pub use wrappers::*;

mod tuple {
    /// Implements `Contains<T>` for `C` where `T` is any tuple and `C` is a tuple
    /// whose elements implement `Contains<_>` for their corresponding element in `T`
    macro_rules! impl_tuples {
        ($( ($($C:ident : $T:ident),+) ),+$(,)?) => {$(
            impl<$($T, $C: $crate::Contains<$T>),+> $crate::Contains<($($T,)+)> for ($($C,)+) {
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
}
