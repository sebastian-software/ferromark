use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CMARK_SOURCE");
    println!("cargo:rerun-if-env-changed=CMARK_BUILD");
    println!("cargo:rerun-if-changed=bridge.c");
    let source = PathBuf::from(env::var_os("CMARK_SOURCE").expect("use prepare.py"));
    let build = PathBuf::from(env::var_os("CMARK_BUILD").expect("use prepare.py"));
    let gfm = env::var_os("CARGO_FEATURE_CMARK_GFM").is_some();
    let mut cc = cc::Build::new();
    cc.file("bridge.c")
        .include(source.join("src"))
        .include(build.join("src"))
        .flag_if_supported("-std=c99");
    if gfm {
        cc.define("BENCH_GFM", None)
            .define("CMARK_GFM_STATIC_DEFINE", None)
            .define("CMARK_GFM_EXTENSIONS_STATIC_DEFINE", None)
            .include(source.join("extensions"))
            .include(build.join("extensions"));
    } else {
        cc.define("CMARK_STATIC_DEFINE", None);
    }
    cc.compile("cmark_bench_bridge");
    println!(
        "cargo:rustc-link-search=native={}",
        build.join("src").display()
    );
    if gfm {
        println!(
            "cargo:rustc-link-search=native={}",
            build.join("extensions").display()
        );
        println!("cargo:rustc-link-lib=static=cmark-gfm-extensions");
        println!("cargo:rustc-link-lib=static=cmark-gfm");
    } else {
        println!("cargo:rustc-link-lib=static=cmark");
    }
}
