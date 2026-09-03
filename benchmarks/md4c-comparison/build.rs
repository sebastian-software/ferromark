use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const MD4C_REVISION: &str = "65c6c9d72cebd9a731aaa5597414ce04d9ea5de3";

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
    let directory = env::var_os("MD4C_DIR")
        .map(PathBuf::from)
        .expect("set MD4C_DIR to the explicit md4c checkout used for comparison");
    verify_md4c_revision(&directory);
    directory
}

fn verify_md4c_revision(directory: &Path) {
    let output = Command::new("git")
        .args(["-C"])
        .arg(directory)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git must be available to verify the pinned md4c revision");
    assert!(
        output.status.success(),
        "MD4C_DIR must point to a git checkout so its pinned revision can be verified"
    );

    let revision =
        String::from_utf8(output.stdout).expect("git rev-parse must return a UTF-8 commit hash");
    assert_eq!(
        revision.trim(),
        MD4C_REVISION,
        "MD4C_DIR must be checked out at md4c revision {MD4C_REVISION}"
    );
}
