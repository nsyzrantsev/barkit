#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


pub mod tre {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(feature = "approx")]
pub mod fuzzy;
pub mod compile;
pub mod errors;
pub mod flags;

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