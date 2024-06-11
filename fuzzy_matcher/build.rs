extern crate cmake;

fn main() {
    let dst = cmake::Config::new("RE-flex")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());

    println!("cargo:include=RE-flex/include");

    cxx_build::bridge("src/lib.rs")
        .file("cxx/fuzzy_matcher.cpp")
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-IRE-flex/include")
        .compile("fuzzy_matcher");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cxx/fuzzy_matcher.hpp");
    println!("cargo:rerun-if-changed=cxx/fuzzy_matcher.cpp");
    println!("cargo:rerun-if-changed=FuzzyMatcher/fuzzymatcher.h");
}
