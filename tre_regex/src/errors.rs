use std::ffi::{c_char, c_int, c_uint, CString};
use std::fmt;
use std::ptr::null_mut;

use crate::{tre, TreRegex};

// Public types
pub type ErrorInt = c_int;
pub type Result<T> = std::result::Result<T, RegexError>;

/// Custom error type for errors in the binding itself.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingErrorCode(u32);

impl BindingErrorCode {
    /// Error occured with [`CString`]
    pub const CSTRING: Self = Self(1);

    /// Error occured with encoding bytes
    pub const ENCODING: Self = Self(2);

    /// An attempt was made to unwrap a vacant [`Regex`] object
    pub const REGEX_VACANT: Self = Self(3);
}

/// Type of error: `Binding` (see [`BindingErrorCode`]), or `Tre`
///
/// See the TRE documentation for more information on valid error codes for `Tre`.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Binding-specific error
    Binding(BindingErrorCode),

    /// Error from TRE
    Tre(tre::reg_errcode_t),
}

/// Error type returned in results
#[derive(Debug, PartialEq, Eq)]
pub struct RegexError {
    /// Kind of error
    pub kind: ErrorKind,

    /// Error string
    pub error: String,
}

impl RegexError {
    #[must_use]
    #[inline]
    pub fn new(kind: ErrorKind, error: &str) -> Self {
        Self {
            kind,
            error: error.to_string(),
        }
    }
}

impl std::error::Error for RegexError {}

// Quick and dirty display impl
impl fmt::Display for RegexError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (code {:?})", self.error, self.kind)
    }
}

impl TreRegex {
    #[must_use]
    pub fn regerror(&self, result: ErrorInt) -> RegexError {
        // SAFETY: compiled_reg should be valid; see safety concerns for Regex.
        let Some(compiled_reg_obj) = self.get() else {
            return RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object",
            );
        };
        let bufsize = unsafe { tre::tre_regerror(result, compiled_reg_obj, null_mut(), 0) };
        let mut errbuf = vec![0u8; bufsize];
        // SAFETY: compiled_reg should be valid; errbuf has enough room as validated above
        unsafe {
            tre::tre_regerror(
                result,
                compiled_reg_obj,
                errbuf.as_mut_ptr().cast::<c_char>(),
                bufsize,
            );
        }
        let errstr = CString::from_vec_with_nul(errbuf).map_err(|e| {
            RegexError::new(
                ErrorKind::Binding(BindingErrorCode::CSTRING),
                &format!("Could not convert error buffer to C string: {e}"),
            )
        });
        let Ok(errstr) = errstr else {
            return errstr.unwrap_err();
        };
        let errstr = errstr.to_str().map_err(|e| {
            RegexError::new(
                ErrorKind::Binding(BindingErrorCode::ENCODING),
                &format!("Could not encode error string to UTF-8: {e}"),
            )
        });
        let Ok(errstr) = errstr else {
            return errstr.unwrap_err();
        };

        // Value cannot ever be negative.
        #[allow(clippy::cast_sign_loss)]
        RegexError::new(ErrorKind::Tre(tre::reg_errcode_t(result as c_uint)), errstr)
    }
}

#[must_use]
pub fn regerror(compiled_reg: &Regex, result: ErrorInt) -> RegexError {
    compiled_reg.regerror(result)
}