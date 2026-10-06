#!/bin/sh
# Regenerate the committed FFI bindings (src/lib.rs) against the
# system/Nix-provided libucl (0.9.x).
#
# Requires `bindgen` (libclang), `pkg-config`, and libucl headers on the
# include path — all provided by the project's Nix shell. Run from this
# crate directory:
#
#   ./generate-bindings.sh
set -eu

bindgen wrapper.h -o src/lib.rs \
    --bitfield-enum "(ucl_parser_flags.*|ucl_string_flags.*|ucl_object_flags.*)" \
    --blocklist-function "ucl_object_emit_file_funcs" \
    --blocklist-type "FILE|_IO_.*|__.*FILE.*|__mbstate_t|__sFILE|__sbuf|__ubuf" \
    --default-enum-style rust \
    --opaque-type "ucl_parser" \
    --opaque-type "ucl_object_s" \
    --allowlist-function "ucl_.*" \
    --allowlist-type "ucl_.*" \
    --raw-line '#![allow(non_camel_case_types)]' \
    --raw-line '#![allow(non_upper_case_globals)]' \
    --raw-line '#![allow(non_snake_case)]' \
    --raw-line '#![allow(dead_code)]' \
    --raw-line '#![allow(clippy::all)]' \
    --raw-line '#![allow(rustdoc::all)]' \
    -- -I"$(pkg-config --variable=includedir libucl)"
