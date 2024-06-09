extern crate cmake;

fn main() {
    let dst = cmake::Config::new("RE-flex")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());

    println!("cargo:include=RE-flex/include");

    cxx_build::bridge("src/lib.rs")
        .file("src/bindings.cpp")
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-IRE-flex/include")
        .compile("fuzzymatcher");

    //  c++ -Wall -Wunused -Wextra -O2 -I. -I../include
}
