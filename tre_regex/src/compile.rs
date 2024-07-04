use std::ffi::c_char;
use std::mem;

use crate::{
    err::{regerror, Result},
    flags::RegcompFlags,
    tre, TreRegex,
};

impl TreRegex {
    pub fn new(reg: &str, flags: RegcompFlags) -> Result<Self> {
        Self::new_bytes(reg.as_bytes(), flags)
    }

    pub fn new_bytes(reg: &[u8], flags: RegcompFlags) -> Result<Self> {
        let mut unwrapped_compiled_reg = mem::MaybeUninit::<tre::regex_t>::uninit();

        // SAFETY: unwrapped_compiled_reg is being initalised. reg is immutably passed and is not
        // modified by the caller. Wrapping is also impossible.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_regncomp(
                unwrapped_compiled_reg.as_mut_ptr(),
                reg.as_ptr().cast::<c_char>(),
                reg.len(),
                flags.get(),
            )
        };

        // SAFETY: tre::tre_regcomp fully initalises compiled_reg
        let compiled_reg = Self(Some(unsafe { unwrapped_compiled_reg.assume_init() }));
        if result != 0 {
            return Err(regerror(&compiled_reg, result));
        }

        Ok(compiled_reg)
    }
}

#[inline]
pub fn regcomp(reg: &str, flags: RegcompFlags) -> Result<TreRegex> {
    TreRegex::new(reg, flags)
}


#[inline]
pub fn regcomp_bytes(reg: &[u8], flags: RegcompFlags) -> Result<Regex> {
    Regex::new_bytes(reg, flags)
}