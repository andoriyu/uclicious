{
  description = "uclicious — Rust wrapper around the Universal Configuration Library (UCL/libucl)";

  inputs = {
    molds.url = "git+ssh://forgejo@git.coldsteel.casa/Coldsteel/molds.git";
    nixpkgs.follows = "molds/nixpkgs-unstable";
    flake-parts.follows = "molds/flake-parts";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = {pkgs, ...}: {
        formatter = pkgs.alejandra;
      };
    };
}
