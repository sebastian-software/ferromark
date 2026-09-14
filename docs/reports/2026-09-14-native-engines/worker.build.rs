fn main() {
    println!("cargo:rustc-link-search=native={}", "/private/tmp/ferromark-v2-native-comparison/build-01/native");
    println!("cargo:rustc-link-lib=static=bun_bench_native");
    println!("cargo:rustc-link-lib=static=md4c");
    println!("cargo:rustc-link-lib=static=mimalloc");
    println!("cargo:rustc-link-lib=c++");
}
