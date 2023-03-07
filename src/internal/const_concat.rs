//! Helper function for the `const_concat` macros.

/// Syntactic sugar for const functions
macro_rules! sugar {
    (for $i:ident in $slice:ident $then:block) => {
        let mut slices = $slice;
        while let [$i, tail @ ..] = slices {
            slices = tail;
            $then
        }
    };
}

/// Versions working on string slices
pub mod string {
    /// Count the number of bytes in a slice of strings `&[&str]`
    pub const fn count_len(strings: &[&str]) -> usize {
        let mut len = 0;
        sugar! {
            for head in strings {
                len += head.len();
            }
        }
        len
    }

    /// Concat a slice of strings populating a buffer of fixed size
    pub const fn concat_into<const N: usize>(
        mut buffer: [u8; N],
        strings: &[&str],
    ) -> (usize, [u8; N]) {
        let mut i = 0;
        sugar! {
            for string in strings {
                let bytes = string.as_bytes();
                sugar!{
                    for byte in bytes {
                        // Handle buffer overflow
                        if i == N {
                            buffer[i - 1] = DOT;
                            buffer[i - 2] = DOT;
                            buffer[i - 3] = DOT;
                            return (N, buffer);
                        }

                        buffer[i] = *byte;
                        i += 1;
                    }
                }
            }
        }
        (i, buffer)
    }

    const DOT: u8 = ".".as_bytes()[0];

    #[cfg(test)]
    mod test {
        use std::str::from_utf8;

        use super::concat_into;
        use crate::const_concat;

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
}

/// Versions working on generic slices
pub mod slice {
    /// Count the number of elements `T` in a 2d-slice `&[&[T]]`
    pub const fn count_len<T>(mut slices: &[&[T]]) -> usize {
        let mut len = 0;
        while let [head, tail @ ..] = slices {
            slices = tail;
            len += head.len();
        }
        len
    }

    /// Concat a slice of slices populating a buffer of fixed size
    pub const fn concat_into<const N: usize, T: Copy>(
        mut buffer: [T; N],
        mut slices: &[&[T]],
    ) -> (usize, [T; N]) {
        let mut i = 0;
        while let [mut slice, tail @ ..] = slices {
            slices = tail;
            while let [head, tail @ ..] = slice {
                slice = tail;

                // Catch buffer overflow
                if i == N {
                    return (N, buffer);
                }

                buffer[i] = *head;
                i += 1;
            }
        }
        (i, buffer)
    }
}
