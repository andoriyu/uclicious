# Uclicious

Rust configuration library for UCL, with derive macros and libucl FFI bindings.

**FD repo:** Coldsteel/foundry

The three crates stay together: `uclicious`, `uclicious_derive`, and
`uclicious-libucl-sys`. Native libucl 0.9.x and pkg-config are required.
Bindings are committed; consumers do not need bindgen or libclang.

Run commands from this repository through `devenv shell -- <command>`.
CI runs style, static analysis, instrumented tests and an independent Nix build.
Crates publish to the coldsteel registry after all master checks pass;
PR publishing plans are read-only. Do not publish manually.

Use scoped commit messages (`uclicious: description`), with no AI trailers.
Do not merge PRs; the owner merges them. All FDs remain in Coldsteel/foundry.
