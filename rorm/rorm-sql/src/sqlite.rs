use std::ffi::{c_char, c_void, CStr, CString};

/**
This function formats a string into a SQLite quoted string.
*/
pub(crate) fn fmt(input: &str) -> String {
    let c_str = CString::new("%Q").expect("This should not fail");
    let input_str = CString::new(input).expect("Found \\0 in format string.");

    let formatted;
    unsafe {
        let formatted_str: *mut c_char =
            libsqlite3_sys::sqlite3_mprintf(c_str.as_ptr(), input_str.as_ptr());

        formatted = String::from(
            CStr::from_ptr(formatted_str)
                .to_str()
                .expect("Return string must be UTF-8"),
        );

        libsqlite3_sys::sqlite3_free(formatted_str as *mut c_void);
    }

    formatted
}
