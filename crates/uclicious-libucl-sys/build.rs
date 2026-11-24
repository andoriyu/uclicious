//! Link against the system-provided libucl via pkg-config, and compile the
//! small C shim in `src/refcount.c` (this needs a C compiler).
//!
//! Bindings are pre-generated and committed (`src/lib.rs`); they are not
//! produced at build time, so consumers do not need libclang/bindgen. Use
//! `./generate-bindings.sh` inside the devenv shell to regenerate them when
//! libucl changes. See the crate README / kiln FD-116 for the rationale.

fn main() {
    let library = pkg_config::Config::new()
        .atleast_version("0.9.0")
        .probe("libucl")
        .expect(
            "libucl >= 0.9 not found via pkg-config. \
             Install libucl (the Nix devenv provides it) and ensure \
             PKG_CONFIG_PATH points at its lib/pkgconfig directory.",
        );
    // Warnings stay warnings: a consumer's newer compiler must not break the build.
    println!("cargo:rerun-if-changed=src/refcount.c");
    cc::Build::new()
        // Nix enables _FORTIFY_SOURCE, which requires optimization even in debug.
        .opt_level(1)
        .includes(&library.include_paths)
        .file("src/refcount.c")
        .compile("uclicious_refcount");
    // The fixture writes reference counts directly, so only this workspace's
    // tests build it, through a feature that consumers leave off.
    if std::env::var_os("CARGO_FEATURE_REFCOUNT_TEST_FIXTURE").is_some() {
        println!("cargo:rerun-if-changed=src/refcount_test.c");
        cc::Build::new()
            .opt_level(1)
            .includes(&library.include_paths)
            .file("src/refcount_test.c")
            .compile("uclicious_refcount_test");
    }
}
