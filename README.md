# Uclicious

Rust bindings to the Universal Configuration Library (libucl), with typed
configuration conversion and derive macros.

| Crate | Purpose |
|---|---|
| uclicious | Configuration parser, object types, conversion and variable handlers |
| uclicious_derive | Derive macros for typed configuration |
| uclicious-libucl-sys | Committed FFI bindings to system libucl 0.9.x |

See [the library guide](crates/uclicious/README.md) for usage.
Run `devenv shell -- cargo test --workspace` for tests. The development shell
provides libucl, pkg-config and tools for regenerating bindings. Downstream
builds need libucl and pkg-config, but do not need bindgen or libclang.

The crates retain their existing coldsteel registry names and versions.
Publishing is automatic after required master CI checks pass.

Development and Feature Designs: **Coldsteel/foundry**, including
[FD-1017](https://git.coldsteel.casa/Coldsteel/foundry/issues/1017).
History extraction provenance is recorded in [docs/extraction](docs/extraction).
Each crate retains its own license file and upstream authorship.
