{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (pkgs.lib.hasSuffix "\.css" path)
            || (pkgs.lib.hasSuffix "\.js" path)
            || (pkgs.lib.hasSuffix "\.svg" path)
            || (pkgs.lib.hasSuffix "\.sqlite3" path)
            || (craneLib.filterCargoSources path type);
        };

        commonArgs = {
          inherit src;
          buildInputs = [
            pkgs.git
            pkgs.pkg-config
          ];
          nativeBuildInputs = [ pkgs.openssl ];
        };

        ui = pkgs.buildNpmPackage {
          pname = "ui";
          version = "0.0.0";
          src = ./ui;
          npmDepsHash = "sha256-RVVCmlfembWI+MLxt+96V2Xmczkscuw79aNPWtYlGG8=";
          installPhase = ''
            cp -pr --reflink=auto -- dist "$out/"
          '';
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            preBuild = ''
              cp -pr --reflink=auto -- ${ui} ui/dist
            '';
          }
        );

        docker = pkgs.dockerTools.buildLayeredImage {
          name = "sql-studio";
          tag = "latest";
          created = "now";
          contents = [ bin ];
        };

        version = "0.1.44";
        deploy = pkgs.writeShellScriptBin "deploy" ''
          ${pkgs.skopeo}/bin/skopeo --insecure-policy copy docker-archive:${docker} docker://docker.io/frectonz/sql-studio:${version} --dest-creds="frectonz:$ACCESS_TOKEN"
          ${pkgs.skopeo}/bin/skopeo --insecure-policy copy docker://docker.io/frectonz/sql-studio:${version} docker://docker.io/frectonz/sql-studio:latest --dest-creds="frectonz:$ACCESS_TOKEN"
        '';
      in
      {
        packages = {
          inherit deploy docker ui;
          default = bin;
          stable = pkgs.callPackage ./package.nix { };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.bacon
            pkgs.emmet-ls
            pkgs.cargo-dist
            pkgs.cargo-watch
            pkgs.rust-analyzer
            pkgs.cargo-outdated
            rustToolchain

            pkgs.nodejs
            pkgs.nodePackages.typescript-language-server
            pkgs.nodePackages.vscode-langservers-extracted
            pkgs.nodePackages."@tailwindcss/language-server"

            pkgs.httpie
            pkgs.sqlite
            pkgs.prefetch-npm-deps
          ];
        };

        formatter = pkgs.treefmt.withConfig {
          runtimeInputs = [ pkgs.nixfmt-rfc-style ];

          settings = {
            # Log level for files treefmt won't format
            on-unmatched = "info";

            # Configure nixfmt for .nix files
            formatter.nixfmt = {
              command = "nixfmt";
              includes = [ "*.nix" ];
            };
          };
        };
      }
    );
}
