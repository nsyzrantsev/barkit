extern crate cxx;

#[cxx::bridge(namespace = "fuzzy_matcher")]
mod ffi {
    unsafe extern "C++" {
        include!("fuzzy_matcher/cxx/fuzzy_matcher.hpp");

        type FuzzyMatcher;

        fn create_regex(pattern: &str) -> UniquePtr<CxxString>;
        fn create_fuzzy_matcher(regex: &str, threshold: i8, input: &str) -> UniquePtr<FuzzyMatcher>;
    }
}

pub use ffi::*;