//! This module is not considered public api.
//!
//! But since the derive macros need to have access to its content it is all public.
//! Anyway feel free to look at and maybe even use it.

pub mod array_utils;
pub mod const_concat;
pub mod field;
pub mod hmr;
pub mod patch;
pub mod query_context;
pub mod relation_path;
pub use rorm_declaration::imr;

/// Declare a type level equivalent of `Option<T>` for a concrete `T`
///
/// ## Parameters
/// - `$option:ident`: name for the new trait (an alias for `Option<T>`)
/// - `$trait:ident`: the trait to wrap (`T`)
/// - `$none:ty = ()`: unit type to symbolize none (an alias for `Option::None`)
#[doc(hidden)]
#[macro_export]
macro_rules! declare_type_option {
    ($option:ident, $trait:ident) => {
        $crate::declare_type_option!($option, $trait, ());
    };
    ($option:ident, $trait:ident, $none:ty) => {
        /// A type-level [Option],
        #[doc = concat!("ether some [", stringify!($trait) ,"] or none i.e. `", stringify!($none), "`")]
        pub trait $option {
            $crate::sealed!(trait);

            /// [Option::unwrap_or]
            ///
            #[doc = concat!("`Self`, if it is \"some\" i.e. not `", stringify!($none), "` and `Default` otherwise")]
            type UnwrapOr<Default: $trait>: $trait;
        }
        impl<T: $trait> $option for T {
            type UnwrapOr<Default: $trait> = Self;
        }
        impl $option for $none {
            type UnwrapOr<Default: $trait> = Default;
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! const_panic {
    ($strings:expr) => {
        $crate::const_panic!($strings, 1024)
    };
    ($strings:expr, $MAX_LEN:literal) => {{
        let (len, buffer) =
            $crate::internal::const_concat::string::concat_into([0u8; $MAX_LEN], $strings);
        let bytes = unsafe { ::std::slice::from_raw_parts(&buffer as *const u8, len) };
        let string = unsafe { ::std::str::from_utf8_unchecked(bytes) };
        panic!("{}", string);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! concat_columns {
    ($columns: expr) => {{
        // The number 1024 was chosen arbitrarily as it seemed big enough
        let (len, array): &'static _ =
            &$crate::internal::const_concat::slice::concat_into([""; 1024], $columns);
        if *len == 1024 {
            panic!("rorm doesn't support more than 1023 columns");
        }
        unsafe { ::std::slice::from_raw_parts::<'static, _>(array as *const _, *len) }
    }};
}

/// Concatenate string in a const block
///
/// It avoids using another const scope internally, to be able to handle generic parameters from the outer scope.
/// This means it can't use the strings' exact length to allocate the byte array.
/// As workaround there is a fixed number of array sizes from which the smallest fitting one is selected.
#[doc(hidden)]
#[macro_export]
macro_rules! const_concat {
    ($strings:expr) => {{
        let len = $crate::internal::const_concat::string::count_len($strings);
        if len <= (1 << 8) {
            $crate::const_concat!($strings, (1 << 8))
        } else if len <= (1 << 12) {
            $crate::const_concat!($strings, (1 << 12))
        } else {
            $crate::const_concat!($strings, (1 << 16))
        }
    }};
    ($strings:expr, $MAX_LEN:expr) => {{
        let (len, bytes): &'static (usize, [u8; $MAX_LEN]) =
            &$crate::internal::const_concat::string::concat_into([0u8; $MAX_LEN], $strings);

        unsafe {
            ::std::str::from_utf8_unchecked(::std::slice::from_raw_parts::<'static, u8>(
                bytes as *const u8,
                *len,
            ))
        }
    }};
}

/// Wrap a `Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result` to implement [`Display`](std::fmt::Display)
pub struct DisplayImpl<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result>(
    /// The wrapped closure
    pub F,
);
impl<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result> std::fmt::Display for DisplayImpl<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}
