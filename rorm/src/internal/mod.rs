//! This module is not considered public api.
//!
//! But since the derive macros need to have access to its content it is all public.
//! Anyway feel free to look at and maybe even use it.

pub mod as_db_type;
pub mod field;

#[doc(hidden)]
#[macro_export]
macro_rules! const_panic {
    ($strings:expr) => {
        const_panic!($strings, 1024);
    };
    ($strings:expr, $MAX_LEN:literal) => {{
        let (len, buffer) = $crate::internal::concat_into([0u8; $MAX_LEN], $strings);
        let bytes = unsafe { ::std::slice::from_raw_parts(&buffer as *const u8, len) };
        let string = unsafe { ::std::str::from_utf8_unchecked(bytes) };
        panic!("{}", string);
    }};
}

#[doc(hidden)]
pub const fn concat_into<const N: usize>(
    mut buffer: [u8; N],
    mut strings: &[&'static str],
) -> (usize, [u8; N]) {
    let mut i = 0;
    while let [head, tail @ ..] = strings {
        strings = tail;
        let mut bytes = head.as_bytes();
        while let [head, tail @ ..] = bytes {
            bytes = tail;
            buffer[i] = *head;
            i += 1;

            // Handle buffer overflow
            if i == N {
                buffer[i - 1] = DOT;
                buffer[i - 2] = DOT;
                buffer[i - 3] = DOT;
                return (N, buffer);
            }
        }
    }
    (i, buffer)
}

const DOT: u8 = ".".as_bytes()[0];
