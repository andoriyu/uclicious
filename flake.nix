{
  description = "Minimal Rust Development Environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    andoriyu = {
      url = "github:andoriyu/flakes";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        fenix.follows = "fenix";
      };
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    attic = {
      url = "github:zhaofengli/attic";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
    andoriyu,
    crane,
    attic,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      cwd = builtins.toString ./.;
      overlays = [ fenix.overlays.default ];
      pkgs = import nixpkgs {inherit system overlays;};
      control-plane = pkgs.callPackage ./nix/control-plane {inherit nixpkgs system crane fenix andoriyu;};
    in
      with pkgs; {
        formatter = nixpkgs.legacyPackages.${system}.alejandra;
        devShell = clangStdenv.mkDerivation rec {
          name = "rust";
          nativeBuildInputs = [
            (with fenix.packages.${system};
              combine [
                (stable.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
              ])
            andoriyu.packages.${system}.git-cliff
            bacon
            binutils
            cargo-cache
            cargo-deny
            cargo-diet
            cargo-nextest
            cargo-outdated
            cargo-sort
            cargo-sweep
            cargo-wipe
            cargo-workspaces
            cmake
            curl
            gnumake
            pkg-config
            rust-analyzer-nightly
            zlib
  
          ];
        };
      });
}
