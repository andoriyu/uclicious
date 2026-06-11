# uclicious

Rust wrapper around the Universal Configuration Library (UCL / `libucl`), with a
`#[derive(Uclicious)]` macro for deserializing UCL into typed structs.

This is a **first-party** crate, developed here in foundry. GitHub
(`github.com/andoriyu/uclicious`) is export-only. It is published to the
**`coldsteel`** cargo registry and consumed by other foundry projects (notably
`kiln`, see FD-116) by `version` + `registry = "coldsteel"`.

Read `../docs/agent_guide/summary.md` first. In particular:
- `scoped_commits.md` — `<scope>: <description>`, no `<type>` prefix, no AI attribution trailers. Scope here is `uclicious`.
- `structured_logging_conventions.md`, `no_silent_fallback_values.md`, `dry_helpers_utilities.md`.

## Layout

- `crates/uclicious` — main library + `FromObject` trait and runtime (`raw`, variable handlers).
- `crates/uclicious-derive` — `uclicious_derive` proc-macro (crate name keeps the underscore).
- `crates/uclicious-libucl-sys` — low-level FFI to `libucl`.

## libucl / FFI

- The sys crate links the **Nix-provided libucl 0.9.x** via **pkg-config**
  (`build.rs`); there is no bundled/cmake build.
- FFI bindings are **pre-generated and committed** to
  `crates/uclicious-libucl-sys/src/lib.rs`, so downstream consumers do **not**
  need libclang/bindgen. Regenerate them with
  `cd crates/uclicious-libucl-sys && ./generate-bindings.sh` inside the devenv
  shell when libucl changes.
- `devenv.nix` provides `libucl`, `pkg-config`, and `rust-bindgen`.

## Conventions

- Errors: `thiserror` in libraries.
- Build/test inside the devenv shell (`direnv allow` / `devenv shell`) so
  `pkg-config` finds libucl: `cargo test`.
