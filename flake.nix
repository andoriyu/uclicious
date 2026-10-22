{
  description = "Uclicious configuration libraries";
  inputs = {
    molds.url = "git+https://git.coldsteel.casa/Coldsteel/molds.git";
    nixpkgs.follows = "molds/nixpkgs-unstable";
    flake-parts.follows = "molds/flake-parts";
    crane.follows = "molds/crane";
  };
  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
      perSystem = {
        pkgs,
        lib,
        ...
      }: let
        base = inputs.crane.mkLib pkgs;
        craneLib = base.appendCrateRegistries [
          (base.registryFromSparse {
            indexUrl = "https://cargo.coldsteel.casa/api/v1/crates/";
            configSha256 = "086fydg7na3d72cki0ndymyd1mk8m635ax6ralp1f1ffqn3sz90y";
          })
        ];
        args = {
          pname = "uclicious-workspace";
          version = (lib.importTOML ./crates/uclicious/Cargo.toml).package.version;
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              craneLib.filterCargoSources path type
              || lib.hasSuffix ".sql" path
              || lib.hasSuffix ".toml" path
              || lib.hasSuffix ".md" path
              || lib.hasSuffix ".ucl" path
              || lib.hasSuffix ".h" path;
          };
          nativeBuildInputs = [pkgs.pkg-config];
          buildInputs = [pkgs.libucl];
          cargoExtraArgs = "--workspace";
          # Tests run in the independent Rust CI job with native libucl.
          doCheck = false;
        };
      in {
        packages.default = craneLib.buildPackage (args
          // {
            cargoArtifacts = craneLib.buildDepsOnly args;
          });
        formatter = pkgs.alejandra;
      };
    };
}
