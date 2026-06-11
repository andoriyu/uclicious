//! Link against the system-provided libucl via pkg-config.
//!
//! Bindings are pre-generated and committed (`src/lib.rs`); they are not
//! produced at build time, so consumers do not need libclang/bindgen. Use
//! `./generate-bindings.sh` inside the devenv shell to regenerate them when
//! libucl changes. See the crate README / kiln FD-116 for the rationale.

fn main() {
    pkg_config::Config::new()
        .atleast_version("0.9.0")
        .probe("libucl")
        .expect(
            "libucl >= 0.9 not found via pkg-config. \
             Install libucl (the Nix devenv provides it) and ensure \
             PKG_CONFIG_PATH points at its lib/pkgconfig directory.",
        );
}
