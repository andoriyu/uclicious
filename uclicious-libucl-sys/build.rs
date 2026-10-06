//! Link against the system-provided libucl via pkg-config.
//!
//! Bindings are pre-generated and committed (`src/lib.rs`); they are not
//! produced at build time, so consumers do not need libclang/bindgen. Use
//! `./generate-bindings.sh` inside the Nix development shell to regenerate
//! them when libucl changes.

fn main() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    // docs.rs only generates documentation and does not provide libucl.
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }

    pkg_config::Config::new()
        .atleast_version("0.9.0")
        .probe("libucl")
        .expect(
            "libucl >= 0.9 not found via pkg-config. \
             Install libucl (the Nix development shell provides it) and ensure \
             PKG_CONFIG_PATH points at its lib/pkgconfig directory.",
        );
}
