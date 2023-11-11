{
  description = "High-performance computing projects";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=refs/heads/master";
    nixpkgs-asciidoc.url = "github:zebreus/nixpkgs?ref=f1a3be7a1160cc4810c0274ab76f0fac813eb4d6";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, nixpkgs-asciidoc }:
    flake-utils.lib.eachDefaultSystem (system:
      with nixpkgs.legacyPackages.${system};
      let
        pkgs-with-asciidoc = import nixpkgs-asciidoc { inherit system; };
      in
      rec {

        name = "high-performance-computing";
        packages.default =
          llvmPackages_16.stdenv.mkDerivation {
            name = name;
            src = ./.;

            buildInputs = [
              llvmPackages_16.openmp
              gcc12
              clang_16

              openmpi
              gnumake
              zlib
              sshfs

              clang-tools_16
              lldb
              nil

              pkgs-with-asciidoc.python3
              pkgs-with-asciidoc.deno
              pkgs-with-asciidoc.asciidoctor-web-pdf
              pkgs-with-asciidoc.asciidoctor-js
              pkgs-with-asciidoc.sass
              pkgs-with-asciidoc.gnumake
              pkgs-with-asciidoc.nixpkgs-fmt
              pkgs-with-asciidoc.nil
              pkgs-with-asciidoc.jq
            ];
          };
      }
    );
}
