use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(md4c)");

    let md4c_dir = md4c_dir();
    let src_dir = md4c_dir.join("src");
    let md4c_c = src_dir.join("md4c.c");
    let md4c_html_c = src_dir.join("md4c-html.c");
    let md4c_entity_c = src_dir.join("entity.c");

    assert!(
        md4c_c.exists() && md4c_html_c.exists() && md4c_entity_c.exists(),
        "MD4C_DIR must point to an md4c checkout containing src/md4c.c, src/md4c-html.c, and src/entity.c"
    );

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
    env::var_os("MD4C_DIR")
        .map(PathBuf::from)
        .expect("set MD4C_DIR to the explicit md4c checkout used for comparison")
}
