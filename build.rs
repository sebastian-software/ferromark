use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(md4c)");

    let md4c_dir = md4c_dir();
    let src_dir = md4c_dir.join("src");
    let md4c_c = src_dir.join("md4c.c");
    let md4c_html_c = src_dir.join("md4c-html.c");
    let md4c_entity_c = src_dir.join("entity.c");

    if !md4c_c.exists() || !md4c_html_c.exists() || !md4c_entity_c.exists() {
        println!(
            "cargo:warning=md4c sources not found, comparison benchmarks will exclude md4c. \
             Set MD4C_DIR or clone md4c into ../md4c"
        );
        return;
    }

    let md4c_h = src_dir.join("md4c.h");
    let md4c_html_h = src_dir.join("md4c-html.h");
    let md4c_entity_h = src_dir.join("entity.h");

    println!("cargo:rerun-if-env-changed=MD4C_DIR");
    println!("cargo:rerun-if-changed={}", md4c_c.display());
    println!("cargo:rerun-if-changed={}", md4c_html_c.display());
    println!("cargo:rerun-if-changed={}", md4c_entity_c.display());
    println!("cargo:rerun-if-changed={}", md4c_h.display());
    println!("cargo:rerun-if-changed={}", md4c_html_h.display());
    println!("cargo:rerun-if-changed={}", md4c_entity_h.display());

    cc::Build::new()
        .file(md4c_c)
        .file(md4c_html_c)
        .file(md4c_entity_c)
        .include(&src_dir)
        .flag_if_supported("-std=c99")
        .compile("md4c");

    println!("cargo:rustc-cfg=md4c");
}

fn md4c_dir() -> PathBuf {
    if let Ok(dir) = env::var("MD4C_DIR") {
        return PathBuf::from(dir);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../md4c")
}
