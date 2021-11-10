{
  description = "Minimal Rust Development Environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    andoriyu = {
      url = "github:andoriyu/flakes";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };
    devshell.url = "github:numtide/devshell/master";
  };
  outputs =
    { self, nixpkgs, rust-overlay, flake-utils, andoriyu, devshell, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        cwd = builtins.toString ./.;
        overlays = [ devshell.overlay rust-overlay.overlay andoriyu.overlay ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile "${cwd}/rust-toolchain.toml";
      in with pkgs; {
        devShell = clangStdenv.mkDerivation {
        name = "rust";
        nativeBuildInputs = [
            clangStdenv
            binutils
            gnumake
            cmake
            openssl
            openssl.dev
            pkgconfig
            rust
            rust-analyzer
            cargo-expand-nightly
            cargo-release
            git-cliff
          ];
          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
          OPENSSL_DIR = "${openssl.bin}/bin";
          OPENSSL_LIB_DIR = "${openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${openssl.out.dev}/include";
        };
      });
}

