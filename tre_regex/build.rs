extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let clang_args: Vec<String> = Vec::new();
    let include_path;

    #[cfg(feature = "vendored")]
    {
        use autotools::Config;
        use fs_extra::dir::{copy, remove, CopyOptions};
        use std::process::Command;

        let tre_path = out_path.join("tre");
        remove(&tre_path).ok();
        copy("tre", &out_path, &CopyOptions::new()).expect("Failed to copy tre!");

        Command::new("sh")
            .current_dir(&tre_path)
            .args(["-c", "./utils/autogen.sh"])
            .status()
            .expect("Could not run autogen.sh! Is autotools installed?");

        let dst = Config::new(&tre_path)
            .enable_static()
            .disable_shared()
            .disable("agrep", None)
            .disable("wchar", None)
            .enable("approx", None)
            .build();

        remove(tre_path).expect("Could not clean up tre dir!");

        println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
        println!("cargo:rustc-link-lib=static=tre");
        println!("cargo:rustc-link-lib=c");

        include_path = out_path.join("include").join("tre").join("tre.h").to_str().unwrap().to_string();
    }
    #[cfg(not(feature = "vendored"))]
    {
        let library = pkg_config::Config::new().statik(true).find("tre").unwrap();
        clang_args.extend(library.include_paths.iter().map(|path| format!("-I{}", path.to_string_lossy())));
        include_path = library.include_paths.iter().find(|path| path.join("tre.h").exists())
            .expect("Could not find tre.h in include paths")
            .join("tre.h")
            .to_str()
            .unwrap()
            .to_string();
    }

    let mut bindings = bindgen::Builder::default()
        .clang_args(clang_args)
        .header(include_path)
        .derive_default(true)
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .newtype_enum("reg_errcode_t")
        .opaque_type("regex_t")
        .allowlist_function("tre_.*")
        .allowlist_type("(reg.*_t|tre_.*)")
        .allowlist_var("REG_.*")
        .blocklist_type("register_t");

    if !cfg!(feature = "wchar") {
        bindings = bindings.blocklist_function("tre_reg(a)?w(n)?(comp|exec)");
    }

    if !cfg!(feature = "approx") {
        bindings = bindings
            .blocklist_function("tre_rega(w)?(n)?exec")
            .blocklist_type("rega(match|params)_t")
            .blocklist_item("REG_APPROX_MATCHER");
    }

    bindings.generate().expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}