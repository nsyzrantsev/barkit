#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


pub mod tre {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(feature = "approx")]
mod fuzzy;
mod compile;
mod errors;
mod flags;

#[cfg(feature = "approx")]
pub use crate::fuzzy::*;
pub use crate::compile::*;
pub use crate::errors::*;
pub use crate::flags::*;

#[derive(Debug)]
pub struct TreRegex(Option<tre::regex_t>);

impl TreRegex {
    #[must_use]
    #[inline]
    pub const unsafe fn new_from(regex: tre::regex_t) -> Self {
        Self(Some(regex))
    }

    #[must_use]
    #[inline]
    pub unsafe fn release(&mut self) -> Option<tre::regex_t> {
        let regex = self.0;
        self.0 = None;
        regex
    }

    #[must_use]
    #[inline]
    pub const fn get(&self) -> &Option<tre::regex_t> {
        &self.0
    }

    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> &mut Option<tre::regex_t> {
        &mut self.0
    }
}

impl Drop for TreRegex {
    #[inline]
    fn drop(&mut self) {
        let Some(compiled_reg) = self.get_mut() else {
            return;
        };

        unsafe {
            tre::tre_regfree(compiled_reg);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Match<'h> {
    haystack: &'h [u8],
    start: usize,
    end: usize,
}

impl<'h> Match<'h> {
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn range(&self) -> core::ops::Range<usize> {
        self.start..self.end
    }

    #[inline]
    pub fn as_bytes(&self) -> &'h [u8] {
        &self.haystack[self.range()]
    }
}