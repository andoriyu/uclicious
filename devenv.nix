{pkgs, ...}: {
  # The Rust toolchain and cargo tooling come from the imported
  # molds/devenv/rust module chain (see devenv.yaml). The packages below are
  # the libucl FFI build inputs for uclicious-libucl-sys.
  packages = with pkgs; [
    pkg-config
    libucl
    rust-bindgen # for crates/uclicious-libucl-sys/generate-bindings.sh
  ];

  # bindgen needs libclang to parse ucl.h when regenerating the committed
  # FFI bindings.
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

  enterShell = ''
    echo "uclicious dev environment"
    echo "  libucl: $(pkg-config --modversion libucl 2>/dev/null || echo 'not found')"
  '';
}
