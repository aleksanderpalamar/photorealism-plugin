fn main() {
    println!("cargo:rerun-if-changed=src/dxgi.def");

    if std::env::var("CARGO_CFG_WINDOWS").is_err() {
        return;
    }

    println!("cargo:rustc-cdylib-link-arg=src/dxgi.def");
}
