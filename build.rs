use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let md4c_dir = md4c_dir();
    let src_dir = md4c_dir.join("src");
    let md4c_c = src_dir.join("md4c.c");
    let md4c_html_c = src_dir.join("md4c-html.c");
    let md4c_h = src_dir.join("md4c.h");
    let md4c_html_h = src_dir.join("md4c-html.h");

    if !md4c_c.exists() || !md4c_html_c.exists() {
        panic!(
            "md4c sources not found. Set MD4C_DIR or ensure ../md4c exists with src/md4c.c and src/md4c-html.c"
        );
    }

    println!("cargo:rerun-if-env-changed=MD4C_DIR");
    println!("cargo:rerun-if-changed={}", md4c_c.display());
    println!("cargo:rerun-if-changed={}", md4c_html_c.display());
    println!("cargo:rerun-if-changed={}", md4c_h.display());
    println!("cargo:rerun-if-changed={}", md4c_html_h.display());

    cc::Build::new()
        .file(md4c_c)
        .file(md4c_html_c)
        .include(&src_dir)
        .flag_if_supported("-std=c99")
        .compile("md4c");
}

fn md4c_dir() -> PathBuf {
    if let Ok(dir) = env::var("MD4C_DIR") {
        return PathBuf::from(dir);
    }
    let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("../md4c");
    fallback
}
