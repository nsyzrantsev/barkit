extern crate cxx;

#[cxx::bridge(namespace = "fuzzy_matcher")]
mod ffi {
    unsafe extern "C++" {
        include!("fuzzy_matcher/cxx/fuzzy_matcher.hpp");

        type FuzzyMatcher;
        type Pattern;

        fn create_regex(pattern: &str) -> UniquePtr<Pattern>;
        fn create_fuzzy_matcher(regex: UniquePtr<Pattern>, max_errors: i8, input: &str) -> UniquePtr<FuzzyMatcher>;
        fn matches(matcher: UniquePtr<FuzzyMatcher>) -> bool;
        fn edits(matcher: UniquePtr<FuzzyMatcher>) -> u8;
        fn distance(matcher: UniquePtr<FuzzyMatcher>) -> u16;
    }
}

pub use ffi::*;
