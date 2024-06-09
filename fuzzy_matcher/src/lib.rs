#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("fuzzy_matcher/FuzzyMatcher/fuzzymatcher.h");
    }
}