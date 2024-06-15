{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (pkgs.lib.hasSuffix "\.css" path) ||
            (pkgs.lib.hasSuffix "\.js" path) ||
            (pkgs.lib.hasSuffix "\.svg" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };
        commonArgs = { inherit src; };

        ui = pkgs.buildNpmPackage {
          pname = "ui";
          version = "0.0.0";
          src = ./ui;
          npmDepsHash = "sha256-Q7AVCPU+rZhsMRts3Sn/P2G22pePOI0BAnMrcXkUgUo=";
          installPhase = ''
            cp -pr --reflink=auto -- dist "$out/"
          '';
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          preBuild = ''
            cp -pr --reflink=auto -- ${ui} ui/dist
          '';
        });
      in
      {
        packages = {
          default = bin;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.emmet-ls
            pkgs.cargo-watch
            pkgs.rust-analyzer
            rustToolchain

            pkgs.nodejs
            pkgs.nodePackages.typescript-language-server
            pkgs.nodePackages.vscode-langservers-extracted
            pkgs.nodePackages."@tailwindcss/language-server"

            pkgs.httpie
            pkgs.sqlite
          ];
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
