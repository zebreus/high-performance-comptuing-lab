{
  description = "High-performance computing projects";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=refs/heads/master";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      with nixpkgs.legacyPackages.${system};
      rec {
        name = "high-performance-computing";
        packages.default =
          llvmPackages_16.stdenv.mkDerivation {
            name = name;
            src = ./.;

            buildInputs = [
              llvmPackages_16.openmp
              openmpi
              gnumake
              zlib

              clang-tools_16
              lldb
              nil
            ];
          };
      }
    );
}
