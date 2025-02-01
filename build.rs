fn main() {
    if std::env::var("CARGO_FEATURE_CDYLIB").is_ok() {
        println!("cargo:rustc-crate-type=cdylib");
    } else if std::env::var("CARGO_FEATURE_LIB").is_ok() {
        println!("cargo:rustc-crate-type=lib");
    }
}
