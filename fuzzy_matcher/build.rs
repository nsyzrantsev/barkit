extern crate cmake;
use std::process::Command;
use std::env;

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");

    env::set_current_dir("RE-flex").expect("Failed to change directory to RE-flex");

    let status = Command::new("bash")
        .arg("build.sh")
        .status()
        .expect("Failed to execute RE-flex build script");

    if !status.success() {
        panic!("RE-flex build script failed");
    }

    env::set_current_dir(&current_dir).expect("Failed to change back to original directory");
    let out_dir = current_dir.join("RE-flex");

    println!("cargo:rustc-link-search=native={}/lib", out_dir.display());
    println!("cargo:rustc-link-lib=static=reflex");

    cxx_build::bridge("src/lib.rs")
        .file("cxx/fuzzy_matcher.cpp")
        .include("RE-flex/include")
        .flag_if_supported("-std=c++14")
        .compile("fuzzy_matcher");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cxx/fuzzy_matcher.hpp");
    println!("cargo:rerun-if-changed=cxx/fuzzy_matcher.cpp");
    println!("cargo:rerun-if-changed=RE-flex/include");
    println!("cargo:rerun-if-changed=RE-flex/src");
}
