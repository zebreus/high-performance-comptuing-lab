{
  description = "High-performance computing projects";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=refs/heads/master";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      rec {
        name = "high-performance-computing";
        packages.default = (with nixpkgs.legacyPackages.${system};
          stdenv.mkDerivation {
            name = name;

            src = ./.;

            buildInputs = [
              openmpi
              gnumake
              zlib

              nil
            ];
          });


      }
    );
}