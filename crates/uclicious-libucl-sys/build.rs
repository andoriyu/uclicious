//! Link against the system-provided libucl via pkg-config, and compile the
//! small C shim in `src/refcount.c` (this needs a C compiler).
//!
//! Bindings are pre-generated and committed (`src/lib.rs`); they are not
//! produced at build time, so consumers do not need libclang/bindgen. Use
//! `./generate-bindings.sh` inside the devenv shell to regenerate them when
//! libucl changes. See the crate README / kiln FD-116 for the rationale.

fn main() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    // docs.rs only generates documentation and does not provide libucl, so
    // there is nothing to link and no headers to compile the shim against.
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }

    let mut libucl = pkg_config::Config::new();
    libucl.atleast_version("0.9.0");
    let missing = "libucl >= 0.9 not found via pkg-config. \
                   Install libucl (the Nix devenv provides it) and ensure \
                   PKG_CONFIG_PATH points at its lib/pkgconfig directory.";

    // GNU ld resolves symbols left to right, so each library must come before
    // the ones it depends on: the test fixture, then the shim, then libucl.
    // Find the headers first without emitting libucl's link flags.
    let headers = libucl
        .clone()
        .cargo_metadata(false)
        .probe("libucl")
        .expect(missing)
        .include_paths;

    // The fixture writes reference counts directly, so only this workspace's
    // tests build it, through a feature that consumers leave off.
    if std::env::var_os("CARGO_FEATURE_REFCOUNT_TEST_FIXTURE").is_some() {
        println!("cargo:rerun-if-changed=src/refcount_test.c");
        cc::Build::new()
            // Nix enables _FORTIFY_SOURCE, which requires optimization even in debug.
            .opt_level(1)
            .includes(&headers)
            .file("src/refcount_test.c")
            .compile("uclicious_refcount_test");
    }
    // Warnings stay warnings: a consumer's newer compiler must not break the build.
    println!("cargo:rerun-if-changed=src/refcount.c");
    cc::Build::new()
        .opt_level(1)
        .includes(&headers)
        .file("src/refcount.c")
        .compile("uclicious_refcount");
    libucl.probe("libucl").expect(missing);
}
