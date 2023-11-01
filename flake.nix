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
        packages.nil = nil;
        packages.clang-tools = clang-tools_16;
        packages.lldb = llvmPackages_16.lldb;
        packages.default = 
           llvmPackages_16.stdenv.mkDerivation {
            name = name;

            src = ./.;

            buildInputs = [
              packages.clang-tools
              packages.lldb
              openmpi
              gnumake
              zlib
              
              packages.nil
            ];
          };
      }
    );
}