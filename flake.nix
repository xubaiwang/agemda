{
  description = "Flake utils demo";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          default =
            with pkgs;
            rustPlatform.buildRustPackage {
              pname = "agemda";
              version = "0.1.0";
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
              buildInputs = [ xdg-utils ];
            };
        };

        devShells.default =
          with pkgs;
          mkShell {
            packages = [
              rustfmt
              rust-analyzer
              clippy
            ];
            nativeBuildInputs = [
              rustc
              cargo
            ];
          };
      }
    );
}
