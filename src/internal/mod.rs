//! This module is not considered public api.
//!
//! But since the derive macros need to have access to its content it is all public.
//! Anyway feel free to look at and maybe even use it.

pub mod field;
pub mod hmr;
pub mod query_context;
pub mod relation_path;

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
            $crate::sealed!();

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
        $crate::const_panic!($strings, 1024);
    };
    ($strings:expr, $MAX_LEN:literal) => {{
        let (len, buffer) = $crate::internal::concat_into([0u8; $MAX_LEN], $strings);
        let bytes = unsafe { ::std::slice::from_raw_parts(&buffer as *const u8, len) };
        let string = unsafe { ::std::str::from_utf8_unchecked(bytes) };
        panic!("{}", string);
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
        let len = $crate::internal::count_len($strings);
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
            &$crate::internal::concat_into([0u8; $MAX_LEN], $strings);

        unsafe {
            ::std::str::from_utf8_unchecked(::std::slice::from_raw_parts::<'static, u8>(
                bytes as *const u8,
                *len,
            ))
        }
    }};
}

#[doc(hidden)]
pub const fn count_len(mut strings: &[&str]) -> usize {
    let mut len = 0;
    while let [head, tail @ ..] = strings {
        strings = tail;
        len += head.len();
    }
    len
}

#[doc(hidden)]
pub const fn concat_into<const N: usize>(
    mut buffer: [u8; N],
    mut strings: &[&str],
) -> (usize, [u8; N]) {
    let mut i = 0;
    while let [head, tail @ ..] = strings {
        strings = tail;
        let mut bytes = head.as_bytes();
        while let [head, tail @ ..] = bytes {
            // Handle buffer overflow
            if i == N {
                buffer[i - 1] = DOT;
                buffer[i - 2] = DOT;
                buffer[i - 3] = DOT;
                return (N, buffer);
            }

            bytes = tail;
            buffer[i] = *head;
            i += 1;
        }
    }
    (i, buffer)
}

const DOT: u8 = ".".as_bytes()[0];

#[cfg(test)]
mod test {
    use crate::internal::concat_into;
    use std::str::from_utf8;

    #[test]
    fn compare_with_std_concat() {
        const STD1: &str = concat!("a", "a");
        const RORM1: &str = const_concat!(&["a", "a"]);
        assert_eq!(STD1, RORM1);

        const STD2: &str = concat!("abc", "abc");
        const RORM2: &str = const_concat!(&["abc", "abc"]);
        assert_eq!(STD2, RORM2);

        const STD3: &str = concat!("a", "abc", "abcdef", "abcdefghi");
        const RORM3: &str = const_concat!(&["a", "abc", "abcdef", "abcdefghi"]);
        assert_eq!(STD3, RORM3);
    }

    #[test]
    fn test_concat_into() {
        // Matching buffer
        let (written, bytes) = concat_into([0u8; 8], &["123", "45", "678"]);
        assert_eq!(Ok("12345678"), from_utf8(&bytes[..written]));
        assert_eq!(8, written);

        // Too small buffer
        let (written, bytes) = concat_into([0u8; 8], &["123", "45", "678", "9"]);
        assert_eq!(Ok("12345..."), from_utf8(&bytes[..written]));
        assert_eq!(8, written);

        // Too big buffer
        let (written, bytes) = concat_into([0u8; 8], &["123", "45", "6"]);
        assert_eq!(Ok("123456"), from_utf8(&bytes[..written]));
        assert_eq!(6, written);
    }
}
